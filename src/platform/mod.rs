

use std::ops::Deref;
use std::ops::DerefMut;


pub mod xcbplatform;

use window::Window;

pub type WinId = usize;

pub fn with_win<F, R>(p: &Platform, wid: WinId, f: F) -> R
where F: FnOnce(&Window) -> R {
    let window = p.window(wid);
    f(window)
}

pub fn with_win_mut<F, R>(p: &mut Platform, wid: WinId, f: F) -> R
where F: FnOnce(&mut Window) -> R {
    let mut window = p.window_mut(wid);
    f(window)
}


pub trait EventLoop {
    fn loop_events(&mut self) -> i32;
}


pub trait Platform : EventLoop {
    fn create_window(&mut self) -> WinId;
    fn window(&self, id: WinId) -> &Window;
    fn window_mut(&mut self, id: WinId) -> &mut Window;
}


impl EventLoop for Box<Platform> {
    fn loop_events(&mut self) -> i32 {
        (*self).deref_mut().loop_events()
    }
}
impl Platform for Box<Platform> {
    fn create_window(&mut self) -> WinId {
        (*self).deref_mut().create_window()
    }
    fn window(&self, id: WinId) -> &Window {
        (*self).deref().window(id)
    }
    fn window_mut(&mut self, id: WinId) -> &mut Window {
        (*self).deref_mut().window_mut(id)
    }
}

