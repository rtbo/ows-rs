
// for Atom names
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(unused_variables)]

mod xcbkeyboard;

use ::{RcCell, WeakCell};
use platform::{Platform, PlatformWindow, EventLoop};
use window::{self, Window, WindowBase};
use geometry::*;

use xcb::{self, dri2};

use std::rc::{Rc, Weak};
use std::cell::{RefCell, Cell};
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



struct XcbSharedState {
    conn: xcb::Connection,
    def_screen: usize,
    atoms: HashMap<Atom, xcb::Atom>,
    windows: RefCell<HashMap<xcb::Window, Weak<XcbWindow>>>,
}

impl XcbSharedState {
    fn atom(&self, atom: Atom) -> xcb::Atom {
        *self.atoms.get(&atom).unwrap()
    }
}


pub struct XcbPlatform {
    shared_state: Rc<XcbSharedState>,
    kbd: xcbkeyboard::XcbKeyboard,
    kbd_ev: u8,
    dri2_ev: u8,
    exit_code: Cell<Option<i32>>,
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
                    windows: RefCell::new(HashMap::new()),
                }),
                kbd: kbd,
                kbd_ev: kbd_ev,
                dri2_ev: dri2_ev,
                exit_code: Cell::new(None),
            }
        }).ok()
    }

    fn conn(&self) -> &xcb::Connection {
        &self.shared_state.conn
    }

    fn atom(&self, atom: Atom) -> xcb::Atom {
        self.shared_state.atom(atom)
    }

    fn window(&self, xcb_win: xcb::Window) -> Option<Rc<XcbWindow>> {
        let windows = self.shared_state.windows.borrow();
        windows.get(&xcb_win).and_then(|ww| Weak::upgrade(&ww))
    }

    fn handle_client_message(&self, ev: &xcb::ClientMessageEvent) {
        let wm_protocols = self.atom(Atom::WM_PROTOCOLS);
        let wm_delete_window = self.atom(Atom::WM_DELETE_WINDOW);

        if ev.type_() == wm_protocols && ev.format() == 32 {
            let protocol = ev.data().data32()[0];
            if protocol == wm_delete_window {
                if let Some(pw) = self.window(ev.window()) {
                    if handler_fire_or!(pw.base.borrow().on_close.clone(), true,
                            make_window(pw.clone())) {
                        pw.close();
                        if self.shared_state.windows.borrow().is_empty() &&
                                self.exit_code.get().is_none() {
                            self.exit_code.set(Some(0));
                        }
                    }
                }
            }
        }
    }

    fn handle_configure_notify(&self, ev: &xcb::ConfigureNotifyEvent) {
        if let Some(w) = self.window(ev.event()) {
            w.handle_configure_notify(&ev);
        }
    }

    fn handle_map_notify(&self, ev: &xcb::MapNotifyEvent) {
        if let Some(w) = self.window(ev.event()) {
            event_fire!(w.base.borrow().on_show.clone(), make_window(w.rc_me()));
        }
    }

    fn handle_unmap_notify(&self, ev: &xcb::UnmapNotifyEvent) {
        if let Some(w) = self.window(ev.event()) {
            event_fire!(w.base.borrow().on_hide.clone(), make_window(w.rc_me()));
        }
    }
}


impl Platform for XcbPlatform {
    fn create_window(&self, base: RcCell<WindowBase>) -> Rc<PlatformWindow> {
        XcbWindow::new(base, self.shared_state.clone())
    }
}

impl EventLoop for XcbPlatform {
    fn loop_events(&self) -> i32 {
        while let Some(ev) = self.conn().wait_for_event() {
            let r = ev.response_type() & !0x80;
            match r {
                xcb::CLIENT_MESSAGE => {
                    self.handle_client_message(xcb::cast_event(&ev));
                },
                xcb::CONFIGURE_NOTIFY => {
                    self.handle_configure_notify(xcb::cast_event(&ev));
                },
                xcb::MAP_NOTIFY => {
                    self.handle_map_notify(xcb::cast_event(&ev));
                },
                xcb::UNMAP_NOTIFY => {
                    self.handle_unmap_notify(xcb::cast_event(&ev));
                },
                _ => {}
            }
            if self.exit_code.get().is_some() { break; }
        }
        // 2 ways to exit event loop:
        //  - setting exit_code
        //  - error (wait_for_event returns None)
        self.exit_code.get().expect(
            "XCB event loop was exited abruptly"
        )
    }
    fn exit(&self, code: i32) {
        self.exit_code.set(Some(code));
    }
}



fn make_window(pw: Rc<XcbWindow>) -> Window {
    let base = pw.base.clone();
    Window::make(base, pw)
}


pub struct XcbWindow {
    base: RcCell<WindowBase>,
    weak_me: RefCell<Weak<XcbWindow>>,
    shared_state: Rc<XcbSharedState>,
    xcb_win: Cell<xcb::Window>,
    rect: Cell<IRect>,
    last_known_state: Cell<window::State>,
    created: Cell<bool>,
}


impl XcbWindow {
    fn new(base: RcCell<WindowBase>, shared_state: Rc<XcbSharedState>) -> Rc<XcbWindow> {
        let w = Rc::new(XcbWindow {
            base: base,
            weak_me: RefCell::new(Weak::new()),
            shared_state: shared_state,
            xcb_win: Cell::new(0),
            rect: Cell::new(IRect::new(0, 0, 0, 0)),
            last_known_state: Cell::new(window::State::Hidden),
            created: Cell::new(false),
        });
        (*w.weak_me.borrow_mut()) = Rc::downgrade(&w);
        w
    }

