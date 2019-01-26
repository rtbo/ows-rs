use crate::geom::IRect;
use winit::WindowId;

pub struct Frame {
    pub window: WindowId,
    pub viewport: IRect,
    pub clear_color: Option<[f32; 4]>,
}

impl Frame {
    pub fn new (window: WindowId, viewport: IRect, clear_color: Option<[f32; 4]>) -> Frame {
        Frame { window, viewport, clear_color }
    }
}