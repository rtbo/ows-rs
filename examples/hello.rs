
extern crate ows;
#[cfg(unix)]
use ows::display::wayland as disp;
use ows::window::Window;

fn main() {
    let mut dpy = disp::Display::open().expect("could not open display");
    let mut win = Window::new(&mut dpy);
    win.set_title(String::from("Hello, Ows!"))
}
