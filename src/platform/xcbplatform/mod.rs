
// for Atom names
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(unused_variables)]

mod xcbkeyboard;

use platform::{Platform, EventLoop, WinId};
use window::{self, Window, WindowBase};
use geometry::*;

use xcb::{self, dri2};

use std::rc::Rc;
use std::cell::Cell;
use std::collections::{HashMap};
use std::mem;


iterable_key_enum! {
    Atom =>
        UTF8_STRING,

        WM_PROTOCOLS,
        WM_DELETE_WINDOW,
        WM_TRANSIENT_FOR,
        WM_CHANGE_STATE,
        WM_STATE,
        _NET_WM_STATE,
        _NET_WM_STATE_MODAL,
        _NET_WM_STATE_STICKY,
        _NET_WM_STATE_MAXIMIZED_VERT,
        _NET_WM_STATE_MAXIMIZED_HORZ,
        _NET_WM_STATE_SHADED,
        _NET_WM_STATE_SKIP_TASKBAR,
        _NET_WM_STATE_SKIP_PAGER,
        _NET_WM_STATE_HIDDEN,
        _NET_WM_STATE_FULLSCREEN,
        _NET_WM_STATE_ABOVE,
        _NET_WM_STATE_BELOW,
        _NET_WM_STATE_DEMANDS_ATTENTION,
        _NET_WM_STATE_FOCUSED,
        _NET_WM_NAME
}

const XCB_ICCCM_WM_STATE_WITHDRAWN: u32 = 0;
const XCB_ICCCM_WM_STATE_NORMAL: u32 = 1;
const XCB_ICCCM_WM_STATE_ICONIC: u32 = 3;

const NET_WM_STATE_NONE: u32 = 0x0000;
const NET_WM_STATE_MODAL: u32 = 0x0001;
const NET_WM_STATE_STICKY: u32 = 0x0002;
const NET_WM_STATE_MAXIMIZED_VERT: u32 = 0x0004;
const NET_WM_STATE_MAXIMIZED_HORZ: u32 = 0x0008;
const NET_WM_STATE_MAXIMIZED: u32 = 0x000C;
const NET_WM_STATE_SHADED: u32 = 0x0010;
const NET_WM_STATE_SKIP_TASKBAR: u32 = 0x0020;
const NET_WM_STATE_SKIP_PAGER: u32 = 0x0040;
const NET_WM_STATE_HIDDEN: u32 = 0x0080;
const NET_WM_STATE_FULLSCREEN: u32 = 0x0100;
const NET_WM_STATE_ABOVE: u32 = 0x0200;
const NET_WM_STATE_BELOW: u32 = 0x0400;
const NET_WM_STATE_DEMANDS_ATTENTION: u32 = 0x0800;
const NET_WM_STATE_FOCUSED: u32 = 0x1000;


fn wid_to_xcb(wid: WinId) -> xcb::Window {
    debug_assert!(wid >> 32 == 0);
    wid as xcb::Window
}

fn xcb_to_wid(xcb: xcb::Window) -> WinId {
    xcb as WinId
}


struct XcbSharedState {
    conn: xcb::Connection,
    def_screen: usize,
    atoms: HashMap<Atom, xcb::Atom>,
}

impl XcbSharedState {
    fn atom(&self, atom: Atom) -> xcb::Atom {
        *self.atoms.get(&atom).unwrap()
    }
}


pub struct XcbPlatform {
    shared_state: Rc<XcbSharedState>,
    windows: HashMap<WinId, XcbWindow>,
    kbd: xcbkeyboard::XcbKeyboard,
    kbd_ev: u8,
    dri2_ev: u8,
    exit_code: Option<i32>,
}


