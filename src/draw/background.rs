use raqote::{Point, SolidSource};

use super::{Border, DrawTarget, Drawable, RoundedRect, Space};
use crate::{style::Radius, Color};

pub struct Params {
    pub width: u32,
    pub height: u32,
    pub color: Color,
    pub radius: Radius,
    pub border: Option<(Color, f32)>,
}

pub struct Background {
    rect: RoundedRect,
}

impl Background {
    pub fn new(params: &Params) -> Self {
        let color = params.color;
        let radius = params.radius.clone();
        let border = if let Some((c, w)) = params.border {
            Some(Border::new(c, w))
        } else {
            None
        };
        let rect = RoundedRect::new(radius, color).with_border(border);

        Self { rect }
    }
}

impl Drawable for Background {
    fn draw(self, dt: &mut DrawTarget<'_>, scale: u16, space: Space, start_point: Point) -> Space {
        // Clear the draw target to avoid artefacts for scales > 1 in the corners
        dt.clear(SolidSource {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        });

        self.rect.draw(dt, scale, space, start_point);

        Space {
            width: 0.,
            height: 0.,
        }
    }
}
