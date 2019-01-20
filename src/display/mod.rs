use crate::window::Window;
use std::sync::Arc;

#[cfg(unix)]
pub mod wayland;
#[cfg(windows)]
pub mod win32;

pub trait Display: Drop + Sized {
    type OpenError;
    type Window: Window<Self>;

    fn open() -> Result<Self, Self::OpenError>;

    fn create_window(&self) -> Self::Window;

    fn collect_events(&self);

    fn instance(&self) -> Arc<gfx_back::Instance>;
}
