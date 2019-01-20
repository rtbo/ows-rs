use crate::geom::IRect;

pub struct Frame {
    pub viewport: IRect,
    pub clear_color: Option<[f32; 4]>,
}
