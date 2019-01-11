
use crate::window;
use super::DisplayState;
use std::cell::RefCell;
use std::rc::{Rc, Weak};


pub struct Window
{
    state: Rc<RefCell<WindowState>>
}

impl Window
{
    pub(in super) fn new(dpy: Rc<RefCell<DisplayState>>) -> Window {
        Window {
            state: Rc::new(RefCell::new( WindowState {
                dpy: Rc::downgrade(&dpy),
                title: String::new(),
                size: (0, 0)
            } ))
        }
    }
}

struct WindowState
{
    dpy: Weak<RefCell<DisplayState>>,
    title: String,
    size: (i32, i32),
}

impl window::Window for Window
{
    fn title(&self) -> String {
        self.state.borrow().title.clone()
    }
    fn set_title(&mut self, val: String) {
        self.state.borrow_mut().title = val;
    }

    fn show (&mut self, state: window::State) {}

    fn close(&mut self) {}
}
