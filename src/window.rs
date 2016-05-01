
use geometry::ISize;
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
define_handler!{OnResizeHandler: FnMut(w: Window, new_size: ISize)}


#[macro_export]
macro_rules! handler_do {
    ($handler:expr, $closure:expr) => {{
        // A lifetime error occurs without this no-op let.
        let handler = $handler;
        handler.borrow_mut().set(Some(Box::new($closure)));
    }};
}

#[macro_export]
macro_rules! handler_do_nothing {
    ($handler:expr) => {{
        // A lifetime error occurs without this no-op let.
        let handler = $handler;
        handler.borrow_mut().set(None);
    }};
}

#[macro_export]
macro_rules! handler_add {
    ($handler:expr, $closure:expr) => {{
        // A lifetime error occurs without this no-op let.
        let handler = $handler;
        let id = handler.borrow_mut().add(Box::new($closure));
        id
    }};
}

#[macro_export]
macro_rules! handler_rem {
    ($handler:expr, $id:expr) => {{
        // A lifetime error occurs without this no-op let.
        let handler = $handler;
        let res = handler.borrow_mut().remove($id);
        res
    }};
}


#[macro_export]
macro_rules! fire {
    ($handler:expr, $($p:expr),*) => {{
        // A lifetime error occurs without this no-op let.
        let handler = $handler;
        handler.borrow_mut().fire($($p),*);
    }};
}

#[macro_export]
macro_rules! fire_res {
    ($handler:expr, $($p:expr),*) => {{
        // A lifetime error occurs without this no-op let.
        let handler = $handler;
        let res = handler.borrow_mut().fire($($p),*);
        res
    }};
}

#[macro_export]
macro_rules! fire_or {
    ($handler:expr, $($p:expr),+) => {{
        // A lifetime error occurs without this no-op let.
        let handler = $handler;
        let res = handler.borrow_mut().fire_or($($p),+);
        res
    }};
}


#[derive(Clone)]
pub struct WindowBase {
    pub title: String,
    pub state: State,
    pub on_close: RcCell<OnCloseHandler>,
    pub on_resize: RcCell<OnResizeHandler>,
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
                on_resize: Rc::new(RefCell::new(OnResizeHandler::new())),
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

    pub fn on_resize(&self) -> RcCell<OnResizeHandler> {
        self.base.borrow().on_resize.clone()
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
        handler_add!(self.base.on_resize.clone(), f);
        self
    }
}

