use gfx_back as back;
use gfx_hal as hal;

pub type Instance = back::Instance;
pub type Surface = <back::Backend as hal::Backend>::Surface;
pub type Device = <back::Backend as hal::Backend>::Device;
pub type QueueFamily = <back::Backend as hal::Backend>::QueueFamily;
pub type CommandQueue = hal::CommandQueue<back::Backend, hal::Graphics>;
pub type Swapchain = <back::Backend as hal::Backend>::Swapchain;
