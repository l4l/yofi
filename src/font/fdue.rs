use std::cell::RefCell;

use anyhow::Context;
use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle, VerticalAlign};
use once_cell::sync::Lazy;
use raqote::{AntialiasMode, DrawOptions, DrawTarget, Point, SolidSource};
use rust_fontconfig::{FcFontCache, FcFontPath, FcPattern};
use std::collections::BinaryHeap;
use sublime_fuzzy::{best_match, format_simple, Match};

use super::{FontBackend, FontColor, Result};

static FONTCONFIG: Lazy<FontConfig> = Lazy::new(FontConfig::new);
const BUF_SIZE: usize = 256 * 256;

pub struct Font {
    inner: fontdue::Font,
}

// Because Font is re-created on every `draw` call method we cached dynamic allocations
struct FontConfig {
    cache: FcFontCache,
    // Layout in fontdue uses allocations, so we're reusing it for reduce memory allocations
    layout: RefCell<Layout>,
    // Move buffer to heap, because it is very big for stack; only one allocation happens
    buffer: RefCell<Vec<u32>>,
}

impl FontConfig {
    fn new() -> Self {
        Self {
            cache: FcFontCache::build(),
            layout: RefCell::new(Layout::new(CoordinateSystem::PositiveYDown)),
            buffer: RefCell::new(vec![0; BUF_SIZE]),
        }
    }
}

//SAFETY: We do not use multiple threads, so it never happens that the & is posted to another thread
unsafe impl Sync for FontConfig {}

#[derive(Eq)]
struct FuzzyResult<'a> {
    text: &'a str,
    match_: Match,
}

impl<'a> PartialEq for FuzzyResult<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.match_.score() == other.match_.score()
    }
}

impl<'a> Ord for FuzzyResult<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.match_.score().cmp(&other.match_.score())
    }
}

impl<'a> PartialOrd for FuzzyResult<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.match_.score().partial_cmp(&other.match_.score())
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

        Ok(Font { inner })
    }

    fn try_find_best_font(name: &str) -> Vec<String> {
        const COUNT_MATCHES: usize = 5;

        FONTCONFIG
            .cache
            .list()
            .keys()
            .filter(|v| v.name.is_some())
            .map(|font| {
                let current = font.name.as_ref().unwrap().as_str();
                if let Some(matching) = best_match(name, current) {
                    return Some(FuzzyResult {
                        text: current,
                        match_: matching,
                    });
                }
                None
            })
            .filter(|v| v.is_some())
            .map(|v| v.unwrap())
            .collect::<BinaryHeap<FuzzyResult>>()
            .into_iter()
            .take(COUNT_MATCHES)
            .map(|v| format_simple(&v.match_, v.text, "\x1b[1m", "\x1b[0m"))
            .collect()
    }
}

impl FontBackend for Font {
    fn default() -> Self {
        FONTCONFIG
            .cache
            .query(&FcPattern::default())
            .map(Font::from_fc_path)
            .unwrap()
            .unwrap()
    }

    fn font_by_name(name: &str) -> Result<Self> {
        FONTCONFIG
            .cache
            .query(&FcPattern {
                name: Some(name.to_string()),
                ..Default::default()
            })
            .map(Font::from_fc_path)
            .ok_or_else(|| {
                let matching = Font::try_find_best_font(name);
                println!("The font could not be found.");
                if matching.len() > 0 {
                    println!("Best matches:\n");
                    matching.into_iter().for_each(|res| println!("{}", res));
                    println!();
                }
                anyhow::anyhow!("cannot find font")
            })?
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
        let mut buf = FONTCONFIG.buffer.borrow_mut();
        let mut layout = FONTCONFIG.layout.borrow_mut();

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

    fn measure_text_width(
        &self,
        _: &DrawTarget,
        font_size: f32,
        text: &str,
        _: AntialiasMode,
    ) -> f32 {
        text.chars()
            .map(|c| self.inner.metrics(c, font_size).advance_width)
            .sum::<f32>()
    }
}
