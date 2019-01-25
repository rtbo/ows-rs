extern crate ows;
extern crate winit;

use ows::geom::IRect;
use ows::gfx;
use ows::render::{self, frame::Frame};

use std::sync::{mpsc, Arc};
use std::thread;

fn main() {
    let mut events_loop = winit::EventsLoop::new();

    let window = winit::WindowBuilder::new()
        .with_title("A fantastic window!")
        .build(&events_loop)
        .unwrap();

    let instance = gfx::Instance::create("hello", 0);
    let surface = instance.create_surface(&window);
    let size = window
        .get_inner_size()
        .map(|s| s.to_physical(window.get_hidpi_factor()))
        .unwrap();

    // spawn the render thread
    let rwins = vec![render::WindowInfo::new(window.id(), size, surface)];

    let (tx, rx) = mpsc::sync_channel::<render::Msg>(1);
    let jh = thread::spawn(move || {
        render::render_loop(Arc::new(instance), rwins, rx);
    });

    events_loop.run_forever(|event| {
        println!("received event: {:?}", event);

        let size: (u32, u32) = window
            .get_inner_size()
            .map(|s| s.to_physical(window.get_hidpi_factor()))
            .unwrap()
            .into();

        tx.send(render::Msg::Frame(Frame::new(
            window.id(),
            IRect::new(0, 0, size.0 as _, size.1 as _),
            Some([0.8f32, 0.5f32, 0.6f32, 1f32]),
        )))
        .unwrap();

        match event {
            winit::Event::WindowEvent { event: winit::WindowEvent::CloseRequested, .. } => {
                winit::ControlFlow::Break
            }
            _ => winit::ControlFlow::Continue,
        }
    });
    tx.send(render::Msg::Exit).unwrap();
    jh.join().unwrap();

}
