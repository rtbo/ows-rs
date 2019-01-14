
use crate::display::Display;

pub enum State
{
    Normal(Option<(u16, u16)>),
    Maximized,
    Minimized,
    Fullscreen,
}

pub trait Window<D : Display>
{
    fn title(&self) -> &str;
    fn set_title(&mut self, val: String);

    fn show (&mut self, state: State);

    fn close(&mut self);
}
