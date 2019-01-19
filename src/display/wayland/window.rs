use super::{Display, DisplayShared};
use crate::geometry::{ISize};
use crate::window;
use std::cell::RefCell;
use std::mem;
use std::rc::Rc;
use wlc::protocol::wl_compositor::RequestsTrait as CompReqs;
use wlc::protocol::wl_surface::{RequestsTrait as SurfReqs, WlSurface};
use wlp::xdg_shell::client::xdg_surface::{self, RequestsTrait as XdgSurfReqs, XdgSurface};
use wlp::xdg_shell::client::xdg_toplevel::{self, RequestsTrait as XdgTlReqs, XdgToplevel};
use wlp::xdg_shell::client::xdg_wm_base::RequestsTrait as XdgWmReqs;

pub struct Window {
    disp_shared: Rc<DisplayShared>,
    shared: Rc<RefCell<WindowShared>>,
    title: String,
}

struct WindowShared {
    _surf: Option<wlc::Proxy<WlSurface>>,
    _xdg_surf: Option<wlc::Proxy<XdgSurface>>,
    xdg_tl: Option<wlc::Proxy<XdgToplevel>>,
    event_buf: Vec<window::Event>,
    size: ISize,
}

impl Window {
    pub(super) fn new(shared: Rc<DisplayShared>) -> Window {
        Window {
            disp_shared: shared.clone(),
            shared: Rc::new(RefCell::new(WindowShared {
                _surf: None,
                _xdg_surf: None,
                xdg_tl: None,
                event_buf: Vec::new(),
                size: ISize::new(0, 0),
            })),
            title: String::new(),
        }
    }
}

impl window::Window<Display> for Window {
    fn title(&self) -> &str {
        &self.title
    }

    fn set_title(&mut self, val: String) {
        match &self.shared.borrow_mut().xdg_tl {
            Some(tl) => tl.set_title(val.clone()),
            _ => {}
        }
        self.title = val;
    }

    fn show(&mut self, _: window::State) {
        let surf = self.disp_shared
            .compositor
            .create_surface(|np| np.implement(|_, _| {}, ()))
            .unwrap();

        let xdg_surf = self.disp_shared
            .xdg_shell
            .get_xdg_surface(&surf, |np| {
                np.implement(
                    |ev, xdg_surf| match ev {
                        xdg_surface::Event::Configure { serial } => xdg_surf.ack_configure(serial),
                    },
                    (),
                )
            })
            .unwrap();

        let ws = self.shared.clone();
        let xdg_tl = unsafe {
            xdg_surf
                .get_toplevel(|np| {
                    np.implement_nonsend(
                        move |ev, _| match ev {
                            xdg_toplevel::Event::Configure { width, height, states, } => {
                                ws.borrow_mut().handle_configure(ISize::new(width, height), states);
                            }
                            xdg_toplevel::Event::Close => {
                                ws.borrow_mut().handle_close();
                            }
                        },
                        (),
                        &self.disp_shared.queue_token,
                    )
                })
                .unwrap()
        };

        xdg_tl.set_title(self.title.clone());

        surf.commit();

        let mut ws = self.shared.borrow_mut();
        ws._surf = Some(surf);
        ws._xdg_surf = Some(xdg_surf);
        ws.xdg_tl = Some(xdg_tl);
    }

    fn close(&mut self) {}

    fn retrieve_events(&mut self) -> Vec<window::Event> {
        let mut evs = Vec::new();
        let mut shared = self.shared.borrow_mut();
        mem::swap(&mut shared.event_buf, &mut evs);
        evs
    }
}

impl WindowShared {
    fn handle_configure(&mut self, size: ISize, states: Vec<u8>) {
        use wlp::xdg_shell::client::xdg_toplevel::State;
        let states: &[State] = unsafe { cast_slice(&states) };
        for state in states {
            println!("configure state: {:?}", state);
        }
        if size != self.size {
            self.size = size;
            self.event_buf.push(window::Event::Resize(size));
        }
    }
    fn handle_close(&mut self) {
        self.event_buf.push(window::Event::Close);
    }
}

unsafe fn cast_slice<T>(slice: &[u8]) -> &[T] {
    assert!(slice.len() % mem::size_of::<T>() == 0);
    let ptr = slice.as_ptr() as *const T;
    let len = slice.len() / mem::size_of::<T>();
    std::slice::from_raw_parts(ptr, len)
}
