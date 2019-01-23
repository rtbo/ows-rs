use crate::geom::ISize;
use crate::gfx;
use crate::window;
use std::cell::{Cell, RefCell};
use std::mem;
use std::rc::Rc;
use std::sync::Arc;
use wlc::protocol::wl_compositor::{RequestsTrait as CompReqs, WlCompositor};
use wlc::protocol::wl_surface::{RequestsTrait as SurfReqs, WlSurface};
use wlc::{ConnectError, EventQueue, GlobalManager, Proxy};
use wlp::xdg_shell::client::xdg_surface::{self, RequestsTrait as XdgSurfReqs, XdgSurface};
use wlp::xdg_shell::client::xdg_toplevel::{self, RequestsTrait as XdgTlReqs, XdgToplevel};
use wlp::xdg_shell::client::xdg_wm_base::{self, RequestsTrait as XdgReqs, XdgWmBase};

pub struct Display {
    queue: RefCell<EventQueue>,
    glob_manager: GlobalManager,
    inner: Rc<DispInner>,
}

pub struct DispInner {
    dpy: wlc::Display,
    queue_token: wlc::QueueToken,
    compositor: Proxy<WlCompositor>,
    xdg_shell: Proxy<XdgWmBase>,
    instance: Arc<gfx_back::Instance>,
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
        let glob_manager = GlobalManager::new_with_cb(&dpy_wrapper, move |ev, reg| {});

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
            instance: Arc::new(gfx_back::Instance::create("ows-rs wayland", 0)),
        });

        Ok(Display {
            queue: RefCell::new(queue),
            glob_manager,
            inner,
        })
    }

    fn create_window(&self) -> Window {
        Window::new(self.inner.clone())
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

pub struct Window {
    inner: Rc<WindowInner>,
    disp_inner: Rc<DispInner>,
    surf: Proxy<WlSurface>,
    xdg_surf: Proxy<XdgSurface>,
    xdg_tl: Proxy<XdgToplevel>,

    title: String,
}

struct WindowInner {
    event_buf: RefCell<Vec<window::Event>>,
    size: Cell<ISize>,
}

impl Window {
    fn new(disp_inner: Rc<DispInner>) -> Window {
        let inner = Rc::new(WindowInner {
            event_buf: RefCell::new(Vec::new()),
            size: Cell::new(ISize { w: 0, h: 0 }),
        });

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

        let inner2 = inner.clone();
        let xdg_tl = xdg_surf
            .get_toplevel(|tl| unsafe {
                tl.implement_nonsend(
                    move |ev, tl| match ev {
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

    /// Creates a gfx::Surface to render on the window.
    /// This may panic if show was not called before (to be revised)
    fn create_surface(&self) -> gfx::Surface {
        self.disp_inner.instance.create_surface_from_wayland(
            self.disp_inner.dpy.c_ptr() as _,
            self.surf.c_ptr() as _,
            640,
            480,
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
