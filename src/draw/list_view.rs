use std::marker::PhantomData;
use std::ops::Range;

use bit_vec::BitVec;
use font_kit::loaders::freetype::Font;
use itertools::Itertools;
use oneshot::Sender;
use raqote::{AntialiasMode, DrawOptions, DrawTarget, Image, Point, SolidSource, Source};

use super::{Drawable, Space};
use crate::style::Margin;

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
    pub name: &'a str,
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
    fn draw(self, dt: &mut DrawTarget, scale: u16, space: Space, point: Point) -> Space {
        let margin = self.params.margin * f32::from(scale);
        let item_spacing = self.params.item_spacing * f32::from(scale);
        let icon_size = self.params.icon_size * scale;
        let icon_spacing = self.params.icon_spacing * f32::from(scale);

        let top_offset = point.y + margin.top;
        let font_size = f32::from(self.params.font_size * scale);
        let icon_size_f32 = f32::from(icon_size);
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
            let y_offset = top_offset + relative_offset + entry_height;

            let fallback_icon = self.params.fallback_icon.as_ref().map(|i| i.as_image());
            if let Some(icon) = item.icon.as_ref().or_else(|| fallback_icon.as_ref()) {
                if icon.width == icon.height && icon.height == i32::from(icon_size) {
                    dt.draw_image_at(
                        x_offset,
                        y_offset - icon_size_f32,
                        &icon,
                        &DrawOptions::default(),
                    );
                } else {
                    dt.draw_image_with_size_at(
                        icon_size_f32,
                        icon_size_f32,
                        x_offset,
                        y_offset - icon_size_f32,
                        &icon,
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

            fn substr<'a, 'b: 'a>(x: &'b str, r: &Range<usize>) -> &'a str {
                debug_assert!(r.end <= x.chars().count());
                let start = x
                    .char_indices()
                    .nth(r.start)
                    .map(|x| x.0)
                    .unwrap_or_else(|| x.len());
                let end = x
                    .char_indices()
                    .nth(r.end - 1)
                    .map(|l| l.0 + l.1.len_utf8())
                    .unwrap_or_else(|| x.len());
                &x[start..end]
            }

            let antialias = AntialiasMode::Gray;
            let draw_opts = DrawOptions {
                antialias,
                ..DrawOptions::new()
            };

            if let Some(match_color) = self.params.match_color {
                let font = &self.params.font;
                macro_rules! draw_substr {
                    ($range:expr, $pos:expr, $color:expr) => {{
                        let s = substr(item.name, $range);
                        let measured = dt.measure_text(&font, font_size, s, antialias).unwrap();

                        dt.draw_text(
                            &font,
                            font_size,
                            s,
                            $pos,
                            &Source::Solid($color),
                            &draw_opts,
                        );
                        Point::new(
                            $pos.x + (measured.size.width + measured.min_x()) as f32,
                            $pos.y,
                        )
                    }};
                }

                let (pos, idx) = match_ranges
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
                    .fold((pos, 0), |(pos, _), (is_matched, range)| {
                        let color = if is_matched { match_color } else { color };

                        (draw_substr!(&range, pos, color), range.end)
                    });

                let tail_str = substr(item.name, &(idx..item.name.chars().count()));
                let color = Source::Solid(color);
                dt.draw_text(&font, font_size, tail_str, pos, &color, &draw_opts);
            } else {
                dt.draw_text(
                    &self.params.font,
                    font_size,
                    item.name,
                    pos,
                    &Source::Solid(color),
                    &draw_opts,
                );
            }
        }

        space
    }
}
