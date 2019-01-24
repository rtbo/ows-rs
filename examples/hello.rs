
extern crate ows;
#[cfg(unix)]
use ows::display::wayland as disp;
#[cfg(windows)]
use ows::display::win32 as disp;

use ows::display::Display;
use ows::geom::IRect;
use ows::key;
use ows::render::{self, frame::Frame};
use ows::window::{self, Window};

use std::thread;
use std::sync::mpsc;

fn main() {
    let dpy = disp::Display::open().expect("could not open display");
    let mut win = dpy.create_window();

    win.set_title(String::from("Hello, Ows!"));

    win.show(window::State::Normal(Some((640, 480))));

    let token = win.token();

    // spawn the render thread
    let inst = dpy.instance();
    let rwins = vec![ render::WindowInfo::new(token, win.size(), win.create_surface()) ];
    let (tx, rx) = mpsc::sync_channel::<render::Msg>(1);
    thread::spawn(move || {
        render::render_loop(inst, rwins, rx);
    });

    'main: loop {
        dpy.collect_events();
        for ev in win.retrieve_events() {
            println!("received event: {:?}", &ev);
            match ev {
                window::Event::Close => {
                    tx.send(render::Msg::WindowClose(token)).unwrap();
                    win.close();
                    break 'main;
                }
                window::Event::KeyDown(sym, _, _, _) => {
                    if sym == key::Sym::Escape {
                        tx.send(render::Msg::WindowClose(token)).unwrap();
                        win.close();
                        break 'main;
                    }
                }
                _ => {}
            }
        }
        tx.send(render::Msg::Frame(Frame::new(
            token, IRect::new(0, 0, 640, 480), Some([0.8f32, 0.5f32, 0.6f32, 1f32])
        ))).unwrap();
    }
    tx.send(render::Msg::Exit).unwrap();
}
