
mod window;

use self::window::Window;

pub use wlc::ConnectError;
use wlc::GlobalManager;
use std::cell::RefCell;
use std::rc::Rc;


pub struct Display
{
    state: Rc<RefCell<DisplayState>>,
}

impl Display
{
    pub fn open() -> Result<Display, ConnectError>
    {
        wlc::Display::connect_to_env()
            .map(|(dpy, queue)| Display {
                state: DisplayState::new(dpy, queue)
            })
    }
}

impl Drop for Display
{
    fn drop(&mut self) {}
}

impl super::Display for Display
{
    type Window = Window;

    fn create_window(&mut self) -> Window
    {
        Window::new(self.state.clone())
    }
}


struct DisplayState
{
    dpy: wlc::Display,
    queue: wlc::EventQueue,
}

impl DisplayState
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
