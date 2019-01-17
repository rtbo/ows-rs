use super::geometry::{IPoint, ISize};
use super::key;
use super::mouse;
use crate::display::Display;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum State {
    Normal(Option<(u16, u16)>),
    Maximized,
    Minimized,
    Fullscreen,
}

pub trait Window<D: Display> {
    fn title(&self) -> &str;
    fn set_title(&mut self, val: String);

    fn show(&mut self, state: State);

    fn close(&mut self);

    fn retrieve_events(&mut self) -> Vec<Event>;
}

#[derive(Clone, Debug)]
pub enum Event {
    Resize(ISize),
    Close,
    State(State),
    MouseEnter(IPoint, mouse::State, key::Mods),
    MouseLeave(IPoint, mouse::State, key::Mods),
    MouseMove(IPoint, mouse::State, key::Mods),
    MouseDown(IPoint, mouse::But, mouse::State, key::Mods),
    MouseUp(IPoint, mouse::But, mouse::State, key::Mods),
    KeyDown(key::Sym, key::Code, key::Mods, String),
    KeyUp(key::Sym, key::Code, key::Mods),
}
