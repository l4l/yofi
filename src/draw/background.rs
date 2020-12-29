use raqote::{DrawTarget, Point, SolidSource};

use super::{Drawable, Space};

pub struct Params {
    pub color: SolidSource,
}

pub struct Background {
    params: Params,
}

impl Background {
    pub fn new(params: Params) -> Self {
        Self { params }
    }
}

impl<'a> Drawable for Background {
    fn draw(self, dt: &mut DrawTarget, _: u16, _: Space, _: Point) -> Space {
        dt.clear(self.params.color);

        Space {
            width: 0.,
            height: 0.,
        }
    }
}
