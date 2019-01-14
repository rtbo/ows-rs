
use super::{WldShared, Display};
use crate::window;
use crate::display;
use wlc::protocol::wl_compositor::RequestsTrait as CompRequests;
use wlc::protocol::wl_surface::WlSurface;
use wlp::xdg_shell::client::xdg_wm_base::RequestsTrait as XdgWmRequests;
use wlp::xdg_shell::client::xdg_surface::XdgSurface;
use std::cell::RefCell;
use std::rc::Rc;


pub struct WldWindow
{
    shared: Rc<RefCell<WldShared>>,
    win_shared: Option<WldWinShared>,
    title: String,
    _size: (i32, i32),
}

struct WldWinShared
{
    _surf: wlc::Proxy<WlSurface>,
    _xdg_surf: wlc::Proxy<XdgSurface>,
}

impl WldWindow
{
    pub(in super) fn new(shared: Rc<RefCell<WldShared>>) -> WldWindow {
        WldWindow {
            shared: shared.clone(),
            win_shared: None,
            title: String::new(),
            _size: (0, 0)
        }
    }
}

impl display::Window<Display> for WldWindow
{
    fn title(&self) -> &str {
        &self.title
    }
    fn set_title(&mut self, val: String) {
        self.title = val;
    }

    fn show (&mut self, _: window::State)
    {
        let shared = self.shared.borrow();

        let surf = shared.compositor.create_surface(
            |np| np.implement(|_, _| {}, ())
        ).unwrap();
        let xdg_surf = shared.xdg_shell.get_xdg_surface(
            &surf, |np| np.implement(|_, _| {}, ())
        ).unwrap();

        self.win_shared = Some(WldWinShared {
            _surf: surf, _xdg_surf: xdg_surf
        });
    }

    fn close(&mut self) {}
}
