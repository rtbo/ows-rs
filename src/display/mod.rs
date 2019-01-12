
#![cfg(unix)]
pub mod wayland;

use crate::window;

use std::cell::RefCell;
use std::rc::Rc;


pub trait Display : Drop
{
    type Window : Window;
    fn create_window(&mut self) -> Rc<RefCell<Self::Window>>;
}


pub trait Window
{
    fn title(&self) -> &str;
    fn set_title(&mut self, val: String);

    fn show (&mut self, state: window::State);

    fn close(&mut self);
}
