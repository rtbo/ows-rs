use crate::geom::{IPoint, ISize};
use crate::gfx;
use crate::key;
use crate::mouse;
use crate::window;
use std::cell::{Cell, RefCell};
use std::mem;
use std::rc::Rc;
use std::sync::Arc;
use wlc::protocol::wl_compositor::{RequestsTrait as CompReqs, WlCompositor};
use wlc::protocol::wl_keyboard::{self, WlKeyboard};
use wlc::protocol::wl_pointer::{self, WlPointer};
use wlc::protocol::wl_seat::{self, RequestsTrait as SeatReqs, WlSeat};
use wlc::protocol::wl_surface::{RequestsTrait as SurfReqs, WlSurface};
use wlc::{ConnectError, EventQueue, GlobalManager, Proxy};
use wlp::xdg_shell::client::xdg_surface::{self, RequestsTrait as XdgSurfReqs, XdgSurface};
use wlp::xdg_shell::client::xdg_toplevel::{self, RequestsTrait as XdgTlReqs, XdgToplevel};
use wlp::xdg_shell::client::xdg_wm_base::{self, RequestsTrait as XdgReqs, XdgWmBase};

pub struct Display {
    queue: RefCell<EventQueue>,
    inner: Rc<DispInner>,
}

pub struct DispInner {
    dpy: wlc::Display,
    queue_token: wlc::QueueToken,
    compositor: Proxy<WlCompositor>,
    xdg_shell: Proxy<XdgWmBase>,
    pointer: RefCell<Option<Proxy<WlPointer>>>,
    keyboard: RefCell<Option<Proxy<WlKeyboard>>>,
    key_mods: Cell<key::Mods>,
    mouse_state: Cell<mouse::State>,
    instance: Arc<gfx_back::Instance>,
    windows: RefCell<Vec<Rc<WindowInner>>>,
    pointed_window: RefCell<Option<Rc<WindowInner>>>,
}

impl Drop for Display {
    fn drop(&mut self) {}
}

impl super::Display for Display {
    type Window = Window;
    type OpenError = ConnectError;

    fn open() -> Result<Display, ConnectError> {
        let (dpy, mut queue) = wlc::Display::connect_to_env()?;

        let queue_token = queue.get_token();

        let dpy_wrapper = dpy.make_wrapper(&queue_token).unwrap();
        let glob_manager = GlobalManager::new(&dpy_wrapper);

        queue.sync_roundtrip().unwrap();
        queue.sync_roundtrip().unwrap();

        let compositor = glob_manager
            .instantiate_auto(|comp| comp.implement(|_, _| {}, ()))
            .expect("wl_compositor not advertized by Compositor");

        let xdg_shell = glob_manager
            .instantiate_auto::<XdgWmBase, _>(|np| {
                np.implement(
                    |ev, xdg_shell| match ev {
                        xdg_wm_base::Event::Ping { serial } => xdg_shell.pong(serial),
                    },
                    (),
                )
            })
            .expect("xdg_shell protocol not supported by Compositor");

        let inner = Rc::new(DispInner {
            dpy,
            queue_token,
            compositor,
            xdg_shell,
            pointer: RefCell::new(None),
            keyboard: RefCell::new(None),
            key_mods: Cell::new(key::Mods::empty()),
            mouse_state: Cell::new(mouse::State::empty()),
            instance: Arc::new(gfx_back::Instance::create("ows-rs wayland", 0)),
            windows: RefCell::new(Vec::new()),
            pointed_window: RefCell::new(None),
        });

        let queue_token = queue.get_token();
        let inner2 = inner.clone();
        let _seat = glob_manager.instantiate_auto::<WlSeat, _>(|seat| unsafe {
            seat.implement_nonsend(
                move |ev, seat| match ev {
                    wl_seat::Event::Capabilities { capabilities } => {
                        seat_caps_event(&inner2, &seat, capabilities);
                    }
                    _ => {}
                },
                (),
                &queue_token,
            )
        });

        Ok(Display {
            queue: RefCell::new(queue),
            inner,
        })
    }

    fn create_window(&self) -> Window {
        let w = Window::new(self.inner.clone());
        self.inner.windows.borrow_mut().push(w.inner.clone());
        w
    }

    fn collect_events(&self) {
        self.inner.dpy.flush().unwrap();
        self.queue
            .borrow_mut()
            .dispatch()
            .expect("Error occured during Wayland queue dispatch");
    }

    fn instance(&self) -> Arc<gfx_back::Instance> {
        self.inner.instance.clone()
    }
}

impl DispInner {
    fn find_window(&self, surf: &Proxy<WlSurface>) -> Option<Rc<WindowInner>> {
        for w in &*self.windows.borrow() {
            if w.surf.equals(&surf) {
                return Some(w.clone());
            }
        }
        None
    }
}

