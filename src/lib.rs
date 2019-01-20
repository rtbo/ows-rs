#[macro_use]
extern crate bitflags;

extern crate gfx_hal as hal;
extern crate gfx_backend_vulkan as gfx_back;
extern crate libc;

#[cfg(unix)]
extern crate wayland_client as wlc;
#[cfg(unix)]
extern crate wayland_protocols as wlp;

#[cfg(windows)]
extern crate winapi;

pub mod display;
pub mod geom;
pub mod gfx;
pub mod key;
pub mod mouse;
pub mod render;
pub mod window;
