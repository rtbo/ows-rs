#[cfg(unix)]
pub mod wayland;

#[cfg(windows)]
pub mod win32;

use crate::window::Window;

pub trait Display: Drop + Sized {
    type OpenError;
    type Window: Window<Self>;

    fn open() -> Result<Self, Self::OpenError>;

    fn create_window(&self) -> Self::Window;

    fn collect_events(&self);
}
