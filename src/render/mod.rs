use crate::gfx;
use crate::window;
use gfx_hal::{self as hal, Instance, PhysicalDevice, QueueFamily};
use std::sync::{mpsc, Arc};

pub mod frame;

pub enum Msg {
    WindowOpen(window::Token, gfx::Surface),
    WindowClose(window::Token),
    Frame(frame::Frame),
    Frames(Vec<frame::Frame>),
}

pub fn render_loop(instance: Arc<gfx::Instance>, rx: mpsc::Receiver<Msg>) {
    let mut renderer = Renderer::new(instance);
    for msg in rx {
        match msg {
            Msg::WindowOpen(tok, surf) => {
                renderer.window_open(tok, surf);
            }
            Msg::WindowClose(_) => {}
            Msg::Frame(_) => {}
            Msg::Frames(_) => {}
        }
    }
}

struct Renderer {
    _instance: Arc<gfx::Instance>,
    _device: gfx::Device,
    _queue: gfx::CommandQueue,
    _memory_props: hal::MemoryProperties,
    windows: Vec<Window>,
}

impl Renderer {
    fn new(instance: Arc<gfx::Instance>) -> Renderer {
        use gfx_hal::Graphics;
        for (idx, adapter) in instance.enumerate_adapters().iter().enumerate() {
            println!("Adapter {}: {:?}", idx, adapter.info);
        }
        let (adapter, device, mut queues) = instance
            .enumerate_adapters()
            .into_iter()
            .map(|a| {
                let dq = a.open_with::<_, Graphics>(1, |qf| {
                    qf.supports_graphics() && qf.supports_transfer()
                });
                (a, dq)
            })
            // filter out devices that can't open
            .filter_map(|adq| {
                let (a, dq) = (adq.0, adq.1);
                dq.ok().map(|dq| (a, dq.0, dq.1))
            })
            // take the first one that can open
            .nth(0)
            .expect("could not open a graphics adapter");

        let queue = queues.queues.remove(0);
        let memory_props = adapter.physical_device.memory_properties();

        Renderer {
            _instance: instance,
            _device: device,
            _queue: queue,
            _memory_props: memory_props,
            windows: Vec::new(),
        }
    }
    fn window_open(&mut self, tok: window::Token, surf: gfx::Surface) {
        self.windows.push(Window::new(tok, surf));
    }
}

struct Window {
    _tok: window::Token,
    _surf: gfx::Surface,
    //swapchain: gfx::Swapchain,
}

impl Window {
    fn new(tok: window::Token, surf: gfx::Surface) -> Window {
        Window {
            _tok: tok,
            _surf: surf,
        }
    }
}
