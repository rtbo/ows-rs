
extern crate ows;
#[cfg(unix)]
use ows::display::wayland as disp;

use ows::display::Display;
use ows::window::Window;

fn main() {
    let dpy = disp::Display::open().expect("could not open display");
    let mut win = dpy.create_window();

    win.set_title(String::from("Hello, Ows!"))
}
