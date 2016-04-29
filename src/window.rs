
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

define_handler!{OnCloseHandler: FnMut() => bool}
define_handler!{OnResizeHandler: FnMut(new_size: ISize)}

pub struct Window {
    base: RcCell<WindowBase>,
    pw: Rc<PlatformWindow>,
}

impl Window {
    pub fn new(p: &Platform) -> Window {
        let base = Rc::new(RefCell::new(WindowBase::new()));
        let pw = p.create_window(base.clone());
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

    pub fn state(&self) -> State {
        self.pw.state()
    }
    pub fn set_state(&mut self, state: State) {
        self.pw.set_state(state);
    }

    pub fn on_close(&self) -> RcCell<OnCloseHandler> {
        self.base.borrow().on_close()
    }

    pub fn on_resize(&self) -> RcCell<OnResizeHandler> {
        self.base.borrow().on_resize()
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
//pub trait Window {
//
//    fn base(&self) -> &WindowBase;
//    fn base_mut(&mut self) -> &mut WindowBase;
//
//    fn title(&self) -> String;
//    fn set_title(&mut self, tit: String);
//
//    fn show_normal(&mut self) {
//        self.set_state(State::Normal);
//    }
//
//    fn show_minimized(&mut self) {
//        self.set_state(State::Minimized);
//    }
//
//    fn show_maximized(&mut self) {
//        self.set_state(State::Maximized);
//    }
//
//    fn show_fullscreen(&mut self) {
//        self.set_state(State::Fullscreen);
//    }
//
//    fn hide(&mut self) {
//        self.set_state(State::Hidden);
//    }
//
//    fn state(&self) -> State;
//    fn set_state(&mut self, state: State);
//
//
//    fn on_close(&self) -> RcCell<OnCloseHandler> {
//        self.base().on_close.clone()
//    }
//
//    fn on_resize(&self) -> RcCell<OnResizeHandler> {
//        self.base().on_resize.clone()
//    }
//
//
//}


pub struct WindowBase {
    title: String,
    on_close: RcCell<OnCloseHandler>,
    on_resize: RcCell<OnResizeHandler>,
}

impl WindowBase {
    pub fn new() -> WindowBase {
        WindowBase {
            title: String::new(),
            on_close: Rc::new(RefCell::new(OnCloseHandler::new())),
            on_resize: Rc::new(RefCell::new(OnResizeHandler::new())),
        }
    }

    pub fn title(&self) -> String {
        self.title.clone()
    }

    pub fn on_close(&self) -> RcCell<OnCloseHandler> {
        self.on_close.clone()
    }

    pub fn on_resize(&self) -> RcCell<OnResizeHandler> {
        self.on_resize.clone()
    }
}


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
