
#[macro_use]
extern crate bitflags;

#[cfg(unix)]
extern crate wayland_client as wlc;
#[cfg(unix)]
extern crate wayland_protocols as wlp;

pub mod display;
pub mod event;
pub mod key;
pub mod window;
