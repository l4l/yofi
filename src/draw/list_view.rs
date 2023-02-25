use std::marker::PhantomData;

use oneshot::Sender;
use raqote::{AntialiasMode, DrawOptions, Image, Point};

use super::{DrawTarget, Drawable, Space};
use crate::font::{Font, FontBackend, FontColor};
use crate::state::ContinuousMatch;
use crate::style::Margin;
use crate::{Color, ListViewInfo};
use unicode_segmentation::UnicodeSegmentation;

pub struct Params {
    pub font: Font,
    pub font_size: u16,
    pub font_color: Color,
    pub selected_font_color: Color,
    pub match_color: Option<Color>,
    pub icon_size: Option<u16>,
    pub fallback_icon: Option<crate::icon::Icon>,
    pub margin: Margin,
    pub hide_actions: bool,
    pub action_left_margin: f32,
    pub item_spacing: f32,
    pub icon_spacing: f32,
}

pub struct ListItem<'a> {
    pub name: &'a str,
    pub subname: Option<&'a str>,
    pub icon: Option<Image<'a>>,
    pub match_mask: Option<ContinuousMatch<'a>>,
}

pub struct ListView<'a, It> {
    items: It,
    skip_offset: usize,
    selected_item: usize,
    info_channel: Sender<ListViewInfo>,
    params: &'a Params,
    _tparam: PhantomData<&'a ()>,
}

impl<'a, It> ListView<'a, It> {
    pub fn new(
        items: It,
        skip_offset: usize,
        selected_item: usize,
        info_channel: Sender<ListViewInfo>,
        params: &'a Params,
    ) -> Self {
        Self {
            items,
            skip_offset,
            selected_item,
            info_channel,
            params,
            _tparam: PhantomData,
        }
    }
}

impl<'a, It> Drawable for ListView<'a, It>
where
    It: Iterator<Item = ListItem<'a>>,
{
    fn draw(self, dt: &mut DrawTarget<'_>, scale: u16, space: Space, point: Point) -> Space {
        let margin = &self.params.margin * f32::from(scale);
        let item_spacing = self.params.item_spacing * f32::from(scale);
        let icon_size = self.params.icon_size.unwrap_or(0) * scale;
        let icon_spacing = self.params.icon_spacing * f32::from(scale);

        let icon_size_f32 = f32::from(icon_size);
        let font_size = f32::from(self.params.font_size * scale);
        let top_offset = point.y + margin.top + (icon_size_f32 - font_size).max(0.) / 2.;
        let entry_height = font_size.max(icon_size_f32);

        let mut iter = self.items.peekable();

        let hide_actions = self.params.hide_actions;
        // For now either all items has subname or none.
        let has_subname = iter
            .peek()
            .map(|e| e.subname.is_some() && !hide_actions)
            .unwrap_or(false);

        let displayed_items = ((space.height - margin.top - margin.bottom + item_spacing)
            / (entry_height + item_spacing)) as usize
            - has_subname as usize;

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

        let mut relative_height = 0;

        for (i, item) in iter.skip(skip_offset).enumerate().take(displayed_items) {
            let relative_offset = (i as f32 + (i > selected_item && has_subname) as i32 as f32)
                * (entry_height + item_spacing);
            let x_offset = point.x + margin.left;
            let y_offset = top_offset + relative_offset;

            relative_height = y_offset as u32 + 5;

            let fallback_icon = self
                .params
                .fallback_icon
                .as_ref()
                .and_then(|i| i.as_image());
            if let Some(icon) = item.icon.as_ref().or(fallback_icon.as_ref()) {
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
            let end_pos = Point::new(dt.width() as f32 - self.params.margin.right, y_offset);

            let color = if i == selected_item {
                self.params.selected_font_color
            } else {
                self.params.font_color
            }
            .as_source();

            let antialias = AntialiasMode::Gray;
            let draw_opts = DrawOptions {
                antialias,
                ..DrawOptions::new()
            };

            let color = if let (Some(match_color), Some(match_mask)) =
                (self.params.match_color, item.match_mask)
            {
                let mut special_color =
                    vec![color; UnicodeSegmentation::graphemes(item.name, true).count()];

                let match_color = match_color.as_source();
                let special_len = special_color.len();
                let mut last_idx = 0; // exclusive

                match_mask.for_each(|m| {
                    let unmatch_range = last_idx..m.start();
                    if !unmatch_range.is_empty() {
                        special_color[unmatch_range].fill(color);
                    }

                    let match_range = m.start()..(m.start() + m.len()).min(special_len);
                    last_idx = match_range.end;
                    if !match_range.is_empty() {
                        special_color[match_range].fill(match_color);
                    }
                });

                FontColor::Multiple(special_color)
            } else {
                FontColor::Single(color)
            };

            let font = &self.params.font;
            font.draw(dt, item.name, font_size, pos, end_pos, color, &draw_opts);

            if i == selected_item && has_subname {
                if let Some(subname) = item.subname {
                    font.draw(
                        dt,
                        subname,
                        font_size,
                        Point::new(
                            pos.x + self.params.action_left_margin,
                            pos.y + entry_height + item_spacing,
                        ),
                        Point::new(end_pos.x, pos.y + entry_height + item_spacing),
                        FontColor::Single(self.params.font_color.as_source()),
                        &draw_opts,
                    );
                }
            }
        }

        self.info_channel
            .send(ListViewInfo {
                new_skip: skip_offset,
                new_height: relative_height,
            })
            .unwrap();

        space
    }
}