impl XcbPlatform {
    pub fn new() -> Option<XcbPlatform> {

        xcb::Connection::connect_with_xlib_display().map(|(conn, def_screen)| {
            conn.set_event_queue_owner(xcb::EventQueueOwner::Xcb);

            let atoms = {
                let mut cookies = Vec::with_capacity(Atom::num_variants());
                for atom in Atom::variants() {
                    let atom_name = format!("{:?}", atom);
                    cookies.push(
                        xcb::intern_atom(&conn, true, &atom_name)
                    );
                }
                let mut atoms = HashMap::<Atom, xcb::Atom>::with_capacity(Atom::num_variants());
                for (i, atom) in Atom::variants().enumerate() {
                    atoms.insert(*atom,
                        match cookies[i].get_reply() {
                            Ok(r) => { r.atom() },
                            Err(_) => {
                                panic!("could not find atom {:?}", atom);
                            }
                        }
                    );
                }
                atoms
            };

            let (kbd, kbd_ev, _) = xcbkeyboard::XcbKeyboard::new(&conn);

            let dri2_ev = {
                conn.prefetch_extension_data(dri2::id());
                match conn.get_extension_data(dri2::id()) {
                    None => { panic!("could not load dri2 extension") },
                    Some(r) => { r.first_event() }
                }
            };

            XcbPlatform {
                shared_state: Rc::new(XcbSharedState {
                    conn: conn,
                    def_screen: def_screen as usize,
                    atoms: atoms,
                }),
                windows: HashMap::new(),
                kbd: kbd,
                kbd_ev: kbd_ev,
                dri2_ev: dri2_ev,
                exit_code: None,
            }
        }).ok()
    }

    fn conn(&self) -> &xcb::Connection {
        &self.shared_state.conn
    }

    fn atom(&self, atom: Atom) -> xcb::Atom {
        self.shared_state.atom(atom)
    }

    fn xcb_win(&self, win: xcb::Window) -> &XcbWindow {
        self.windows.get(&xcb_to_wid(win)).expect("try to access unregistered window")
    }
    fn xcb_win_mut(&mut self, win: xcb::Window) -> &mut XcbWindow {
        self.windows.get_mut(&xcb_to_wid(win)).expect("try to access unregistered window")
    }

    fn handle_client_message(&mut self, ev: &xcb::ClientMessageEvent) {
        let wm_protocols = self.atom(Atom::WM_PROTOCOLS);
        let wm_delete_window = self.atom(Atom::WM_DELETE_WINDOW);

        if ev.type_() == wm_protocols && ev.format() == 32 {
            let protocol = ev.data().data32()[0];
            if protocol == wm_delete_window {
                let close_window = {
                    let w = self.window_mut(xcb_to_wid(ev.window()));
                    let handler = w.on_close();
                    let cw = handler.borrow_mut()
                        .fire_or(true, w);
                    cw
                };
                if close_window {
                    // on_close was not set or returned true
                    // dropping the window will close it
                    self.windows.remove(&xcb_to_wid(ev.window()));
                    if self.windows.is_empty() && self.exit_code.is_none() {
                        self.exit_code = Some(0);
                    }
                }
            }
        }
    }

    fn handle_configure_notify(&mut self, ev: &xcb::ConfigureNotifyEvent) {
        self.xcb_win_mut(ev.event()).handle_configure_notify(&ev);
    }
}


impl Platform for XcbPlatform {
    fn create_window(&mut self) -> WinId {
        let w = XcbWindow::new(self.shared_state.clone());
        let wid = xcb_to_wid(w.xcb_win);
        self.windows.insert(wid, w);
        wid
    }

    fn window(&self, wid: WinId) -> &Window {
        self.windows.get(&wid).expect("try to access unregistered window")
    }
    fn window_mut(&mut self, wid: WinId) -> &mut Window {
        self.windows.get_mut(&wid).expect("try to access unregistered window")
    }
}

impl EventLoop for XcbPlatform {
    fn loop_events(&mut self) -> i32 {
        while let Some(ev) = self.shared_state.conn.wait_for_event() {
            let r = ev.response_type() & !0x80;
            match r {
                xcb::CLIENT_MESSAGE => {
                    self.handle_client_message(xcb::cast_event(&ev));
                },
                xcb::CONFIGURE_NOTIFY => {
                    self.handle_configure_notify(xcb::cast_event(&ev));
                },
                _ => {}
            }
            if self.exit_code.is_some() { break; }
        }
        // 2 ways to exit event loop:
        //  - setting exit_code
        //  - error (wait_for_event returns None)
        self.exit_code.expect(
            "XCB event loop was exited abruptly"
        )
    }
}


