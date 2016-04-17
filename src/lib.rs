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

#[macro_use]
pub mod macros;
pub mod platform;
pub mod geometry;
pub mod key;
pub mod mouse;
pub mod window;

pub use platform::*;
pub use window::Window;


#[cfg(target_os="linux")]
pub fn default_platform() -> Option<Box<Platform>> {
    use platform::xcbplatform::XcbPlatform;
    XcbPlatform::new().map(|p| Box::new(p) as Box<Platform>)
}
