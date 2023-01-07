use std::f32::consts;

use raqote::{DrawOptions, PathBuilder, Point, Source};

use super::{DrawTarget, Drawable, Space};
use crate::{style::Radius, Color};

pub struct Params {
    pub width: u32,
    pub height: u32,
    pub color: Color,
    pub radius: Radius,
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
        let mut pb = PathBuilder::new();

        let width = self.params.width as f32;
        let height = self.params.height as f32;
        let Radius {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        } = self.params.radius;

        pb.arc(top_left, top_left, top_left, consts::PI, consts::FRAC_PI_2);
        pb.arc(
            width - top_right,
            top_right,
            top_right,
            3.0 * consts::FRAC_PI_2,
            consts::FRAC_PI_2,
        );
        pb.arc(
            width - bottom_right,
            height - bottom_right,
            bottom_right,
            2.0 * consts::PI,
            consts::FRAC_PI_2,
        );
        pb.arc(
            bottom_left,
            height - bottom_left,
            bottom_left,
            consts::FRAC_PI_2,
            consts::FRAC_PI_2,
        );
        let path = pb.finish();

        dt.fill(
            &path,
            &Source::Solid(self.params.color.as_source()),
            &DrawOptions::new(),
        );

        Space {
            width: 0.,
            height: 0.,
        }
    }
}