pub struct XcbWindow {
    base: WindowBase,
    shared_state: Rc<XcbSharedState>,
    xcb_win: xcb::Window,
    title: String,
    rect: IRect,
    last_known_state: window::State,
    created: bool,
}


impl XcbWindow {
    fn new(shared_state: Rc<XcbSharedState>) -> XcbWindow {

        let xcb_win = shared_state.conn.generate_id();

        XcbWindow {
            base: WindowBase::new(),
            shared_state: shared_state,
            xcb_win: xcb_win,
            title: "".to_string(),
            rect: IRect::new(0, 0, 0, 0),
            last_known_state: window::State::Hidden,
            created: false,
        }
    }

    fn conn(&self) -> &xcb::Connection {
        &self.shared_state.conn
    }

    fn atom(&self, atom: Atom) -> xcb::Atom {
        self.shared_state.atom(atom)
    }

    fn screen_root(&self, ind: usize) -> xcb::Window {
        let setup = self.conn().get_setup();
        setup.roots().nth(ind).unwrap().root()
    }

    fn def_screen_root(&self) -> xcb::Window {
        let setup = self.conn().get_setup();
        setup.roots().nth(self.shared_state.def_screen).unwrap().root()
    }

    fn create(&mut self) {
        let (s, p);
        {
            let setup = self.conn().get_setup();
            let screen = setup.roots().nth(self.shared_state.def_screen).unwrap();

            s = ISize::new(640, 480);
            p = IPoint::new(
                (screen.width_in_pixels() as i32 - s.w) / 2,
                (screen.height_in_pixels() as i32 - s.h) / 2);
            let values = [
                (xcb::CW_BACK_PIXEL,    screen.white_pixel()),

                (xcb::CW_EVENT_MASK,    xcb::EVENT_MASK_KEY_PRESS |
                                        xcb::EVENT_MASK_KEY_RELEASE |
                                        xcb::EVENT_MASK_BUTTON_PRESS |
                                        xcb::EVENT_MASK_BUTTON_RELEASE |
                                        xcb::EVENT_MASK_ENTER_WINDOW |
                                        xcb::EVENT_MASK_LEAVE_WINDOW |
                                        xcb::EVENT_MASK_POINTER_MOTION |
                                        xcb::EVENT_MASK_BUTTON_MOTION |
                                        xcb::EVENT_MASK_EXPOSURE |
                                        xcb::EVENT_MASK_STRUCTURE_NOTIFY |
                                        xcb::EVENT_MASK_PROPERTY_CHANGE),
            ];

            let wc = xcb::create_window(self.conn(), xcb::COPY_FROM_PARENT as u8,
                self.xcb_win, screen.root(), p.x as i16, p.y as i16,
                s.w as u16, s.h as u16, 0, xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
                screen.root_visual(), &values);

            let wm_delete_window = self.atom(Atom::WM_DELETE_WINDOW);
            let wm_protocols = self.atom(Atom::WM_PROTOCOLS);

            let values = [wm_delete_window];
            let dc = xcb::change_property(self.conn(), xcb::PROP_MODE_REPLACE as u8,
                    self.xcb_win, wm_protocols, xcb::ATOM_ATOM, 32, &values);

            assert!(wc.request_check().is_ok(), "could not create xcb window");
            assert!(dc.request_check().is_ok(), "could not prepare xcb window deletion handling");
        }

        // setup borrows self, so assignments must be after setup's lifetime
        self.last_known_state = window::State::Hidden;
        self.rect = IRect::new_ps(p, s);
        self.created = true;
        self.set_title_sys(&self.title);
    }

    fn created(&self) -> bool { self.created }

    fn handle_configure_notify(&mut self, ev: &xcb::ConfigureNotifyEvent) {
        debug_assert!(self.created());

        let new_pos = self.get_position_sys().unwrap_or(
            IPoint::new(ev.x() as i32, ev.y() as i32)
        );
        if new_pos != self.rect.point() {
        }

        let new_size = ISize::new(ev.width() as i32, ev.height() as i32);
        if new_size != self.rect.size() {
            let handler = self.on_resize();
            handler.borrow_mut().fire(self as &mut Window, new_size);
        }

        self.rect = IRect::new_ps(new_pos, new_size);
    }

