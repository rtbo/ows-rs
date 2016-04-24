
#[cfg(target_os = "linux")]
pub mod xcbplatform;
#[cfg(target_os = "windows")]
pub mod win32platform;


use ::RcCell;
use window::{self, WindowBase};


pub trait EventLoop {
    fn loop_events(&self) -> i32;
    fn exit(&self, code: i32);
}


pub trait Platform : EventLoop {
    fn create_window(&self, base: WindowBase)
            -> RcCell<PlatformWindow>;
}


pub trait PlatformWindow {

    fn base(&self) -> &WindowBase;
    fn base_mut(&mut self) -> &mut WindowBase;

    fn title(&self) -> String;
    fn set_title(&mut self, title: String);

    fn state(&self) -> window::State;
    fn set_state(&mut self, state: window::State);

    fn close(&mut self);

}


