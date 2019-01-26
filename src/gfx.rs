use gfx_back as back;
use gfx_hal as hal;

pub type Backend = back::Backend;
pub type CommandBuffer = hal::command::CommandBuffer<Backend, hal::Graphics, hal::command::OneShot>;
pub type CommandPool = hal::CommandPool<Backend, hal::Graphics>;
pub type CommandQueue = hal::CommandQueue<Backend, hal::Graphics>;
pub type Device = <Backend as hal::Backend>::Device;
pub type Fence = <Backend as hal::Backend>::Fence;
pub type Image = <Backend as hal::Backend>::Image;
pub type Instance = back::Instance;
pub type PhysicalDevice = <Backend as hal::Backend>::PhysicalDevice;
pub type QueueFamily = <Backend as hal::Backend>::QueueFamily;
pub type QueueGroup = hal::QueueGroup<Backend, hal::Graphics>;
pub type Semaphore = <Backend as hal::Backend>::Semaphore;
pub type Surface = <Backend as hal::Backend>::Surface;
pub type Swapchain = <Backend as hal::Backend>::Swapchain;
