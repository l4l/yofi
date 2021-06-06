use anyhow::Context;
use once_cell::sync::OnceCell;
use raqote::{AntialiasMode, DrawOptions, DrawTarget, Point, SolidSource, Source};
use rust_fontconfig::{FcFontCache, FcFontPath, FcPattern};

pub type Font = rusttype::Font<'static>;

use super::{FontBackend, Result};

static FONTCONFIG: OnceCell<FcFontCache> = OnceCell::new();

fn font_from_search(font_search: &FcFontPath) -> Result<Font> {
    let bytes = std::fs::read(font_search.path.as_str()).context("font read")?;
    Font::try_from_vec_and_index(bytes, font_search.font_index as u32).context("font creation")
}

impl FontBackend for Font {
    fn default() -> Self {
        FONTCONFIG
            .get_or_init(FcFontCache::build)
            .query(&FcPattern {
                monospace: rust_fontconfig::PatternMatch::True,
                ..Default::default()
            })
            .map(font_from_search)
            .unwrap()
            .unwrap()
    }

    fn font_by_name(name: &str) -> Result<Self> {
        FONTCONFIG
            .get_or_init(FcFontCache::build)
            .query(&FcPattern {
                name: Some(name.to_string()),
                ..Default::default()
            })
            .map(font_from_search)
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
        let scale = rusttype::Scale::uniform(font_size);
        let v_metrics = self.v_metrics(scale);

        let glyphs = self.layout(
            text,
            scale,
            rusttype::point(start_pos.x, start_pos.y + v_metrics.ascent),
        );

        for glyph in glyphs {
            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                glyph.draw(|x, y, v| {
                    dt.fill_rect(
                        (x + bounding_box.min.x as u32) as f32,
                        (y + bounding_box.min.y as u32) as f32,
                        1.,
                        1.,
                        &Source::Solid(SolidSource::from_unpremultiplied_argb(
                            (v * f32::from(color.a) * 255.0) as u8,
                            color.r,
                            color.g,
                            color.b,
                        )),
                        &opts,
                    );
                });
            }
        }
    }

    fn measure_text_width(
        &self,
        _: &DrawTarget,
        font_size: f32,
        text: &str,
        _: AntialiasMode,
    ) -> f32 {
        let scale = rusttype::Scale::uniform(font_size);
        self.glyphs_for(text.chars())
            .fold((None, 0.0), |(last, x), g| {
                let g = g.scaled(scale);
                let x = x + if let Some(last) = last {
                    self.pair_kerning(scale, last, g.id())
                } else {
                    0.0
                };
                let w = g.h_metrics().advance_width;
                let next = g.positioned(rusttype::point(x, 0.0));

                (Some(next.id()), x + w)
            })
            .1
    }
}
