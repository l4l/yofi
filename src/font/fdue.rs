use std::cell::RefCell;

use anyhow::Context;
use once_cell::sync::Lazy;
use raqote::{AntialiasMode, DrawOptions, DrawTarget, Point, SolidSource};
use rust_fontconfig::{FcFontCache, FcFontPath, FcPattern, PatternMatch};

use super::{FontBackend, Result};

static FONTCONFIG: Lazy<FcFontCache> = Lazy::new(FcFontCache::build);
const BUF_SIZE: usize = 256 * 256;

pub struct Font {
    inner: fontdue::Font,
    buffer: RefCell<[u32; BUF_SIZE]>,
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

        Ok(Font {
            inner,
            buffer: RefCell::new([0; BUF_SIZE]),
        })
    }
}

impl FontBackend for Font {
    fn default() -> Self {
        FONTCONFIG
            .query(&FcPattern {
                monospace: PatternMatch::True,
                ..Default::default()
            })
            .map(Font::from_fc_path)
            .unwrap()
            .unwrap()
    }

    fn font_by_name(name: &str) -> Result<Self> {
        FONTCONFIG
            .query(&FcPattern {
                name: Some(name.to_string()),
                ..Default::default()
            })
            .map(Font::from_fc_path)
            .ok_or_else(|| anyhow::anyhow!("cannot find font"))?
    }

    fn draw(
        &self,
        dt: &mut DrawTarget,
        text: &str,
        font_size: f32,
        start_pos: Point,
        color: SolidSource,
        opts: &DrawOptions,
    ) {
        let mut buf = self.buffer.borrow_mut();
        let (mut x, y) = (start_pos.x, start_pos.y);
        for c in text.chars() {
            let (m, b) = self.inner.rasterize(c, font_size);
            assert!(m.width * m.height <= BUF_SIZE);
            let width = m.width as i32;
            let height = m.height as i32;

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

            dt.draw_image_with_size_at(
                m.width as f32,
                m.height as f32,
                x + m.xmin as f32,
                y + font_size - m.bounds.height - m.ymin as f32,
                &img,
                opts,
            );

            x += m.advance_width;
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