    fn rc_me(&self) -> Rc<XcbWindow> {
        // self is not dropped, so unwrapping should be safe
        Weak::upgrade(&self.weak_me.borrow()).unwrap()
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


    fn created(&self) -> bool { self.created.get() }

    fn handle_configure_notify(&self, ev: &xcb::ConfigureNotifyEvent) {
        debug_assert!(self.created());
        let old_r = self.rect.get();

        // cannot trust ev.x and ev.y as they are not updated
        // when the window is resized by top or left.
        // we fetch the position directly from the server instead
        let new_pos = self.get_position_sys().unwrap_or(
            IPoint::new(ev.x() as i32, ev.y() as i32)
        );
        if new_pos != old_r.point() {
            event_fire!(self.base.borrow().on_move.clone(),
                make_window(self.rc_me()),
                new_pos);
        }

        let new_size = ISize::new(ev.width() as i32, ev.height() as i32);
        if new_size != old_r.size() {
            event_fire!(self.base.borrow().on_resize.clone(),
                make_window(self.rc_me()),
                new_size);
        }

        self.rect.set(IRect::new_ps(new_pos, new_size));
    }

    fn get_position_sys(&self) -> Option<IPoint> {
        debug_assert!(self.created());

        let cookie = xcb::translate_coordinates(self.conn(),
            self.xcb_win.get(), self.def_screen_root(), 0, 0);
        cookie.get_reply().ok().map(|r| {
            IPoint::new(r.dst_x() as i32, r.dst_y() as i32)
        })
    }

    fn get_state_sys(&self) -> window::State {
        if !self.created() {
            window::State::Hidden
        }
        else {
            let wm_state_atom = self.atom(Atom::WM_STATE);

            // checking for minimized
            let cookie = xcb::get_property_unchecked(self.conn(), false,
                    self.xcb_win.get(), wm_state_atom, xcb::ATOM_ANY,
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

    fn get_wm_states(&self) -> u32 {
        debug_assert!(self.created());

        let cookie = xcb::get_property(self.conn(), false,
                self.xcb_win.get(), self.atom(Atom::_NET_WM_STATE),
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
                32, self.xcb_win.get(), self.atom(Atom::_NET_WM_STATE),
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

impl PlatformWindow for XcbWindow {

    fn create(&self) {
        if self.created() { return; }

        let xcb_win = self.shared_state.conn.generate_id();

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
                xcb_win, screen.root(), p.x as i16, p.y as i16,
                s.w as u16, s.h as u16, 0, xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
                screen.root_visual(), &values);

            let wm_delete_window = self.atom(Atom::WM_DELETE_WINDOW);
            let wm_protocols = self.atom(Atom::WM_PROTOCOLS);

            let values = [wm_delete_window];
            let dc = xcb::change_property(self.conn(), xcb::PROP_MODE_REPLACE as u8,
                    xcb_win, wm_protocols, xcb::ATOM_ATOM, 32, &values);

            assert!(wc.request_check().is_ok(), "could not create xcb window");
            assert!(dc.request_check().is_ok(), "could not prepare xcb window deletion handling");
        }

        // setup borrows self, so assignments must be after setup's lifetime
        self.last_known_state.set(window::State::Hidden);
        self.rect.set(IRect::new_ps(p, s));
        self.xcb_win.set(xcb_win);
        {
            let mut windows = self.shared_state.windows.borrow_mut();
            windows.insert(self.xcb_win.get(), self.weak_me.borrow().clone());
        }
        self.created.set(true);
        self.update_title();
        self.update_state();
    }

    fn check_base(&self, base: &WindowBase) -> bool {
        let mine = &(*self.base.borrow()) as *const WindowBase;
        let foreign = base as *const WindowBase;
        mine == foreign
    }

    fn update_title(&self) {
        if self.created() {
            let base = self.base.borrow();
            let title = &base.title;
            xcb::change_property(self.conn(), xcb::PROP_MODE_REPLACE as u8,
                self.xcb_win.get(), xcb::ATOM_WM_NAME, xcb::ATOM_STRING, 8, title.as_bytes());
            xcb::change_property(self.conn(), xcb::PROP_MODE_REPLACE as u8,
                self.xcb_win.get(), xcb::ATOM_WM_ICON_NAME, xcb::ATOM_STRING, 8, title.as_bytes());
        }
    }


    fn update_state(&self) {

        if !self.created() { return; }

        let new_state = self.base.borrow().state;
        let old_state = self.last_known_state.get();

        if new_state == old_state { return; }


        // removing attribute that makes other than normal
        match old_state {
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
                xcb::map_window(self.conn(), self.xcb_win.get());
            },
            _ => {}
        }

        // at this point the window is in normal mode
        match new_state {
            window::State::Minimized => {
                let ev = xcb::ClientMessageEvent::new(32, self.xcb_win.get(),
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
                xcb::unmap_window(self.conn(), self.xcb_win.get());
            },
            _ => {}
        }
        self.conn().flush();
    }


    fn close(&self) {
        if self.created() {
            if self.last_known_state.get() != window::State::Hidden {
                xcb::unmap_window(self.conn(), self.xcb_win.get());
            }
            xcb::destroy_window(self.conn(), self.xcb_win.get());
            self.created.set(false);
            {
                let mut windows = self.shared_state.windows.borrow_mut();
                windows.remove(&self.xcb_win.get());
            }
            self.xcb_win.set(0);
            self.conn().flush();
        }
    }
}

impl Drop for XcbWindow {
    fn drop(&mut self) {
        self.close();
    }
}
