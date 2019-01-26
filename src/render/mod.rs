use crate::gfx;
use gfx_hal::{self as hal, Device, Instance, PhysicalDevice, QueueFamily, Surface, Swapchain};
use hal::format::Format;
use std::borrow::Borrow;
use std::sync::{mpsc, Arc};
use std::thread;
use winit::{self, dpi::PhysicalSize, WindowId};

mod frame;

pub use frame::Frame;

pub struct Thread {
    instance: Arc<gfx::Instance>,
    tx: mpsc::SyncSender<Msg>,
    join_handle: thread::JoinHandle<()>,
}

impl Thread {
    pub fn new<Ws>(windows: Ws) -> Thread
    where
        Ws: IntoIterator,
        Ws::Item: Borrow<winit::Window>,
    {
        let instance = Arc::new(gfx::Instance::create("ows-rs", 0));
        let windows = windows
            .into_iter()
            .map(|w| {
                let w = w.borrow();
                let size = w
                    .get_inner_size()
                    .map(|s| s.to_physical(w.get_hidpi_factor()))
                    .expect("only active window can be sent to render thread");
                WindowInfo {
                    id: w.id(),
                    size,
                    surf: instance.create_surface(&w),
                }
            })
            .collect();

        let instance2 = instance.clone();
        let (tx, rx) = mpsc::sync_channel::<Msg>(1);
        let join_handle = thread::spawn(move || {
            render_loop(instance2, windows, rx);
        });
        Thread {
            instance,
            tx,
            join_handle,
        }
    }

    pub fn add_window(&self, window: &winit::Window) {
        let size = window
            .get_inner_size()
            .map(|s| s.to_physical(window.get_hidpi_factor()))
            .expect("only active window can be sent to render thread");
        let info = WindowInfo {
            id: window.id(),
            size,
            surf: self.instance.create_surface(&window),
        };
        self.tx
            .send(Msg::WindowAdd(info))
            .expect("Could not send new window to render thread");
    }

    pub fn remove_window(&self, id: WindowId) {
        self.tx
            .send(Msg::WindowRemove(id))
            .expect("Could not remove window from render thread");
    }

    pub fn frame(&self, frame: Frame) {
        self.tx
            .send(Msg::Frame(frame))
            .expect("Could not send frame to render thread");
    }

    pub fn frames(&self, frames: Vec<Frame>) {
        self.tx
            .send(Msg::Frames(frames))
            .expect("Could not send frames to render thread");
    }

    pub fn stop(self) {
        self.tx
            .send(Msg::Exit)
            .expect("Could not send exit message to render thread");
        self.join_handle
            .join()
            .expect("Could not join the render thread");
    }
}

enum Msg {
    WindowAdd(WindowInfo),
    WindowRemove(WindowId),
    Frame(frame::Frame),
    Frames(Vec<frame::Frame>),
    Exit,
}

struct WindowInfo {
    id: WindowId,
    size: PhysicalSize,
    surf: gfx::Surface,
}

fn render_loop(instance: Arc<gfx::Instance>, windows: Vec<WindowInfo>, rx: mpsc::Receiver<Msg>) {
    let mut renderer = Renderer::new(instance, windows);
    for msg in rx {
        match msg {
            Msg::WindowAdd(info) => {
                renderer.window_add(info);
            }
            Msg::WindowRemove(id) => {
                renderer.window_remove(id);
            }
            Msg::Frame(frame) => {
                renderer.frame(frame);
            }
            Msg::Frames(_) => {}
            Msg::Exit => {
                break;
            }
        }
    }
    renderer.destroy();
}

struct Renderer {
    physical_device: gfx::PhysicalDevice,
    device: gfx::Device,
    queues: gfx::QueueGroup,
    _memory_props: hal::MemoryProperties,
    windows: Vec<Window>,
}

