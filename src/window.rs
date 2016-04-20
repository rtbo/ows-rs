
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

define_handler!{OnCloseHandler: FnMut(w: &mut Window) => bool}
define_handler!{OnResizeHandler: FnMut(w: &mut Window, new_size: ISize)}

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


    fn on_close_do(&mut self, handler: Box<FnMut(&mut Window) -> bool>) {
        let hdler = self.on_close();
        hdler.borrow_mut().set(Some(handler));
    }
    fn on_close_do_nothing(&mut self) {
        let hdler = self.on_close();
        hdler.borrow_mut().set(None);
    }
    fn on_close(&self) -> RcCell<OnCloseHandler> {
        self.base().on_close.clone()
    }


    fn on_resize_add(&mut self, handler: Box<FnMut(&mut Window, ISize)>) -> usize {
        let hdler = self.on_resize();
        let id = hdler.borrow_mut().add(handler);
        id
    }
    fn on_resize_rem(&mut self, id: usize) -> bool {
        let hdler = self.on_resize();
        let res = hdler.borrow_mut().remove(id);
        res
    }
    fn on_resize(&self) -> RcCell<OnResizeHandler> {
        self.base().on_resize.clone()
    }

}


pub struct WindowBase {
    on_close: RcCell<OnCloseHandler>,
    on_resize: RcCell<OnResizeHandler>,
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
macro_rules! fire {
    ($hdler:expr, $($p:expr),*) => {{
        // A lifetime error occurs without this no-op let.
        let hdler = $hdler;
        hdler.borrow_mut().fire($($p),*);
    }};
}

#[macro_export]
macro_rules! fire_res {
    ($hdler:expr, $($p:expr),*) => {{
        // A lifetime error occurs without this no-op let.
        let hdler = $hdler;
        let res = hdler.borrow_mut().fire($($p),*);
        res
    }};
}

#[macro_export]
macro_rules! fire_or {
    ($hdler:expr, $($p:expr),+) => {{
        // A lifetime error occurs without this no-op let.
        let hdler = $hdler;
        let res = hdler.borrow_mut().fire_or($($p),+);
        res
    }};
}
