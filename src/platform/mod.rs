
#[cfg(target_os = "linux")]
pub mod xcbplatform;
#[cfg(target_os = "windows")]
pub mod win32platform;


use ::RcCell;
use window::{self, WindowBase};

use std::rc::Rc;
use std::ops::Deref;


pub trait EventLoop {
    fn loop_events(&self) -> i32;
    fn exit(&self, code: i32);
}


pub trait Platform : EventLoop {
    fn create_window(&self, base: RcCell<WindowBase>)
            -> Rc<PlatformWindow>;
}


pub trait PlatformWindow {
    fn update_title(&self);

    fn state(&self) -> window::State;
    fn set_state(&self, state: window::State);

    fn close(&self);
}

impl Platform for Rc<Platform> {
    fn create_window(&self, base: RcCell<WindowBase>) -> Rc<PlatformWindow> {
        self.deref().create_window(base)
    }
}

impl EventLoop for Rc<Platform> {
    fn loop_events(&self) -> i32 {
        self.deref().loop_events()
    }
    fn exit(&self, code: i32) {
        self.deref().exit(code);
    }
}
