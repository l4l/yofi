use std::f32::consts;

use font_kit::loaders::freetype::Font;
use raqote::{DrawOptions, DrawTarget, PathBuilder, Point, SolidSource, Source};

use super::{Drawable, Space};
use crate::style::{Margin, Padding};

pub struct Params {
    pub font: Font,
    pub font_size: u16,
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

        let font_size = f32::from(self.params.font_size);

        let border_diameter = self.params.padding.top + font_size + self.params.padding.bottom;
        let border_radius = border_diameter / 2.0;

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
            font_size / /*empirical magic:*/ 3.0 + y_center,
        );
        dt.draw_text(
            &self.params.font,
            font_size,
            self.text,
            pos,
            &Source::Solid(self.params.font_color),
            &DrawOptions::new(),
        );
        // TODO: use padding.right for text wrapping/clipping

        Space {
            width: space.width,
            height: point.y + self.params.margin.top + border_diameter + self.params.margin.bottom,
        }
    }
}
