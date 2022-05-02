use std::f32::consts;

use raqote::{DrawOptions, PathBuilder, Point, Source};

use super::{DrawTarget, Drawable, Space};
use crate::font::{Font, FontBackend, FontColor};
use crate::style::{Margin, Padding};
use crate::Color;

pub struct Params {
    pub font: Font,
    pub font_size: u16,
    pub bg_color: Color,
    pub font_color: Color,
    pub prompt_color: Color,
    pub prompt: Option<String>,
    pub password: bool,
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
    fn draw(self, dt: &mut DrawTarget<'_>, scale: u16, space: Space, point: Point) -> Space {
        let mut pb = PathBuilder::new();

        let font_size = f32::from(self.params.font_size * scale);

        let mut padding = self.params.padding * f32::from(scale);
        const PADDING_TOP: f32 = 2.0;
        const PADDING_BOTTOM: f32 = 5.0;
        padding.top += PADDING_TOP;
        padding.bottom += PADDING_BOTTOM;
        let margin = self.params.margin * f32::from(scale);

        let border_diameter = padding.top + font_size + padding.bottom;
        let border_radius = border_diameter / 2.0;

        let left_x_center = point.x + margin.left + border_radius;
        let y_center = point.y + margin.top + border_radius;

        pb.arc(
            left_x_center,
            y_center,
            border_radius,
            consts::FRAC_PI_2,
            consts::PI,
        );
        let right_x_center = (point.x + space.width - border_radius - margin.right)
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
            &Source::Solid(self.params.bg_color.as_source()),
            &DrawOptions::new(),
        );

        let pos = Point::new(
            left_x_center + padding.left,
            point.y + margin.top + padding.top,
        );

        let password_text = if self.params.password {
            Some("*".repeat(self.text.chars().count()))
        } else {
            None
        };

        let (color, text) = if self.text.is_empty() {
            (
                self.params.prompt_color,
                self.params.prompt.as_deref().unwrap_or_default(),
            )
        } else {
            let text = if let Some(password_text) = password_text.as_ref() {
                password_text.as_str()
            } else {
                self.text
            };
            (self.params.font_color, text)
        };

        self.params.font.draw(
            dt,
            text,
            font_size,
            pos,
            FontColor::Single(color.as_source()),
            &DrawOptions::new(),
        );

        // TODO: use padding.right for text wrapping/clipping

        Space {
            width: space.width,
            height: point.y + margin.top + border_diameter + margin.bottom,
        }
    }
}
