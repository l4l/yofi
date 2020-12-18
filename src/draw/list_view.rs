use std::marker::PhantomData;

use font_kit::loaders::freetype::Font;
use raqote::{DrawOptions, DrawTarget, Image, Point, SolidSource, Source};

use super::{Drawable, Space};
use crate::style::Margin;

const ENTRY_HEIGHT: f32 = 28.;

pub struct Params {
    pub font: Font,
    pub font_color: SolidSource,
    pub selected_font_color: SolidSource,
    pub icon_size: Option<u32>,
    pub fallback_icon: Option<crate::icon::Icon>,
    pub margin: Margin,
    pub item_spacing: f32,
    pub icon_spacing: f32,
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
        let top_offset = point.y + self.params.margin.top + 28.;
        for (i, item) in self.items.skip(skip).enumerate() {
            let relative_offset = (i as f32) * (ENTRY_HEIGHT + self.params.item_spacing);
            if relative_offset + self.params.margin.bottom + ENTRY_HEIGHT > space.height {
                break;
            }

            let x_offset = point.x + self.params.margin.left;
            let y_offset = top_offset + relative_offset;

            let fallback_icon = self.params.fallback_icon.as_ref().map(|i| i.as_image());
            if let Some(icon) = item.icon.as_ref().or_else(|| fallback_icon.as_ref()) {
                dt.draw_image_at(
                    x_offset,
                    y_offset - icon.height as f32,
                    &icon,
                    &DrawOptions::default(),
                );
            }

            let pos = Point::new(
                x_offset
                    + self.params.icon_size.map(|s| s as f32).unwrap_or(0.0)
                    + self.params.icon_spacing,
                y_offset,
            );
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