    fn set_title_sys(&self, title: &str) {
        debug_assert!(self.created());
        xcb::change_property(self.conn(), xcb::PROP_MODE_REPLACE as u8,
            self.xcb_win, xcb::ATOM_WM_NAME, xcb::ATOM_STRING, 8, title.as_bytes());
        xcb::change_property(self.conn(), xcb::PROP_MODE_REPLACE as u8,
            self.xcb_win, xcb::ATOM_WM_ICON_NAME, xcb::ATOM_STRING, 8, title.as_bytes());
    }

    fn get_position_sys(&self) -> Option<IPoint> {
        debug_assert!(self.created());

        let cookie = xcb::translate_coordinates(self.conn(),
            self.xcb_win, self.def_screen_root(), 0, 0);
        cookie.get_reply().ok().map(|r| {
            IPoint::new(r.dst_x() as i32, r.dst_y() as i32)
        })
    }

    fn get_wm_states(&self) -> u32 {
        debug_assert!(self.created());

        let cookie = xcb::get_property(self.conn(), false,
                self.xcb_win, self.atom(Atom::_NET_WM_STATE),
                xcb::ATOM_ATOM, 0, 1024);
        let mut res: u32 = NET_WM_STATE_NONE;
        if let Ok(reply) = cookie.get_reply() {
            if reply.format() == 32 && reply.type_() == xcb::ATOM_ATOM {
                for a in reply.value::<u32>() {
                    let a = *a;
                    if a == self.atom(Atom::_NET_WM_STATE_MODAL) {
                        res |= NET_WM_STATE_MODAL;
                    }
                    else if a == self.atom(Atom::_NET_WM_STATE_STICKY) {
                        res |= NET_WM_STATE_STICKY;
                    }
                    else if a == self.atom(Atom::_NET_WM_STATE_MAXIMIZED_VERT) {
                        res |= NET_WM_STATE_MAXIMIZED_VERT;
                    }
                    else if a == self.atom(Atom::_NET_WM_STATE_MAXIMIZED_HORZ) {
                        res |= NET_WM_STATE_MAXIMIZED_HORZ;
                    }
                    else if a == self.atom(Atom::_NET_WM_STATE_SHADED) {
                        res |= NET_WM_STATE_SHADED;
                    }
                    else if a == self.atom(Atom::_NET_WM_STATE_SKIP_TASKBAR) {
                        res |= NET_WM_STATE_SKIP_TASKBAR;
                    }
                    else if a == self.atom(Atom::_NET_WM_STATE_SKIP_PAGER) {
                        res |= NET_WM_STATE_SKIP_PAGER;
                    }
                    else if a == self.atom(Atom::_NET_WM_STATE_HIDDEN) {
                        res |= NET_WM_STATE_HIDDEN;
                    }
                    else if a == self.atom(Atom::_NET_WM_STATE_FULLSCREEN) {
                        res |= NET_WM_STATE_FULLSCREEN;
                    }
                    else if a == self.atom(Atom::_NET_WM_STATE_ABOVE) {
                        res |= NET_WM_STATE_ABOVE;
                    }
                    else if a == self.atom(Atom::_NET_WM_STATE_BELOW) {
                        res |= NET_WM_STATE_BELOW;
                    }
                    else if a == self.atom(Atom::_NET_WM_STATE_DEMANDS_ATTENTION) {
                        res |= NET_WM_STATE_DEMANDS_ATTENTION;
                    }
                    else if a == self.atom(Atom::_NET_WM_STATE_FOCUSED) {
                        res |= NET_WM_STATE_FOCUSED;
                    }
                }
            }
        }
        res
    }

    fn set_wm_state(&self, yes: bool, atom1: xcb::Atom, atom2: xcb::Atom) {
        debug_assert!(self.created());

        let ev = xcb::ClientMessageEvent::new(
                32, self.xcb_win, self.atom(Atom::_NET_WM_STATE),
                xcb::ClientMessageData::from_data32([
                    if yes { 1 }else { 0 },
                    atom1, atom2, 0, 0
                ])
        );

        xcb::send_event(self.conn(), false, self.def_screen_root(),
            xcb::EVENT_MASK_STRUCTURE_NOTIFY |
            xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY |
            xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT,
            &ev);
    }

}

