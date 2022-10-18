use raqote::Point;

use super::{DrawTarget, Drawable, Space};
use crate::Color;

pub struct Params {
    pub color: Color,
}

pub struct Background<'a> {
    params: &'a Params,
}

impl<'a> Background<'a> {
    pub fn new(params: &'a Params) -> Self {
        Self { params }
    }
}

impl Drawable for Background<'_> {
    fn draw(self, dt: &mut DrawTarget<'_>, _: u16, _: Space, _: Point) -> Space {
        dt.clear(self.params.color.as_source());

        Space {
            width: 0.,
            height: 0.,
        }
    }
}
