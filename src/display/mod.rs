
#![cfg(unix)]
pub mod wayland;

use crate::window::{self, Window};

pub trait Display : Drop
{
    type Window : Window;
    fn create_window(&mut self) -> Self::Window;
}
