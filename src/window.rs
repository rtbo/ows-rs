
use geometry::ISize;
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
    pub on_close: RcCell<OnCloseHandler>,
    pub on_resize: RcCell<OnResizeHandler>,
}

impl WindowBase {
    pub fn new() -> WindowBase {
        WindowBase {
            on_close: Rc::new(RefCell::new(OnCloseHandler::new())),
            on_resize: Rc::new(RefCell::new(OnResizeHandler::new())),
        }
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
