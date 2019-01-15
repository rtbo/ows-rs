
#[macro_use]
extern crate bitflags;

#[cfg(unix)]
extern crate wayland_client as wlc;
#[cfg(unix)]
extern crate wayland_protocols as wlp;

#[cfg(windows)]
extern crate winapi;
#[cfg(windows)]
extern crate libc;

pub mod display;
pub mod geometry;
pub mod key;
pub mod mouse;
pub mod window;
