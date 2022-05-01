use raqote::Point;

use super::{DrawTarget, Drawable, Space};
use crate::Color;

pub struct Params {
    pub color: Color,
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
    fn draw(self, dt: &mut DrawTarget<'_>, _: u16, _: Space, _: Point) -> Space {
        dt.clear(self.params.color.as_source());

        Space {
            width: 0.,
            height: 0.,
        }
    }
}
