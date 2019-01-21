use crate::geom::ISize;
use crate::gfx;
use crate::window;
use gfx_hal::{
    self as hal, Device, Instance, PhysicalDevice, QueueFamily, Surface, Swapchain,
};
use hal::{format::Format, command::CommandBuffer, command::Submittable};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{mpsc, Arc};

pub mod frame;

pub enum Msg {
    WindowOpen(WindowInfo),
    WindowClose(window::Token),
    Frame(frame::Frame),
    Frames(Vec<frame::Frame>),
    Exit,
}

pub struct WindowInfo {
    token: window::Token,
    size: ISize,
    surf: gfx::Surface,
}

impl WindowInfo {
    pub fn new(token: window::Token, size: ISize, surf: gfx::Surface) -> WindowInfo {
        WindowInfo { token, size, surf }
    }
}

pub fn render_loop(
    instance: Arc<gfx::Instance>,
    windows: Vec<WindowInfo>,
    rx: mpsc::Receiver<Msg>,
) {
    let mut renderer = Renderer::new(instance, windows);
    for msg in rx {
        match msg {
            Msg::WindowOpen(info) => {
                renderer.window_open(info);
            }
            Msg::WindowClose(tok) => {
                renderer.window_close(tok);
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
}

struct Renderer {
    _instance: Arc<gfx::Instance>,
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
        let (adapter, device, mut queues) = instance
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
            _instance: instance,
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
    fn window_open(&mut self, info: WindowInfo) {
        self.windows.push(Window::new(info, self));
    }
    fn window_close(&mut self, _tok: window::Token) {}
    fn frame(&mut self, frame: frame::Frame) {
        let mut w = self
            .windows
            .iter_mut()
            .find(|w| w.token == frame.window)
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

                let subrange = hal::image::SubresourceRange{
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
                        &[ subrange ],
                    );
                }

                cmd.finish();

                let submission = hal::Submission{
                    command_buffers: Some(&*cmd),
                    wait_semaphores: Some((&w.image_avail, hal::pso::PipelineStage::TRANSFER)),
                    signal_semaphores: Some(&w.render_done),
                };

                self.queues.queues[0].submit(submission, Some(&image.fence));

                if let Err(_) = w.swapchain.present(&mut self.queues.queues[0], idx, Some(&w.render_done)) {
                    w.must_rebuild = true;
                }

            },
        }
    }
}

struct Window {
    token: window::Token,
    _size: ISize,
    _surf: gfx::Surface,
    swapchain: gfx::Swapchain,
    image_avail: gfx::Semaphore,
    render_done: gfx::Semaphore,
    images: Vec<ImageData>,
    must_rebuild: bool,
}

struct ImageData {
    image: gfx::Image,
    pool: Rc<RefCell<gfx::CommandPool>>,
    cmd: gfx::CommandBuffer,
    fence: gfx::Fence,
}

impl ImageData {
    fn new(image: gfx::Image, pool: Rc<RefCell<gfx::CommandPool>>, dev: &gfx::Device) -> ImageData {
        let cmd = pool.borrow_mut().acquire_command_buffer();
        let fence = unsafe { dev.create_fence(true) }.unwrap();
        ImageData {
            image,
            pool,
            cmd,
            fence,
        }
    }
}

impl Window {
    fn new(mut info: WindowInfo, renderer: &Renderer) -> Window {
        let dev = &renderer.device;
        let pd = &renderer.physical_device;
        let queues = &renderer.queues;

        let (swapchain, images) = build_swapchain(&mut info, pd, dev, None);
        let pool = Rc::new(RefCell::new(
            unsafe {
                dev.create_command_pool_typed(
                    &queues,
                    hal::pool::CommandPoolCreateFlags::RESET_INDIVIDUAL,
                )
            }
            .expect("could not create a command pool"),
        ));
        let images = images
            .into_iter()
            .map(|i| ImageData::new(i, pool.clone(), dev))
            .collect();

        Window {
            token: info.token,
            _size: info.size,
            _surf: info.surf,
            swapchain,
            image_avail: dev.create_semaphore().unwrap(),
            render_done: dev.create_semaphore().unwrap(),
            images,
            must_rebuild: false,
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
        if let Some(ca) = compat.iter().find(|&&ca| ca == wish) {
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
    let size = info.size;
    let mut config = hal::SwapchainConfig::new(size.w as u32, size.h as u32, format, image_count)
        .with_mode(present_mode)
        .with_image_usage(usage);
    config.composite_alpha = find_surf_comp_alpha(comp_alpha);
    println!("Creating swapchain {}x{}", size.w, size.h);
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
