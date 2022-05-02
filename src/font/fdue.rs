use std::cell::RefCell;
use std::collections::BinaryHeap;
use std::path::Path;

use anyhow::Context;
use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle, VerticalAlign};
use levenshtein::levenshtein;
use once_cell::sync::Lazy;
use raqote::{DrawOptions, Point, SolidSource};
use rust_fontconfig::{FcFontCache, FcFontPath, FcPattern};

use super::{DrawTarget, FontBackend, FontColor, Result};

static FONTCONFIG_CACHE: Lazy<FcFontCache> = Lazy::new(FcFontCache::build);
const BUF_SIZE: usize = 256 * 256;

pub struct Font {
    inner: fontdue::Font,
    // Layout in fontdue uses allocations, so we're reusing it for reduce memory allocations
    layout: RefCell<Layout>,
    // Move buffer to heap, because it is very big for stack; only one allocation happens
    buffer: RefCell<Vec<u32>>,
}

impl Font {
    fn with_font(inner: fontdue::Font) -> Self {
        Self {
            inner,
            layout: RefCell::new(Layout::new(CoordinateSystem::PositiveYDown)),
            buffer: RefCell::new(vec![0; BUF_SIZE]),
        }
    }
}

#[derive(Eq)]
struct FuzzyResult<'a> {
    text: &'a str,
    distance: usize,
}

impl<'a> PartialEq for FuzzyResult<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}

impl<'a> Ord for FuzzyResult<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.distance.cmp(&other.distance)
    }
}

impl<'a> PartialOrd for FuzzyResult<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.distance.partial_cmp(&other.distance)
    }
}

impl Font {
    fn from_fc_path(font_search: &FcFontPath) -> Result<Self> {
        let bytes = std::fs::read(font_search.path.as_str()).context("font read")?;
        let inner = fontdue::Font::from_bytes(
            bytes,
            fontdue::FontSettings {
                collection_index: font_search.font_index as u32,
                ..Default::default()
            },
        )
        .map_err(|e| anyhow::anyhow!("{}", e))?;

        Ok(Font::with_font(inner))
    }

    fn try_find_best_font(name: &str) -> Vec<String> {
        const COUNT_MATCHES: usize = 5;

        FONTCONFIG_CACHE
            .list()
            .keys()
            .filter_map(|font| {
                let text = font.name.as_ref()?.as_str();
                Some(FuzzyResult {
                    text,
                    distance: levenshtein(name, text),
                })
            })
            .collect::<BinaryHeap<FuzzyResult>>()
            .into_sorted_vec()
            .into_iter()
            .take(COUNT_MATCHES)
            .map(|v| v.text.to_owned())
            .collect()
    }
}

impl FontBackend for Font {
    fn default() -> Self {
        FONTCONFIG_CACHE
            .query(&FcPattern::default())
            .map(Font::from_fc_path)
            .unwrap()
            .unwrap()
    }

    fn font_by_name(name: &str) -> Result<Self> {
        FONTCONFIG_CACHE
            .query(&FcPattern {
                name: Some(name.to_string()),
                ..Default::default()
            })
            .map(Font::from_fc_path)
            .ok_or_else(|| {
                let matching = Font::try_find_best_font(name);
                log::info!("The font {} could not be found.", name);
                if !matching.is_empty() {
                    use itertools::Itertools;
                    log::info!("Best matches:\n\t{}\n", matching.into_iter().format("\n\t"));
                }
                anyhow::anyhow!("cannot find font")
            })?
    }

    fn font_by_path(path: &Path) -> Result<Self> {
        Font::from_fc_path(&FcFontPath {
            path: path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid path"))?
                .to_owned(),
            font_index: 0,
        })
    }

    fn draw(
        &self,
        dt: &mut DrawTarget,
        text: &str,
        font_size: f32,
        start_pos: Point,
        color: FontColor,
        opts: &DrawOptions,
    ) {
        let mut buf = self.buffer.borrow_mut();
        let mut layout = self.layout.borrow_mut();

        layout.reset(&LayoutSettings {
            x: start_pos.x,
            y: start_pos.y,
            max_height: Some(font_size),
            vertical_align: VerticalAlign::Middle,
            ..LayoutSettings::default()
        });

        layout.append(&[&self.inner], &TextStyle::new(text, font_size, 0));

        for (n, g) in layout.glyphs().iter().enumerate() {
            let (_, b) = self.inner.rasterize_config(g.key);

            assert!(g.width * g.height <= BUF_SIZE);
            let width = g.width as i32;
            let height = g.height as i32;

            let color = match color {
                FontColor::Single(color) => color,
                FontColor::Multiple(ref colors) => colors[n],
            };

            for (i, x) in b.into_iter().enumerate() {
                let src = SolidSource::from_unpremultiplied_argb(
                    (u32::from(x) * u32::from(color.a) / 255) as u8,
                    color.r,
                    color.g,
                    color.b,
                );
                buf[i] = (u32::from(src.a) << 24)
                    | (u32::from(src.r) << 16)
                    | (u32::from(src.g) << 8)
                    | u32::from(src.b);
            }

            let img = raqote::Image {
                width,
                height,
                data: &buf[..],
            };

            dt.draw_image_with_size_at(g.width as f32, g.height as f32, g.x, g.y, &img, opts);
        }
    }
}
