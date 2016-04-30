#[macro_use]
extern crate ows;

use ows::{window, Window};

fn main() {
    let p = ows::default_platform()
        .expect("could not create platform");

    let mut attempts = 0;

    let mut w = Window::new()
        .title("Hello, World!".to_string())
        .on_close(move || {
            attempts += 1;
            let comment = if attempts == 2 {"I'm done!"} else {"Try again!"};
            println!("closing attempt n°{}. {}", attempts, comment);
            attempts == 2
        })
        .state(window::State::Normal)
        .done(&p);

    {
        let mut w = w.clone();
        handler_add!(w.on_resize(), move |s| {
            w.set_title(format!("Hello, World; new size: {:?}", s));
        });
    }

    w.show_normal();

    std::process::exit(p.loop_events());
}
