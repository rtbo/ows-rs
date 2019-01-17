
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

    'main: loop {
        dpy.collect_events();
        for ev in win.retrieve_events() {
            println!("received event: {:?}", &ev);
            match ev {
                window::Event::Close => {
                    win.close();
                    break 'main;
                },
                _ => {}
            }
        }
    }
}
