
use crate::window;
use crate::display;
use super::WldState;
use std::cell::RefCell;
use std::rc::{Rc, Weak};


pub struct WldWindow
{
    dpy: Weak<RefCell<WldState>>,
    title: String,
    size: (i32, i32),
}

impl WldWindow
{
    pub(in super) fn new(dpy: Rc<RefCell<WldState>>) -> Rc<RefCell<WldWindow>> {
        Rc::new(RefCell::new( WldWindow {
            dpy: Rc::downgrade(&dpy),
            title: String::new(),
            size: (0, 0)
        } ))
    }
}

impl display::Window for WldWindow
{
    fn title(&self) -> &str {
        &self.title
    }
    fn set_title(&mut self, val: String) {

    }

    fn show (&mut self, state: window::State) {}

    fn close(&mut self) {}
}
