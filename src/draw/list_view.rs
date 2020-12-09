use std::marker::PhantomData;

use font_kit::loaders::freetype::Font;
use raqote::{DrawOptions, DrawTarget, Image, Point, SolidSource, Source};

use super::{Drawable, Space};

const ENTRY_HEIGHT: f32 = 28.;

pub struct Params {
    pub font: Font,
    pub font_color: SolidSource,
    pub selected_font_color: SolidSource,
}

pub struct ListItem<'a> {
    pub name: &'a str,
    pub icon: Option<Image<'a>>,
}

pub struct ListView<'a, It> {
    items: It,
    selected_item: usize,
    params: Params,
    _tparam: PhantomData<&'a ()>,
}

impl<It> ListView<'_, It> {
    pub fn new(items: It, selected_item: usize, params: Params) -> Self {
        Self {
            items,
            selected_item,
            params,
            _tparam: PhantomData,
        }
    }
}

impl<'a, It> Drawable for ListView<'a, It>
where
    It: Iterator<Item = ListItem<'a>>,
{
    fn draw(self, dt: &mut DrawTarget, space: Space, point: Point) -> Space {
        let skip = self.selected_item.saturating_sub(3);
        let top_offset = point.y + 28.;
        for (i, item) in self.items.skip(skip).enumerate() {
            let relative_offset = (i as f32) * ENTRY_HEIGHT;
            if relative_offset + ENTRY_HEIGHT > space.height {
                break;
            }

            let x_offset = point.x + 10.;
            let y_offset = top_offset + relative_offset;

            let x_offset = if let Some(icon) = item.icon.as_ref() {
                let opt = raqote::DrawOptions::default();
                dt.draw_image_at(x_offset, y_offset - icon.height as f32, &icon, &opt);

                x_offset + icon.width as f32 + 3.0
            } else {
                x_offset
            };

            let pos = Point::new(x_offset, y_offset);
            let color = if i + skip == self.selected_item {
                self.params.selected_font_color
            } else {
                self.params.font_color
            };
            dt.draw_text(
                &self.params.font,
                24.,
                item.name,
                pos,
                &Source::Solid(color),
                &DrawOptions::new(),
            );
        }

        space
    }
}
