use std::f32::consts;

use font_kit::loaders::freetype::Font;
use raqote::{DrawOptions, DrawTarget, PathBuilder, Point, SolidSource, Source};

use super::{Drawable, Space};
use crate::style::{Margin, Padding};

const FONT_SIZE: f32 = 24.0;
const BORDER_RADIUS_BASE: f32 = FONT_SIZE / 2.0;

pub struct Params {
    pub font: Font,
    pub bg_color: SolidSource,
    pub font_color: SolidSource,
    pub margin: Margin,
    pub padding: Padding,
}

pub struct InputText<'a> {
    text: &'a str,
    params: Params,
}

impl<'a> InputText<'a> {
    pub fn new(text: &'a str, params: Params) -> Self {
        Self { text, params }
    }
}

impl<'a> Drawable for InputText<'a> {
    fn draw(self, dt: &mut DrawTarget, space: Space, point: Point) -> Space {
        let mut pb = PathBuilder::new();

        let border_radius =
            self.params.padding.top + BORDER_RADIUS_BASE + self.params.padding.bottom;

        let left_x_center = point.x + self.params.margin.left + border_radius;
        let y_center = point.y + self.params.margin.top + border_radius;

        pb.arc(
            left_x_center,
            y_center,
            border_radius,
            consts::FRAC_PI_2,
            consts::PI,
        );
        let right_x_center = (point.x + space.width - border_radius - self.params.margin.right)
            .max(left_x_center - border_radius);
        pb.arc(
            right_x_center,
            y_center,
            border_radius,
            3.0 * consts::FRAC_PI_2,
            consts::PI,
        );
        let path = pb.finish();

        dt.fill(
            &path,
            &Source::Solid(self.params.bg_color),
            &DrawOptions::new(),
        );

        let pos = Point::new(
            left_x_center + self.params.padding.left,
            FONT_SIZE / /*empirical magic:*/ 3.0 + point.y + self.params.margin.top + border_radius,
        );
        dt.draw_text(
            &self.params.font,
            FONT_SIZE,
            self.text,
            pos,
            &Source::Solid(self.params.font_color),
            &DrawOptions::new(),
        );
        // TODO: use padding.right for text wrapping/clipping

        Space {
            width: space.width,
            height: y_center + border_radius + self.params.margin.bottom,
        }
    }
}
