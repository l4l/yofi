use std::cell::RefCell;
use std::collections::BinaryHeap;
use std::path::{Path, PathBuf};

use anyhow::Context;
use fontconfig::{Fontconfig, Pattern};
use fontdue::layout::{
    CoordinateSystem, Layout, LayoutSettings, TextStyle, VerticalAlign, WrapStyle,
};
use levenshtein::levenshtein;
use once_cell::sync::Lazy;
use raqote::{DrawOptions, Point, SolidSource};

use super::{DrawTarget, FontBackend, FontColor, Result};

static FONTCONFIG_CACHE: Lazy<Fontconfig> =
    Lazy::new(|| Fontconfig::new().expect("failed to initialize fontconfig"));
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
struct FuzzyResult {
    text: String,
    distance: usize,
}

impl PartialEq for FuzzyResult {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}

impl Ord for FuzzyResult {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.distance.cmp(&other.distance)
    }
}

impl PartialOrd for FuzzyResult {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Font {
    fn from_path(path: &Path, index: Option<u32>) -> Result<Self> {
        let bytes = std::fs::read(path).context("font read")?;
        let inner = fontdue::Font::from_bytes(
            bytes,
            index
                .map(|collection_index| fontdue::FontSettings {
                    collection_index,
                    ..Default::default()
                })
                .unwrap_or_default(),
        )
        .map_err(|e| anyhow::anyhow!("{}", e))?;

        Ok(Font::with_font(inner))
    }

    fn try_find_best_font(name: &str) -> Vec<String> {
        const COUNT_MATCHES: usize = 5;

        let pat = Pattern::new(&FONTCONFIG_CACHE);
        fontconfig::list_fonts(&pat, None)
            .iter()
            .filter_map(|pat| {
                let text = pat.name()?.to_string();
                Some(FuzzyResult {
                    distance: levenshtein(name, &text),
                    text,
                })
            })
            .collect::<BinaryHeap<FuzzyResult>>()
            .into_sorted_vec()
            .into_iter()
            .take(COUNT_MATCHES)
            .map(|r| r.text)
            .collect()
    }
}

fn index_to_u32(idx: i32) -> Option<u32> {
    if idx < 0 {
        None
    } else {
        Some(idx as u32)
    }
}

impl FontBackend for Font {
    fn default() -> Self {
        let pat = Pattern::new(&FONTCONFIG_CACHE);
        fontconfig::list_fonts(&pat, None)
            .iter()
            .find_map(|pat| {
                let path = std::path::Path::new(pat.filename()?);
                if !path.exists() {
                    return None;
                }
                let index = pat.face_index().and_then(index_to_u32);
                Font::from_path(path, index)
                    .map_err(|e| log::debug!("cannot load default font at {}: {e}", path.display()))
                    .ok()
            })
            .expect("cannot find any font")
    }

    fn font_by_name(name: &str) -> Result<Self> {
        let cache = &*FONTCONFIG_CACHE;

        // TODO: use Font.find after https://github.com/yeslogic/fontconfig-rs/pull/27
        fn find(
            fc: &Fontconfig,
            family: &str,
            style: Option<&str>,
        ) -> Option<(PathBuf, Option<i32>)> {
            let mut pat = Pattern::new(fc);
            let family = std::ffi::CString::new(family).ok()?;
            pat.add_string(fontconfig::FC_FAMILY, &family);

            if let Some(style) = style {
                let style = std::ffi::CString::new(style).ok()?;
                pat.add_string(fontconfig::FC_STYLE, &style);
            }

            let font_match = pat.font_match();

            font_match
                .filename()
                .map(|filename| (PathBuf::from(filename), font_match.face_index()))
        }

        let (path, index) = find(cache, name, None)
            .or_else(|| {
                let (name, style) = name.rsplit_once(' ')?;
                find(cache, name, Some(style))
            })
            .ok_or_else(|| {
                let matching = Font::try_find_best_font(name);
                log::info!("The font {} could not be found.", name);
                if !matching.is_empty() {
                    use itertools::Itertools;
                    log::info!("Best matches:\n\t{}\n", matching.into_iter().format("\n\t"));
                }
                anyhow::anyhow!("cannot find font")
            })?;

        Font::from_path(&path, index.and_then(index_to_u32))
    }

    fn font_by_path(path: &Path) -> Result<Self> {
        Font::from_path(path, None)
    }

    fn draw(
        &self,
        dt: &mut DrawTarget,
        mut text: &str,
        font_size: f32,
        start_pos: Point,
        end_pos: Point,
        color: FontColor,
        opts: &DrawOptions,
    ) {
        let mut buf = self.buffer.borrow_mut();
        let mut layout = self.layout.borrow_mut();

        layout.reset(&LayoutSettings {
            x: start_pos.x,
            y: start_pos.y,
            max_height: Some(font_size),
            max_width: Some(end_pos.x - start_pos.x),
            vertical_align: VerticalAlign::Middle,
            wrap_style: WrapStyle::Letter,
            ..LayoutSettings::default()
        });

        layout.append(&[&self.inner], &TextStyle::new(text, font_size, 0));

        let take_glyphs = match layout.lines() {
            Some(vec) => {
                // If layout return miltiple lines then we have text overflow, cut the text
                // and layout again
                match vec.get(1) {
                    Some(second_line) => second_line.glyph_start,
                    None => layout.glyphs().len(),
                }
            }
            None => layout.glyphs().len(),
        };

        if take_glyphs != layout.glyphs().len() {
            let overflow_text = "...";

            // Try place ... in end of cutted text. Check strange case if width of window is too small
            // even for overflow_text. No panic at all
            let glyph_offset = if take_glyphs > overflow_text.len() {
                take_glyphs - overflow_text.len()
            } else {
                take_glyphs
            };

            text = &text[0..layout.glyphs().get(glyph_offset).unwrap().byte_offset];
            layout.clear();
            layout.append(&[&self.inner], &TextStyle::new(text, font_size, 0));

            if glyph_offset != take_glyphs {
                layout.append(&[&self.inner], &TextStyle::new(overflow_text, font_size, 0));
            }
        }

        for (n, g) in layout.glyphs().iter().take(take_glyphs).enumerate() {
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
