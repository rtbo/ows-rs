
pub type FPoint = Point<f32>;
pub type IPoint = Point<i32>;

pub type FSize = Size<f32>;
pub type ISize = Size<i32>;

pub type FRect = Rect<f32>;
pub type IRect = Rect<i32>;

pub type FMargins = Margins<f32>;
pub type IMargins = Margins<i32>;



#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Point<T: Copy> {
    pub x: T,
    pub y: T,
}

impl<T: Copy> Point<T> {
    pub fn new(x: T, y: T) -> Point<T> {
        Point { x: x, y: y }
    }
}


#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Size<T: Copy> {
    pub w: T,
    pub h: T,
}

impl<T: Copy> Size<T> {
    pub fn new(w: T, h: T) -> Size<T> {
        Size { w: w, h: h }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Rect<T: Copy> {
    pub x: T,
    pub y: T,
    pub w: T,
    pub h: T,
}

impl<T: Copy> Rect<T> {
    pub fn new(x: T, y: T, w: T, h: T) -> Rect<T> {
        Rect {
            x: x, y: y, w: w, h: h,
        }
    }
    pub fn new_s(x: T, y: T, size: Size<T>) -> Rect<T> {
        Rect {
            x: x, y: y,
            w: size.w, h: size.h,
        }
    }
    pub fn new_p(point: Point<T>, w: T, h: T) -> Rect<T> {
        Rect {
            x: point.x, y: point.y,
            w: w, h: h,
        }
    }
    pub fn new_ps(point: Point<T>, size: Size<T>) -> Rect<T> {
        Rect {
            x: point.x, y: point.y,
            w: size.w, h: size.h,
        }
    }

    pub fn point(&self) -> Point<T> {
        Point {
            x: self.x, y: self.y,
        }
    }
    pub fn size(&self) -> Size<T> {
        Size {
            w: self.w, h: self.h,
        }
    }
}


#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Margins<T : Copy> {
    pub l: T,
    pub r: T,
    pub t: T,
    pub b: T,
}

impl<T: Copy> Margins<T> {
    pub fn new (l: T, r: T, t: T, b: T) -> Margins<T> {
        Margins {
            l: l, r: r, t: t, b: b,
        }
    }
}
