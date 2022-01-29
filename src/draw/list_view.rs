use std::marker::PhantomData;

use bit_vec::BitVec;
use itertools::Itertools;
use oneshot::Sender;
use raqote::{AntialiasMode, DrawOptions, DrawTarget, Image, Point, SolidSource};

use super::{Drawable, Space};
use crate::font::{Font, FontBackend, FontColor};
use crate::style::Margin;
use unicode_segmentation::UnicodeSegmentation;

pub struct Params {
    pub font: Font,
    pub font_size: u16,
    pub font_color: SolidSource,
    pub selected_font_color: SolidSource,
    pub match_color: Option<SolidSource>,
    pub icon_size: u16,
    pub fallback_icon: Option<crate::icon::Icon>,
    pub margin: Margin,
    pub item_spacing: f32,
    pub icon_spacing: f32,
}

pub struct ListItem<'a> {
    pub name: String,
    pub icon: Option<Image<'a>>,
    pub match_mask: Option<&'a BitVec>,
}

pub struct ListView<'a, It> {
    items: It,
    skip_offset: usize,
    selected_item: usize,
    new_skip: Sender<usize>,
    params: Params,
    _tparam: PhantomData<&'a ()>,
}

impl<It> ListView<'_, It> {
    pub fn new(
        items: It,
        skip_offset: usize,
        selected_item: usize,
        new_skip: Sender<usize>,
        params: Params,
    ) -> Self {
        Self {
            items,
            skip_offset,
            selected_item,
            new_skip,
            params,
            _tparam: PhantomData,
        }
    }
}

impl<'a, It> Drawable for ListView<'a, It>
where
    It: Iterator<Item = ListItem<'a>>,
{
    fn draw(self, mut dt: &mut DrawTarget, scale: u16, space: Space, point: Point) -> Space {
        let margin = self.params.margin * f32::from(scale);
        let item_spacing = self.params.item_spacing * f32::from(scale);
        let icon_size = self.params.icon_size * scale;
        let icon_spacing = self.params.icon_spacing * f32::from(scale);

        let icon_size_f32 = f32::from(icon_size);
        let font_size = f32::from(self.params.font_size * scale);
        let top_offset = point.y + margin.top + (icon_size_f32 - font_size).max(0.) / 2.;
        let entry_height = font_size.max(icon_size_f32);

        let displayed_items = ((space.height - margin.top - margin.bottom + item_spacing)
            / (entry_height + item_spacing)) as usize;

        let max_offset = self.skip_offset + displayed_items;
        let (selected_item, skip_offset) = if self.selected_item < self.skip_offset {
            (0, self.selected_item)
        } else if max_offset <= self.selected_item {
            (
                displayed_items - 1,
                self.skip_offset + (self.selected_item - max_offset) + 1,
            )
        } else {
            (self.selected_item - self.skip_offset, self.skip_offset)
        };

        self.new_skip.send(skip_offset).unwrap();

        for (i, item) in self
            .items
            .skip(skip_offset)
            .enumerate()
            .take(displayed_items)
        {
            let relative_offset = (i as f32) * (entry_height + item_spacing);
            let x_offset = point.x + margin.left;
            let y_offset = top_offset + relative_offset;

            let fallback_icon = self.params.fallback_icon.as_ref().map(|i| i.as_image());
            if let Some(icon) = item.icon.as_ref().or_else(|| fallback_icon.as_ref()) {
                if icon.width == icon.height && icon.height == i32::from(icon_size) {
                    dt.draw_image_at(
                        x_offset,
                        y_offset + (font_size - icon_size_f32) / 2.,
                        icon,
                        &DrawOptions::default(),
                    );
                } else {
                    dt.draw_image_with_size_at(
                        icon_size_f32,
                        icon_size_f32,
                        x_offset,
                        y_offset + (font_size - icon_size_f32) / 2.,
                        icon,
                        &DrawOptions::default(),
                    );
                }
            }

            let pos = Point::new(x_offset + icon_size_f32 + icon_spacing, y_offset);
            let color = if i == selected_item {
                self.params.selected_font_color
            } else {
                self.params.font_color
            };

            let empty = BitVec::new();
            let match_ranges = item.match_mask.unwrap_or(&empty);

            let antialias = AntialiasMode::Gray;
            let draw_opts = DrawOptions {
                antialias,
                ..DrawOptions::new()
            };

            let color = if let Some(match_color) = self.params.match_color {
                let mut special_color =
                    vec![color; UnicodeSegmentation::graphemes(item.name.as_str(), true).count()];

                let special_len = special_color.len();

                match_ranges
                    .iter()
                    .group_by(|x| *x)
                    .into_iter()
                    .enumerate()
                    .scan(0, |start, (_, group)| {
                        let count = group.1.count();
                        let s = *start;
                        let range = s..s + count;
                        *start += count;
                        Some((group.0, range))
                    })
                    .for_each(|(is_matched, range)| {
                        let color = if is_matched { match_color } else { color };

                        if range.start < special_len {
                            special_color[range.start..range.end.min(special_len)].fill(color);
                        }
                    });

                FontColor::Multiple(special_color)
            } else {
                FontColor::Single(color)
            };

            let font = &self.params.font;
            font.draw(
                &mut dt,
                item.name.as_str(),
                font_size,
                pos,
                color,
                &draw_opts,
            );
        }

        space
    }
}
