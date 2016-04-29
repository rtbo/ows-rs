#[macro_use]
extern crate ows;

use ows::{Window};

fn main() {
    let p = ows::default_platform()
        .expect("could not create platform");

    let mut w = Window::new(&p);
    w.set_title("Hello, World".to_string());

    {
        let mut attempts = 0;
        handler_do!(w.on_close(), move || {
            attempts += 1;
            println!("closing attempt n°{}", attempts);
            attempts == 2
        });
        let mut w = w.clone();
        handler_add!(w.on_resize(), move |s| {
            w.set_title(format!("Hello, World; new size: {:?}", s));
        });
    }

    w.show_normal();

    std::process::exit(p.loop_events());
}
