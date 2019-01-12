
mod window;

use self::window::WldWindow;

pub use wlc::ConnectError;
use wlc::GlobalManager;
use std::cell::RefCell;
use std::rc::Rc;


pub struct Display
{
    state: Rc<RefCell<WldState>>,
}

impl Display
{
    pub fn open() -> Result<Display, ConnectError>
    {
        wlc::Display::connect_to_env()
            .map(|(dpy, queue)| Display {
                state: WldState::new(dpy, queue)
            })
    }
}

impl Drop for Display
{
    fn drop(&mut self) {}
}

impl super::Display for Display
{
    type Window = WldWindow;

    fn create_window(&mut self) -> Rc<RefCell<WldWindow>>
    {
        WldWindow::new(self.state.clone())
    }
}


struct WldState
{
    dpy: wlc::Display,
    queue: wlc::EventQueue,
}

impl WldState
{
    fn new (dpy: wlc::Display, mut queue: wlc::EventQueue) -> Rc<RefCell<Self>>
    {
        let globals = GlobalManager::new(&dpy);
        queue.sync_roundtrip().unwrap();

        for (id, interface, version) in globals.list() {
            println!("{}: {} (version = {})", id, interface, version);
        }

        Rc::new(RefCell::new(Self{
            dpy: dpy, queue: queue
        }))
    }
}
