
extern crate ows;
use ows::display::wayland as disp;

fn main() {
    let dpy = disp::Display::open().expect("could not open display");
}
