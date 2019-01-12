
use crate::display;

use std::cell::{Ref, RefCell};
use std::rc::Rc;

pub enum State
{
    Normal(Option<(u16, u16)>),
    Maximized,
    Minimized,
    Fullscreen,
}

pub struct Window
{
    dpy: Rc<RefCell<display::Window>>
}

impl Window
{
    pub fn new<D>(dpy: &mut D) -> Window
        where D : display::Display,
              D::Window: 'static
    {
        Window {
            dpy: dpy.create_window()
        }
    }

    pub fn title(&self) -> Ref<str> {
        Ref::map(self.dpy.borrow(), | dpy | dpy.title() )
    }

    pub fn set_title(&mut self, val: String) {
        self.dpy.borrow_mut().set_title(val);
    }

    pub fn show(&mut self, state: State) {
        self.dpy.borrow_mut().show(state);
    }

    pub fn close(&mut self) {
        self.dpy.borrow_mut().close();
    }
}
