extern crate ows;
extern crate winit;

use ows::geom::IRect;
use ows::render;


fn main() {
    let mut events_loop = winit::EventsLoop::new();

    let window = winit::WindowBuilder::new()
        .with_title("A fantastic window!")
        .with_dimensions(winit::dpi::LogicalSize::new(640 as _, 480 as _))
        .build(&events_loop)
        .unwrap();

    let render_thread = render::Thread::new(Some(&window));

    // spawn the render thread
    events_loop.run_forever(|event| {
        println!("received event: {:?}", event);

        let size: (u32, u32) = window
            .get_inner_size()
            .map(|s| s.to_physical(window.get_hidpi_factor()))
            .unwrap()
            .into();

        render_thread.frame(render::Frame::new(
            window.id(),
            IRect::new(0, 0, size.0 as _, size.1 as _),
            Some([0.8f32, 0.5f32, 0.6f32, 1f32]),
        ));

        match event {
            winit::Event::WindowEvent { event: winit::WindowEvent::CloseRequested, .. } => {
                winit::ControlFlow::Break
            }
            _ => winit::ControlFlow::Continue,
        }
    });

    render_thread.stop();
}
