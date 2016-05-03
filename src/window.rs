
use geometry::{ISize, IPoint};
use platform::{Platform, PlatformWindow};
use ::RcCell;

use std::rc::Rc;
use std::cell::RefCell;


#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum State {
    Normal,
    Minimized,
    Maximized,
    Fullscreen,
    Hidden,
}

define_handler!{OnCloseHandler: FnMut(w: Window) => bool}
// generic event for events without parameters
define_event!{OnEvent: FnMut(w: Window)}
define_event!{OnResizeEvent: FnMut(w: Window, new_size: ISize)}
define_event!{OnMoveEvent: FnMut(w: Window, new_pos: IPoint)}



#[derive(Clone)]
pub struct WindowBase {
    pub title: String,
    pub state: State,
    pub on_close: RcCell<OnCloseHandler>,
    pub on_resize: RcCell<OnResizeEvent>,
    pub on_move: RcCell<OnMoveEvent>,
    pub on_show: RcCell<OnEvent>,
    pub on_hide: RcCell<OnEvent>,
}


pub struct Window {
    base: RcCell<WindowBase>,
    pw: Rc<PlatformWindow>,
}

impl Window {
    pub fn new() -> WindowBuilder {
        WindowBuilder {
            base: WindowBase {
                title: String::new(),
                state: State::Normal,
                on_close: Rc::new(RefCell::new(OnCloseHandler::new())),
                on_resize: Rc::new(RefCell::new(OnResizeEvent::new())),
                on_move: Rc::new(RefCell::new(OnMoveEvent::new())),
                on_show: Rc::new(RefCell::new(OnEvent::new())),
                on_hide: Rc::new(RefCell::new(OnEvent::new())),
            },
        }
    }

    /// Builds a Window out of already existing base and pw.
    /// This should only be used by PlatformWindow implementations.
    /// the base must fit the one referenced in the PlatformWindow
    pub fn make(base: RcCell<WindowBase>, pw: Rc<PlatformWindow>) -> Window {
        debug_assert!(pw.check_base(&base.borrow()));
        Window {
            base: base,
            pw: pw,
        }
    }

    pub fn title(&self) -> String {
        self.base.borrow().title.clone()
    }
    pub fn set_title(&mut self, title: String) {
        if title != self.base.borrow().title {
            self.base.borrow_mut().title = title;
            self.pw.update_title();
        }
    }


    pub fn state(&self) -> State {
        self.base.borrow().state
    }
    pub fn set_state(&mut self, state: State) {
        if state != self.base.borrow().state {
            self.base.borrow_mut().state = state;
            self.pw.update_state();
        }
    }
    pub fn show_normal(&mut self) {
        self.set_state(State::Normal);
    }
    pub fn show_minimized(&mut self) {
        self.set_state(State::Minimized);
    }
    pub fn show_maximized(&mut self) {
        self.set_state(State::Maximized);
    }
    pub fn show_fullscreen(&mut self) {
        self.set_state(State::Fullscreen);
    }
    pub fn hide(&mut self) {
        self.set_state(State::Hidden);
    }


    pub fn on_close(&self) -> RcCell<OnCloseHandler> {
        self.base.borrow().on_close.clone()
    }

    pub fn on_resize(&self) -> RcCell<OnResizeEvent> {
        self.base.borrow().on_resize.clone()
    }

    pub fn on_move(&self) -> RcCell<OnMoveEvent> {
        self.base.borrow().on_move.clone()
    }

    pub fn on_show(&self) -> RcCell<OnEvent> {
        self.base.borrow().on_show.clone()
    }

    pub fn on_hide(&self) -> RcCell<OnEvent> {
        self.base.borrow().on_hide.clone()
    }
}

impl Clone for Window {
    fn clone(&self) -> Window {
        Window {
            base: self.base.clone(),
            pw: self.pw.clone(),
        }
    }
}



pub struct WindowBuilder {
    base: WindowBase,
}


impl WindowBuilder {

    pub fn done(self, p: &Platform) -> Window {
        let base = Rc::new(RefCell::new(self.base));
        let pw = p.create_window(base.clone());
        pw.create();
        Window {
            base: base,
            pw: pw,
        }
    }

    pub fn title(mut self, title: String) -> WindowBuilder {
        self.base.title = title;
        self
    }

    pub fn state(mut self, state: State) -> WindowBuilder {
        self.base.state = state;
        self
    }

    pub fn on_close<F>(self, f: F) -> WindowBuilder
    where F: 'static + FnMut(Window) -> bool {
        handler_do!(self.base.on_close.clone(), f);
        self
    }

    pub fn on_resize<F>(self, f: F) -> WindowBuilder
    where F: 'static + FnMut(Window, ISize) {
        event_add!(self.base.on_resize.clone(), f);
        self
    }

    pub fn on_move<F>(self, f: F) -> WindowBuilder
    where F: 'static + FnMut(Window, IPoint) {
        event_add!(self.base.on_move.clone(), f);
        self
    }

    pub fn on_show<F>(self, f: F) -> WindowBuilder
    where F: 'static + FnMut(Window) {
        event_add!(self.base.on_show.clone(), f);
        self
    }

    pub fn on_hide<F>(self, f: F) -> WindowBuilder
    where F: 'static + FnMut(Window) {
        event_add!(self.base.on_hide.clone(), f);
        self
    }
}