impl Renderer {
    fn new(instance: Arc<gfx::Instance>, windows: Vec<WindowInfo>) -> Renderer {
        use gfx_hal::Graphics;
        for (idx, adapter) in instance.enumerate_adapters().iter().enumerate() {
            println!("Adapter {}: {:?}", idx, adapter.info);
        }
        let (adapter, device, queues) = instance
            .enumerate_adapters()
            .into_iter()
            .map(|a| {
                let dq = a.open_with::<_, Graphics>(1, |qf| {
                    qf.supports_graphics()
                        && qf.supports_transfer()
                        && windows.iter().all(|w| w.surf.supports_queue_family(qf))
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

        let physical_device = adapter.physical_device;
        let memory_props = physical_device.memory_properties();
        let mut renderer = Renderer {
            physical_device,
            device,
            queues,
            _memory_props: memory_props,
            windows: Vec::with_capacity(windows.len()),
        };
        renderer.windows = windows
            .into_iter()
            .map(|info| Window::new(info, &renderer))
            .collect();
        renderer
    }

    fn destroy(self) {
        self.device.wait_idle().unwrap();
        for w in self.windows.into_iter() {
            w.destroy(&self.device);
        }
        std::mem::drop(self.queues);
        std::mem::drop(self.device);
    }

    fn window_add(&mut self, info: WindowInfo) {
        self.windows.push(Window::new(info, self));
    }

    fn window_remove(&mut self, _id: WindowId) {}

    fn frame(&mut self, frame: frame::Frame) {
        let w = self
            .windows
            .iter_mut()
            .find(|w| w.id == frame.window)
            .expect("Frame sent to render thread with an unknown window token");

        let idx = unsafe {
            w.swapchain
                .acquire_image(u64::max_value(), hal::FrameSync::Semaphore(&w.image_avail))
        };
        match idx {
            Err(err) => match err {
                hal::AcquireError::OutOfDate => {
                    w.must_rebuild = true;
                }
                _ => panic!("{:?}", err),
            },
            Ok(idx) => unsafe {
                let image = &mut w.images[idx as usize];
                let cmd = &mut image.cmd;

                self.device
                    .wait_for_fence(&image.fence, u64::max_value())
                    .unwrap();
                self.device.reset_fence(&image.fence).unwrap();
                cmd.begin();

                let subrange = hal::image::SubresourceRange {
                    aspects: hal::format::Aspects::COLOR,
                    levels: 0..1,
                    layers: 0..1,
                };

                if let Some(cc) = frame.clear_color {
                    cmd.clear_image(
                        &image.image,
                        hal::image::Layout::TransferDstOptimal,
                        hal::command::ClearColor::Float(cc),
                        hal::command::ClearDepthStencil(0f32, 0),
                        &[subrange],
                    );
                }

                cmd.finish();

                let submission = hal::Submission {
                    command_buffers: Some(&*cmd),
                    wait_semaphores: Some((&w.image_avail, hal::pso::PipelineStage::TRANSFER)),
                    signal_semaphores: Some(&w.render_done),
                };

                self.queues.queues[0].submit(submission, Some(&image.fence));

                if let Err(_) =
                    w.swapchain
                        .present(&mut self.queues.queues[0], idx, Some(&w.render_done))
                {
                    w.must_rebuild = true;
                }
            },
        }
    }
}

struct Window {
    id: WindowId,
    _size: (u32, u32),
    _surf: gfx::Surface,
    swapchain: gfx::Swapchain,
    image_avail: gfx::Semaphore,
    render_done: gfx::Semaphore,
    pool: gfx::CommandPool,
    images: Vec<ImageData>,
    must_rebuild: bool,
}

struct ImageData {
    image: gfx::Image,
    cmd: gfx::CommandBuffer,
    fence: gfx::Fence,
}

impl ImageData {
    fn new(image: gfx::Image, pool: &mut gfx::CommandPool, dev: &gfx::Device) -> ImageData {
        ImageData {
            image,
            cmd: pool.acquire_command_buffer(),
            fence: dev.create_fence(true).unwrap(),
        }
    }
}

impl Window {
    fn new(mut info: WindowInfo, renderer: &Renderer) -> Window {
        let dev = &renderer.device;
        let pd = &renderer.physical_device;
        let queues = &renderer.queues;

        let (swapchain, images) = build_swapchain(&mut info, pd, dev, None);
        let mut pool = unsafe {
            dev.create_command_pool_typed(
                &queues,
                hal::pool::CommandPoolCreateFlags::RESET_INDIVIDUAL,
            )
        }
        .expect("could not create a command pool");
        let images = images
            .into_iter()
            .map(|i| ImageData::new(i, &mut pool, dev))
            .collect();

        Window {
            id: info.id,
            _size: info.size.into(),
            _surf: info.surf,
            swapchain,
            image_avail: dev.create_semaphore().unwrap(),
            render_done: dev.create_semaphore().unwrap(),
            pool,
            images,
            must_rebuild: false,
        }
    }

    fn destroy(mut self, dev: &gfx::Device) {
        unsafe {
            dev.destroy_semaphore(self.image_avail);
            dev.destroy_semaphore(self.render_done);
            dev.destroy_swapchain(self.swapchain);
            for ImageData { cmd, fence, .. } in self.images.into_iter() {
                self.pool.free(Some(cmd));
                dev.destroy_fence(fence);
            }
            dev.destroy_command_pool(self.pool.into_raw());
        }
    }
}

const COMPALPHA_ORDER: [hal::CompositeAlpha; 4] = [
    hal::CompositeAlpha::PreMultiplied,
    hal::CompositeAlpha::PostMultiplied,
    hal::CompositeAlpha::Inherit,
    hal::CompositeAlpha::Opaque,
];

fn find_surf_comp_alpha(compat: Vec<hal::CompositeAlpha>) -> hal::CompositeAlpha {
    for &wish in &COMPALPHA_ORDER {
        if let Some(_) = compat.iter().find(|&&ca| ca == wish) {
            return wish;
        }
    }
    panic!();
}

fn find_surf_format(compat: Option<Vec<Format>>) -> Format {
    compat.map_or(Format::Rgba8Unorm, |compat| {
        compat
            .iter()
            .find(|&&f| f == Format::Rgba8Unorm)
            .map(|f| *f)
            .unwrap_or(
                compat
                    .iter()
                    .find(|&&f| f.base_format().1 == hal::format::ChannelType::Unorm)
                    .map(|f| *f)
                    .unwrap_or(compat[0]),
            )
    })
}

fn build_swapchain(
    info: &mut WindowInfo,
    pd: &gfx::PhysicalDevice,
    dev: &gfx::Device,
    old: Option<gfx::Swapchain>,
) -> (gfx::Swapchain, Vec<gfx::Image>) {
    use hal::image;
    let (caps, formats, present_modes, comp_alpha) = info.surf.compatibility(&pd);
    let usage = image::Usage::TRANSFER_DST | image::Usage::COLOR_ATTACHMENT;
    assert!(caps.usage.contains(usage));
    let image_count = std::cmp::max(2, caps.image_count.start);
    let format = find_surf_format(formats);
    assert!(present_modes
        .iter()
        .find(|&&pm| pm == hal::PresentMode::Fifo)
        .is_some());
    let present_mode = hal::PresentMode::Fifo;
    let size: (u32, u32) = info.size.into();
    let mut config = hal::SwapchainConfig::new(size.0, size.1, format, image_count)
        .with_mode(present_mode)
        .with_image_usage(usage);
    config.composite_alpha = find_surf_comp_alpha(comp_alpha);
    println!("Creating swapchain {}x{}", size.0, size.1);
    let (swapchain, backbuffer) = unsafe { dev.create_swapchain(&mut info.surf, config, old) }
        .expect("Can't create swapchain");
    let images = {
        match backbuffer {
            hal::Backbuffer::Images(images) => images,
            _ => panic!("Framebuffer Backbuffer unsupported"),
        }
    };
    (swapchain, images)
}
