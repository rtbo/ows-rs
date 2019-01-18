
use super::{WldShared, Display};
use crate::window;
use crate::display;
use wlc::protocol::wl_compositor::RequestsTrait as CompReqs;
use wlc::protocol::wl_surface::WlSurface;
use wlp::xdg_shell::client::xdg_wm_base::RequestsTrait as XdgWmReqs;
use wlp::xdg_shell::client::xdg_surface::{self, XdgSurface, RequestsTrait as XdgSurfReqs};
use wlp::xdg_shell::client::xdg_toplevel::{self, XdgToplevel, RequestsTrait as XdgTlReqs};
use std::cell::RefCell;
use std::rc::Rc;


pub struct WldWindow
{
    shared: Rc<RefCell<WldShared>>,
    win_shared: Rc<RefCell<WldWinShared>>,
    title: String,
}

struct WldWinShared
{
    _surf: Option<wlc::Proxy<WlSurface>>,
    _xdg_surf: Option<wlc::Proxy<XdgSurface>>,
    xdg_tl: Option<wlc::Proxy<XdgToplevel>>,
    size: (i32, i32),
}

impl WldWindow
{
    pub(in super) fn new(shared: Rc<RefCell<WldShared>>) -> WldWindow {
        WldWindow {
            shared: shared.clone(),
            win_shared: Rc::new(RefCell::new(WldWinShared {
                _surf: None, _xdg_surf: None, xdg_tl: None,
                size: (0, 0)
            })),
            title: String::new(),
        }
    }
}

impl display::Window<Display> for WldWindow
{
    fn title(&self) -> &str {
        &self.title
    }

    fn set_title(&mut self, val: String)
    {
        match &self.win_shared.borrow_mut().xdg_tl {
            Some(tl) => tl.set_title(val.clone()),
            _ => {}
        }
        self.title = val;
    }

    fn show (&mut self, _: window::State)
    {
        let shared = self.shared.borrow();

        let surf = shared.compositor.create_surface(
            |np| np.implement(|_, _| {}, ())
        ).unwrap();

        let xdg_surf = shared.xdg_shell.get_xdg_surface(
            &surf, |np| np.implement(|ev, xdg_surf| {
                match ev {
                    xdg_surface::Event::Configure{serial} => xdg_surf.ack_configure(serial)
                }
            }, ())
        ).unwrap();

        let ws = self.win_shared.clone();
        let xdg_tl = unsafe { xdg_surf.get_toplevel(
            |np| np.implement_nonsend(move |ev, _| {
                match ev {
                    xdg_toplevel::Event::Configure{width, height, states: _} => {
                        ws.borrow_mut().size = (width, height);
                    },
                    xdg_toplevel::Event::Close => {

                    }
                }
            }, (), &shared.queue.get_token())
        ).unwrap() };

        xdg_tl.set_title(self.title.clone());

        let mut ws = self.win_shared.borrow_mut();
        ws._surf = Some(surf);
        ws._xdg_surf = Some(xdg_surf);
        ws.xdg_tl = Some(xdg_tl);
    }

    fn close(&mut self) {}

    fn retrieve_events(&mut self) -> Vec<window::Event> {
        Vec::new()
    }
}