fn seat_caps_event(disp: &Rc<DispInner>, seat: &Proxy<WlSeat>, caps: wl_seat::Capability) {
    let mut pointer = disp.pointer.borrow_mut();
    let mut keyboard = disp.keyboard.borrow_mut();
    if caps.contains(wl_seat::Capability::Pointer) && pointer.is_none() {
        let disp2 = disp.clone();
        *pointer = Some(
            seat.get_pointer(|pointer| unsafe {
                pointer.implement_nonsend(
                    move |ev, _| {
                        pointer_event(&disp2, ev);
                    },
                    (),
                    &disp.queue_token,
                )
            })
            .unwrap(),
        );
    }
    if !caps.contains(wl_seat::Capability::Pointer) && pointer.is_some() {
        *pointer = None;
    }
    if caps.contains(wl_seat::Capability::Keyboard) && keyboard.is_none() {
        let disp2 = disp.clone();
        *keyboard = Some(
            seat.get_keyboard(|kbd| unsafe {
                kbd.implement_nonsend(
                    move |ev, _| {
                        keyboard_event(&disp2, ev);
                    },
                    (),
                    &disp.queue_token,
                )
            })
            .unwrap(),
        );
    }
}

fn pointer_event(disp: &Rc<DispInner>, ev: wl_pointer::Event) {
    match ev {
        wl_pointer::Event::Enter {
            serial: _,
            surface,
            surface_x,
            surface_y,
        } => {
            // TODO: set cursor
            let w = disp.find_window(&surface).expect("Could not find window"); // TODO warn
            let pos = IPoint::new(surface_x as _, surface_y as _);
            let state = disp.mouse_state.get();
            let mods = disp.key_mods.get();
            w.event_buf
                .borrow_mut()
                .push(window::Event::MouseEnter(pos, state, mods));
            w.curs_pos.set(pos);
            *disp.pointed_window.borrow_mut() = Some(w.clone());
        }
        wl_pointer::Event::Leave { serial: _, surface } => {
            let w = disp.find_window(&surface).expect("Could not find window"); // TODO warn
            let pos = w.curs_pos.get();
            let state = disp.mouse_state.get();
            let mods = disp.key_mods.get();
            w.event_buf
                .borrow_mut()
                .push(window::Event::MouseLeave(pos, state, mods));
            *disp.pointed_window.borrow_mut() = None;
        }
        wl_pointer::Event::Motion {
            time: _,
            surface_x,
            surface_y,
        } => {
            if let Some(w) = disp.pointed_window.borrow().as_ref() {
                let pos = IPoint::new(surface_x as _, surface_y as _);
                let state = disp.mouse_state.get();
                let mods = disp.key_mods.get();
                w.event_buf
                    .borrow_mut()
                    .push(window::Event::MouseMove(pos, state, mods));
                w.curs_pos.set(pos);
            }
        }
        wl_pointer::Event::Button {
            serial: _,
            time: _,
            button,
            state,
        } => {
            let (but, state_but) = match button {
                0x110 => (mouse::But::Left, mouse::State::LEFT),
                0x111 => (mouse::But::Right, mouse::State::RIGHT),
                0x112 => (mouse::But::Middle, mouse::State::MIDDLE),
                _ => {
                    println!("unexpected wayland button: {}", button);
                    return;
                }
            };
            let mut mouse_state = disp.mouse_state.get();
            match state {
                wl_pointer::ButtonState::Pressed => {
                    mouse_state.insert(state_but);
                }
                wl_pointer::ButtonState::Released => {
                    mouse_state.remove(state_but);
                }
            }
            disp.mouse_state.set(mouse_state);
            if let Some(w) = disp.pointed_window.borrow().as_ref() {
                let pos = w.curs_pos.get();
                let mods = disp.key_mods.get();
                let ev = match state {
                    wl_pointer::ButtonState::Pressed => {
                        window::Event::MouseDown(pos, but, mouse_state, mods)
                    }
                    wl_pointer::ButtonState::Released => {
                        window::Event::MouseUp(pos, but, mouse_state, mods)
                    }
                };
                w.event_buf.borrow_mut().push(ev);
            }
        }
        _ => {}
    }
}

fn keyboard_event(_disp: &Rc<DispInner>, _ev: wl_keyboard::Event) {}

pub struct Window {
    inner: Rc<WindowInner>,
    disp_inner: Rc<DispInner>,
    surf: Proxy<WlSurface>,
    xdg_surf: Proxy<XdgSurface>,
    xdg_tl: Proxy<XdgToplevel>,

    title: String,
}

struct WindowInner {
    surf: Proxy<WlSurface>,
    event_buf: RefCell<Vec<window::Event>>,
    size: Cell<ISize>,
    curs_pos: Cell<IPoint>,
}