impl Window for XcbWindow {
    fn base(&self) -> &WindowBase { &self.base }
    fn base_mut(&mut self) -> &mut WindowBase { &mut self.base }
    fn title(&self) -> String {
        self.title.clone()
    }
    fn set_title(&mut self, title: String) {
        self.title = title;
        if self.created() {
            self.set_title_sys(&self.title);
            self.conn().flush();
        }
    }
    fn state(&self) -> window::State {
        if !self.created() {
            window::State::Hidden
        }
        else {
            let wm_state_atom = self.atom(Atom::WM_STATE);

            // checking for minimized
            let cookie = xcb::get_property_unchecked(self.conn(), false,
                    self.xcb_win, wm_state_atom, xcb::ATOM_ANY,
                    0, 1024);
            if let Ok(reply) = cookie.get_reply() {
                if reply.format() == 32 && reply.type_() == wm_state_atom {
                    let value = reply.value();
                    if value.len() > 0 {
                        match value[0] {
                            XCB_ICCCM_WM_STATE_WITHDRAWN => {
                                return window::State::Hidden;
                            },
                            XCB_ICCCM_WM_STATE_ICONIC => {
                                return window::State::Minimized;
                            },
                            _ => {}
                        }
                    }
                }
            }

            let states = self.get_wm_states();
            if (states & NET_WM_STATE_FULLSCREEN) != 0 {
                window::State::Fullscreen
            }
            else if (states & NET_WM_STATE_MAXIMIZED) == NET_WM_STATE_MAXIMIZED {
                window::State::Maximized
            }
            else {
                // default to normal
                window::State::Normal
            }
        }
    }

    fn set_state(&mut self, state: window::State) {
        if !self.created() { self.create(); }

        if self.last_known_state == state { return; }

        // removing attribute that makes other than normal
        match self.last_known_state {
            window::State::Maximized => {
                self.set_wm_state(false,
                    self.atom(Atom::_NET_WM_STATE_MAXIMIZED_HORZ),
                    self.atom(Atom::_NET_WM_STATE_MAXIMIZED_VERT));
            },
            window::State::Fullscreen => {
                self.set_wm_state(false,
                    self.atom(Atom::_NET_WM_STATE_FULLSCREEN), 0);
            },
            window::State::Minimized | window::State::Hidden => {
                xcb::map_window(self.conn(), self.xcb_win);
            },
            _ => {}
        }

        // at this point the window is in normal mode
        match state {
            window::State::Minimized => {
                let ev = xcb::ClientMessageEvent::new(32, self.xcb_win,
                    self.atom(Atom::WM_CHANGE_STATE),
                    xcb::ClientMessageData::from_data32([
                        XCB_ICCCM_WM_STATE_ICONIC, 0, 0, 0, 0
                    ])
                );

                xcb::send_event(self.conn(), false, self.def_screen_root(),
                    xcb::EVENT_MASK_STRUCTURE_NOTIFY |
                    xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY,
                    &ev);
            },
            window::State::Maximized => {
                self.set_wm_state(true,
                    self.atom(Atom::_NET_WM_STATE_MAXIMIZED_HORZ),
                    self.atom(Atom::_NET_WM_STATE_MAXIMIZED_VERT)
                );
            },
            window::State::Fullscreen => {
                self.set_wm_state(true,
                    self.atom(Atom::_NET_WM_STATE_FULLSCREEN), 0);
            },
            window::State::Hidden => {
                xcb::unmap_window(self.conn(), self.xcb_win);
            },
            _ => {}
        }
        self.conn().flush();
    }
}

impl Drop for XcbWindow {
    fn drop(&mut self) {
        if self.created() {
            if self.last_known_state != window::State::Hidden {
                xcb::unmap_window(self.conn(), self.xcb_win);
            }
            xcb::destroy_window(self.conn(), self.xcb_win);
            self.conn().flush();
        }
    }
}
