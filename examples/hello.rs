#[macro_use]
extern crate ows;

use ows::{Platform, Window};

fn main() {
    let mut p = ows::default_platform()
        .expect("could not create platform");

    let wid = p.create_window();

    // accessing using "with_win" idiom
    ows::with_win_mut(&mut p, wid,
        |w| w.set_title("Hello, World".to_string())
    );

    println!("{}", ows::with_win(&p, wid, |w| w.title()));

    // alternatively, accessing using "getter" idiom
    {
        let w = p.window_mut(wid);
        let mut attempts = 0;
        handler_do!(w.on_close(), move |_| {
            attempts += 1;
            attempts == 2
        });
        handler_add!(w.on_resize(), |w, s| {
            w.set_title(format!("Hello, World; new size: {:?}", s));
        });
    }

    p.window_mut(wid).show_normal();

    std::process::exit(p.loop_events());
}
