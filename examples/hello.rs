
extern crate ows;
#[cfg(unix)]
use ows::display::wayland as disp;
#[cfg(windows)]
use ows::display::win32 as disp;

use ows::display::Display;
use ows::window::{self, Window};

fn main() {
    let dpy = disp::Display::open().expect("could not open display");
    let mut win = dpy.create_window();

    win.set_title(String::from("Hello, Ows!"));

    win.show(window::State::Normal(Some((640, 480))));

    std::thread::sleep(std::time::Duration::from_secs(3));
}
