use std::f32::consts;

use font_kit::loaders::freetype::Font;
use font_kit::source::SystemSource;
use raqote::{DrawOptions, DrawTarget, PathBuilder, Point, SolidSource, Source};

use super::{Drawable, Space};

const VERTICAL_MARGIN: f32 = 5.0;
const HORIZONTAL_MARGIN: f32 = 5.0;
const BORDER_RADIUS: f32 = 15.0;

pub struct InputText<'a> {
    text: &'a str,
    font: Font,
    font_bg_color: SolidSource,
    font_color: SolidSource,
}

impl<'a> InputText<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            font: FONT.with(Clone::clone),
            font_bg_color: SolidSource::from_unpremultiplied_argb(0x90, 0xcc, 0xcc, 0xcc),
            font_color: SolidSource::from_unpremultiplied_argb(0xff, 0, 0, 0),
        }
    }
}

std::thread_local! {
    static FONT: Font = SystemSource::new()
        .select_best_match(
            &[font_kit::family_name::FamilyName::SansSerif],
            &font_kit::properties::Properties::new(),
        )
        .unwrap()
        .load()
        .unwrap();
}

impl<'a> Drawable for InputText<'a> {
    fn draw(self, dt: &mut DrawTarget, space: Space, point: Point) -> Space {
        let mut pb = PathBuilder::new();

        let side_offset = BORDER_RADIUS + HORIZONTAL_MARGIN;
        let y_center = point.y + BORDER_RADIUS + VERTICAL_MARGIN;

        pb.arc(
            point.x + side_offset,
            y_center,
            BORDER_RADIUS,
            consts::FRAC_PI_2,
            consts::PI,
        );
        pb.arc(
            point.x + space.width - side_offset,
            y_center,
            BORDER_RADIUS,
            3.0 * consts::FRAC_PI_2,
            consts::PI,
        );
        let path = pb.finish();

        dt.fill(
            &path,
            &Source::Solid(self.font_bg_color),
            &DrawOptions::new(),
        );

        let pos = Point::new(point.x + BORDER_RADIUS + VERTICAL_MARGIN + 5.0, 28.);
        dt.draw_text(
            &self.font,
            24.,
            self.text,
            pos,
            &Source::Solid(self.font_color),
            &DrawOptions::new(),
        );

        Space {
            width: space.width,
            height: 2.0 * VERTICAL_MARGIN + 2.0 * BORDER_RADIUS,
        }
    }
}
