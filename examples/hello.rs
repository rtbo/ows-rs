#[macro_use]
extern crate ows;

use ows::{window, Window};

fn main() {
    let p = ows::default_platform()
        .expect("could not create platform");

    let mut attempts = 0;

    let _w = Window::new()
        .title("Hello, World!".to_string())
        .on_close(move |_| {
            attempts += 1;
            let comment = if attempts == 2 {"I'm done!"} else {"Try again!"};
            println!("closing attempt n°{}. {}", attempts, comment);
            attempts == 2
        })
        .on_resize(|mut w, s| {
            w.set_title(format!("Hello, World; new size: {:?}", s));
        })
        .on_move(|_, p| println!("pos: {:?}", p))
        .on_show(|_| println!("show"))
        .on_hide(|_| println!("hide"))
        .on_enter(|_, p| println!("enter {:?}", p))
        .on_leave(|_, p| println!("leave {:?}", p))
        .state(window::State::Normal)
        .done(&p);

    std::process::exit(p.loop_events());
}
