
use crate::display::Display;
use super::geometry::{IPoint, ISize};
use super::key;
use super::mouse;

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

pub enum Event
{
    Resize(ISize),
    Close,
    State(State),
    MouseDown(IPoint, mouse::But, mouse::Buts, key::Mods),
    MouseUp(IPoint, mouse::But, mouse::Buts, key::Mods),
    MouseMove(IPoint, mouse::Buts, key::Mods),
    KeyDown(key::Sym, key::Code, key::Mods, String),
    KeyUp(key::Sym, key::Code, key::Mods)
}
