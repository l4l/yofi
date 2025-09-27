use raqote::{DrawOptions, Point};

use super::{DrawTarget, Drawable, RoundedRect, Space};
use crate::font::{Font, FontBackend, FontColor};
use crate::style::{Margin, Padding, Radius};
use crate::Color;

pub struct Params<'a> {
    pub hide: bool,
    pub font: Font,
    pub font_size: u16,
    pub bg_color: Color,
    pub font_color: Color,
    pub prompt_color: Color,
    pub prompt: Option<&'a str>,
    pub password: bool,
    pub margin: Margin,
    pub padding: Padding,
    pub radius: Radius,
}

pub struct InputText<'a> {
    text: &'a str,
    params: &'a Params<'a>,
    rect: RoundedRect,
}

impl<'a> InputText<'a> {
    pub fn new(text: &'a str, params: &'a Params<'a>) -> Self {
        let color = params.bg_color;
        let radius = params.radius.clone();

        Self {
            text,
            params,
            rect: RoundedRect::new(radius, color),
        }
    }
}

impl<'a> Drawable for InputText<'a> {
    fn draw(self, dt: &mut DrawTarget<'_>, scale: u16, space: Space, point: Point) -> Space {
        if self.params.hide {
            return Space {
                width: 0.,
                height: 0.,
            };
        }

        let font_size = f32::from(self.params.font_size * scale);

        let mut padding = &self.params.padding * f32::from(scale);
        const PADDING_TOP: f32 = 2.0;
        const PADDING_BOTTOM: f32 = 5.0;
        padding.top += PADDING_TOP;
        padding.bottom += PADDING_BOTTOM;
        let margin = &self.params.margin * f32::from(scale);

        let rect_width = space.width - margin.left - margin.right;
        let rect_height = padding.top + font_size + padding.bottom;
        let rect_space = Space {
            width: rect_width,
            height: rect_height,
        };
        let rect_point = Point::new(point.x + margin.left, point.y + margin.top);

        self.rect.draw(dt, scale, rect_space, rect_point);

        padding.left += (rect_height / 2.0)
            .min(self.params.radius.top_left)
            .min(self.params.radius.top_right);

        let pos = Point::new(rect_point.x + padding.left, rect_point.y + padding.top);
        let end_pos = Point::new(
            dt.width() as f32 - self.params.padding.right - self.params.margin.right,
            pos.y,
        );

        let password_text = if self.params.password {
            Some("*".repeat(self.text.chars().count()))
        } else {
            None
        };

        let (color, text) = if self.text.is_empty() {
            (
                self.params.prompt_color,
                self.params.prompt.unwrap_or_default(),
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
            end_pos,
            FontColor::Single(color.as_source()),
            &DrawOptions::new(),
        );

        // TODO: use padding.right for text wrapping/clipping

        Space {
            width: space.width,
            height: margin.top + rect_height + margin.bottom,
        }
    }
}
