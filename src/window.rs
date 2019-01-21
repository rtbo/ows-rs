use super::geom::{IPoint, ISize};
use super::key;
use super::mouse;
use crate::display::Display;
use crate::gfx;

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

    fn size(&self) -> ISize;

    fn show(&mut self, state: State);

    fn retrieve_events(&mut self) -> Vec<Event>;

    fn token(&self) -> Token;

    /// Creates a gfx::Surface to render on the window.
    /// This may panic if show was not called before (to be revised)
    fn create_surface(&self) -> gfx::Surface;

    fn close(&mut self);
}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct Token(usize);

impl Token {
    pub(crate) fn new (tok: usize) -> Token {
        Token(tok)
    }
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
