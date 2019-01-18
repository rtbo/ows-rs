
mod window;

use self::window::WldWindow;

use wlc::ConnectError;
use wlc::protocol::wl_compositor::{WlCompositor};
use wlp::xdg_shell::client::xdg_wm_base::{self, XdgWmBase, RequestsTrait as XdgWmReqs};
use std::cell::RefCell;
use std::rc::Rc;

pub struct Display
{
    state: Rc<RefCell<WldShared>>,
}

impl Display
{
}

impl Drop for Display
{
    fn drop(&mut self) {}
}

impl super::Display for Display
{
    type Window = WldWindow;
    type OpenError = ConnectError;

    fn open() -> Result<Display, ConnectError>
    {
        wlc::Display::connect_to_env()
            .map(|(dpy, queue)| Display {
                state: WldShared::new(dpy, queue)
            })
    }

    fn create_window(&self) -> WldWindow
    {
        WldWindow::new(self.state.clone())
    }

    fn collect_events(&self) {}
}


struct WldShared
{
    _dpy: wlc::Display,
    queue: wlc::EventQueue,
    compositor: wlc::Proxy<WlCompositor>,
    xdg_shell: wlc::Proxy<XdgWmBase>,
}

impl WldShared
{
    fn new (dpy: wlc::Display, mut queue: wlc::EventQueue) -> Rc<RefCell<Self>>
    {
        use wlc::GlobalManager;

        let globals = GlobalManager::new(&dpy);
        queue.sync_roundtrip().unwrap();

        let compositor = globals.instantiate_auto::<WlCompositor, _>(
            |np| np.implement(|_, _| {}, ())
        ).unwrap();
        let xdg_shell = globals.instantiate_auto::<XdgWmBase, _>(
            |np| np.implement(|ev, xdg_shell| {
                match ev {
                    xdg_wm_base::Event::Ping{serial} => xdg_shell.pong(serial)
                }
            }, ())
        ).unwrap();

        for (id, interface, version) in globals.list() {
            println!("{}: {} (version = {})", id, interface, version);
        }

        Rc::new(RefCell::new(Self{
            _dpy: dpy, queue: queue,
            compositor: compositor,
            xdg_shell: xdg_shell,
        }))
    }
}
