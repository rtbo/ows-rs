
extern crate ows;
#[cfg(unix)]
use ows::display::wayland as disp;
#[cfg(windows)]
use ows::display::win32 as disp;

use ows::display::Display;
use ows::geom::IRect;
use ows::render::{self, frame::Frame};
use ows::window::{self, Window};

use std::thread;
use std::sync::mpsc;

fn main() {
    let dpy = disp::Display::open().expect("could not open display");
    let mut win = dpy.create_window();

    win.set_title(String::from("Hello, Ows!"));

    win.show(window::State::Normal(Some((640, 480))));

    // spawn the render thread
    let inst = dpy.instance();
    let (tx, rx) = mpsc::sync_channel::<render::Msg>(1);
    thread::spawn(move || {
        render::render_loop(inst, rx);
    });

    let token = win.token();

    // let the render thread do the boiler plate for our window
    tx.send(render::Msg::WindowOpen(token, win.create_surface())).unwrap();

    'main: loop {
        dpy.collect_events();
        for ev in win.retrieve_events() {
            println!("received event: {:?}", &ev);
            match ev {
                window::Event::Close => {
                    win.close();
                    tx.send(render::Msg::WindowClose(token)).unwrap();
                    break 'main;
                },
                _ => {}
            }
        }
        tx.send(render::Msg::Frame(Frame::new(
            token, IRect::new(0, 0, 640, 480), Some([0.8f32, 0.5f32, 0.6f32, 1f32])
        ))).unwrap();
    }
}