impl Window {
    fn new(disp_inner: Rc<DispInner>) -> Window {
        let compositor = &disp_inner.compositor;
        let xdg_shell = &disp_inner.xdg_shell;
        let queue_token = &disp_inner.queue_token;

        let surf = compositor
            .create_surface(|surf| surf.implement(|_, _| {}, ()))
            .expect("could not create surface");

        let xdg_surf = xdg_shell
            .get_xdg_surface(&surf, |xdg| {
                xdg.implement(
                    |ev, xdg| match ev {
                        xdg_surface::Event::Configure { serial } => {
                            xdg.ack_configure(serial);
                        }
                    },
                    (),
                )
            })
            .expect("could not create XDG surface");

        let inner = Rc::new(WindowInner {
            surf: surf.clone(),
            event_buf: RefCell::new(Vec::new()),
            size: Cell::new(ISize::new(0, 0)),
            curs_pos: Cell::new(IPoint::new(0, 0)),
        });

        let inner2 = inner.clone();
        let xdg_tl = xdg_surf
            .get_toplevel(|tl| unsafe {
                tl.implement_nonsend(
                    move |ev, _| match ev {
                        xdg_toplevel::Event::Configure {
                            width,
                            height,
                            states,
                        } => {
                            let states = cast_vec_check_cap(states);
                            inner2.configure_event(ISize::new(width, height), states);
                        }
                        xdg_toplevel::Event::Close => {
                            inner2.close_event();
                        }
                    },
                    (),
                    queue_token,
                )
            })
            .expect("could not get XDG toplevel");

        // set reasonable min size
        xdg_tl.set_min_size(10, 10);

        Window {
            inner,
            disp_inner,
            surf,
            xdg_surf,
            xdg_tl,
            title: String::new(),
        }
    }
}

impl window::Window<Display> for Window {
    fn title(&self) -> &str {
        &self.title
    }
    fn set_title(&mut self, val: String) {
        self.title = val.clone();
        self.xdg_tl.set_title(val);
    }

    fn size(&self) -> ISize {
        self.inner.size.get()
    }

    fn show(&mut self, state: window::State) {
        self.surf.commit();
        match state {
            window::State::Normal(sz) => {
                let sz = sz.unwrap_or((640, 480));
                let sz = ISize::new(sz.0 as _, sz.1 as _);
                self.inner.size.set(sz);
                self.inner
                    .event_buf
                    .borrow_mut()
                    .push(window::Event::Resize(sz));
            }
            window::State::Maximized => {
                self.xdg_tl.set_maximized();
            }
            window::State::Fullscreen => {
                unimplemented!();
            }
            window::State::Minimized => {
                panic!("should not set to minimized on first show");
            }
        }
    }

    fn retrieve_events(&mut self) -> Vec<window::Event> {
        let mut evs = Vec::new();
        mem::swap(&mut evs, &mut *self.inner.event_buf.borrow_mut());
        evs
    }

    fn token(&self) -> window::Token {
        window::Token::new(self.surf.c_ptr() as _)
    }

    fn create_surface(&self) -> gfx::Surface {
        let sz = self.inner.size();
        self.disp_inner.instance.create_surface_from_wayland(
            self.disp_inner.dpy.c_ptr() as _,
            self.surf.c_ptr() as _,
            sz.w as _,
            sz.h as _,
        )
    }

    fn close(&mut self) {
        self.xdg_tl.destroy();
        self.xdg_surf.destroy();
        self.surf.destroy();
    }
}

impl WindowInner {
    #[inline]
    fn size(&self) -> ISize {
        self.size.get()
    }
    fn configure_event(&self, size: ISize, states: Vec<xdg_toplevel::State>) {
        println!("wayland xdg configure {:?}", states);
        if size != self.size() && size.w != 0 && size.h != 0 {
            self.size.set(size);
            self.event_buf
                .borrow_mut()
                .push(window::Event::Resize(size));
        }
    }
    fn close_event(&self) {
        self.event_buf.borrow_mut().push(window::Event::Close);
    }
}

unsafe fn cast_vec_check_cap<T: Copy>(vec: Vec<u8>) -> Vec<T> {
    if vec.capacity() % mem::size_of::<T>() == 0 {
        cast_vec(vec)
    } else {
        let slice = cast_slice(&vec);
        let mut vec = Vec::with_capacity(slice.len());
        vec.copy_from_slice(slice);
        vec
    }
}

unsafe fn cast_slice<T>(slice: &[u8]) -> &[T] {
    assert!(slice.len() % mem::size_of::<T>() == 0);
    let ptr = slice.as_ptr() as *const T;
    let len = slice.len() / mem::size_of::<T>();
    std::slice::from_raw_parts(ptr, len)
}

unsafe fn cast_vec<T>(mut vec: Vec<u8>) -> Vec<T> {
    let tsize = mem::size_of::<T>();
    assert!(vec.len() % tsize == 0);
    assert!(vec.capacity() % tsize == 0);
    let p = vec.as_mut_ptr();
    let len = vec.len() / tsize;
    let cap = vec.capacity() / tsize;

    mem::forget(vec);

    Vec::from_raw_parts(p as _, len, cap)
}
