


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


    fn on_close_do(&mut self, sig: Option<Box<FnMut() -> bool>>) {
        self.on_close_mut().set(sig);
    }
    fn on_close(&self) -> &OnCloseSig {
        &self.base().on_close
    }
    fn on_close_mut(&mut self) -> &mut OnCloseSig {
        &mut self.base_mut().on_close
    }
}

pub struct OnCloseSig {
    sig: Option<Box<FnMut() -> bool>>,
}

impl OnCloseSig {
    pub fn new() -> OnCloseSig {
        OnCloseSig { sig: None }
    }
    pub fn is_set(&self) -> bool {
        self.sig.is_some()
    }
    pub fn set(&mut self, sig: Option<Box<FnMut() -> bool>>) {
        self.sig = sig;
    }
    pub fn fire(&mut self) -> bool {
        self.sig.as_mut().map(|sig| (*sig)()).unwrap()
    }
    pub fn fire_or(&mut self, def: bool) -> bool {
        self.sig.as_mut().map_or(def, |sig| (*sig)())
    }
}

pub struct WindowBase {
    on_close: OnCloseSig,
}

impl WindowBase {
    pub fn new() -> WindowBase {
        WindowBase {
            on_close: OnCloseSig::new()
        }
    }
}
