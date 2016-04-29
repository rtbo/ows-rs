
#![feature(log_syntax)]
#![feature(downgraded_weak)]

#[cfg(target_os = "windows")]
extern crate winapi;
#[cfg(target_os = "windows")]
extern crate user32;
#[cfg(target_os = "windows")]
extern crate kernel32;

#[cfg(target_os = "linux")]
extern crate xcb;
#[cfg(target_os = "linux")]
extern crate x11;
#[cfg(target_os = "linux")]
extern crate xkbcommon;

extern crate libc;

#[macro_use]
extern crate log;

use std::rc::{Rc, Weak};
use std::cell::RefCell;

#[macro_use]
pub mod macros;
#[macro_use]
pub mod handler;
#[macro_use]
pub mod window;
pub mod platform;
pub mod geometry;
pub mod key;
pub mod mouse;

pub use platform::*;
pub use window::Window;

pub type RcCell<T> = Rc<RefCell<T>>;
pub type WeakCell<T> = Weak<RefCell<T>>;

#[cfg(target_os="linux")]
pub fn default_platform() -> Option<Rc<Platform>> {
    use platform::xcbplatform::XcbPlatform;
    XcbPlatform::new().map(|p| Rc::new(p))
}

#[cfg(target_os="windows")]
pub fn default_platform() -> Option<Rc<Platform>> {
    use platform::win32platform::Win32Platform;
    Some(Rc::new(Win32Platform::new()))
}
