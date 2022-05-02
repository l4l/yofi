use std::convert::TryInto;
use std::path::Path;

use font_kit::family_name::FamilyName;
use font_kit::loaders::freetype::Font as FontkitFont;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use pathfinder_geometry::vector::vec2f;
use raqote::{DrawOptions, Point, Source};

use super::{DrawTarget, FontBackend, FontColor, Result};

pub struct Font {
    inner: FontkitFont,
    face: Option<ttf_parser::Face<'static>>,
}

impl Font {
    fn from_inner(inner: FontkitFont) -> Self {
        let face = match inner.handle().as_ref() {
            Some(font_kit::handle::Handle::Memory { bytes, font_index }) => {
                ttf_parser::Face::from_slice(bytes.as_slice(), *font_index)
                    .map(|face| unsafe { std::mem::transmute(face) })
                    .ok()
            }
            _ => None,
        };

        Self { inner, face }
    }

    fn find_kerning(&self, prev: u32, next: u32) -> Option<f32> {
        let prev = ttf_parser::GlyphId(prev.try_into().ok()?);
        let next = ttf_parser::GlyphId(next.try_into().ok()?);

        self.face
            .as_ref()?
            .tables()
            .kern?
            .subtables
            .into_iter()
            .filter(|st| st.horizontal && !st.variable)
            .filter_map(|st| st.glyphs_kerning(prev, next))
            .next()
            .map(|x| x as f32)
    }
}

impl FontBackend for Font {
    fn default() -> Self {
        SystemSource::new()
            .select_best_match(&[FamilyName::Monospace], &Properties::new())
            .unwrap()
            .load()
            .map(Self::from_inner)
            .unwrap()
    }

    fn font_by_name(name: &str) -> Result<Self> {
        SystemSource::new()
            .select_best_match(&[FamilyName::Title(name.to_string())], &Properties::new())?
            .load()
            .map(Self::from_inner)
            .map_err(Into::into)
    }

    fn font_by_path(path: &Path) -> Result<Self> {
        FontkitFont::from_path(path, 0)
            .map(Self::from_inner)
            .map_err(Into::into)
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
        let em = font_size / 1.24;
        let start = vec2f(start_pos.x, start_pos.y + font_size);
        let units_per_em = self.inner.metrics().units_per_em as f32;

        let (_, _, ids, positions) = text
            .chars()
            .filter_map(|c| {
                let id = self.inner.glyph_for_char(c);

                let id = if let Some(id) = id {
                    id
                } else {
                    log::warn!("cannot find glyph for {:?}", c);
                    return None;
                };

                let advance = match self.inner.advance(id) {
                    Ok(x) => x,
                    Err(err) => {
                        log::warn!("cannot advance font for {:?}: {}", c, err);
                        return None;
                    }
                };

                Some((id, advance))
            })
            .fold(
                (start, None, vec![], vec![]),
                |(start, prev_id, mut ids, mut positions), (id, advance)| {
                    ids.push(id);
                    let kern_x = prev_id
                        .and_then(|prev_id| self.find_kerning(prev_id, id))
                        .unwrap_or(0.0)
                        * em
                        / units_per_em;
                    positions.push(Point::new(start.x() + kern_x, start.y()));

                    let delta = advance * em / units_per_em as f32 + vec2f(kern_x, 0.0);

                    (start + delta, Some(id), ids, positions)
                },
            );

        let mut draw_glyphs = |ids: &[_], positions: &[_], color| {
            dt.draw_glyphs(&self.inner, em, ids, positions, &Source::Solid(color), opts);
        };

        match color {
            FontColor::Single(color) => draw_glyphs(&ids, &positions, color),
            FontColor::Multiple(colors) => {
                for ((id, position), color) in ids.into_iter().zip(positions).zip(colors) {
                    draw_glyphs(&[id], &[position], color);
                }
            }
        };
    }
}
