use crate::geom::IRect;
use crate::window;

pub struct Frame {
    pub window: window::Token,
    pub viewport: IRect,
    pub clear_color: Option<[f32; 4]>,
}

impl Frame {
    pub fn new (window: window::Token, viewport: IRect, clear_color: Option<[f32; 4]>) -> Frame {
        Frame { window, viewport, clear_color }
    }
}