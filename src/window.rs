
use geometry::ISize;


#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum State {
    Normal,
    Minimized,
    Maximized,
    Fullscreen,
    Hidden,
}

pub trait Window {

    fn base(&self) -> &WindowBase;
    fn base_mut(&mut self) -> &mut WindowBase;

    fn title(&self) -> String;
    fn set_title(&mut self, tit: String);

    fn show_normal(&mut self) {
        self.set_state(State::Normal);
    }

    fn show_minimized(&mut self) {
        self.set_state(State::Minimized);
    }

    fn show_maximized(&mut self) {
        self.set_state(State::Maximized);
    }

    fn show_fullscreen(&mut self) {
        self.set_state(State::Fullscreen);
    }

    fn hide(&mut self) {
        self.set_state(State::Hidden);
    }

    fn state(&self) -> State;
    fn set_state(&mut self, state: State);


    fn on_close_do(&mut self, handler: Box<FnMut() -> bool>) {
        self.on_close_mut().set(Some(handler));
    }
    fn on_close(&self) -> &OnCloseHandler {
        &self.base().on_close
    }
    fn on_close_mut(&mut self) -> &mut OnCloseHandler {
        &mut self.base_mut().on_close
    }

}


define_handler!{OnCloseHandler () => bool}
define_handler!{OnResizeHandler (new_size: ISize)}


pub struct WindowBase {
    on_close: OnCloseHandler,
    on_resize: OnResizeHandler,
}

impl WindowBase {
    pub fn new() -> WindowBase {
        WindowBase {
            on_close: OnCloseHandler::new(),
            on_resize: OnResizeHandler::new(),
        }
    }
}
