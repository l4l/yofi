use font_kit::family_name::FamilyName;
pub use font_kit::loaders::freetype::Font;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use raqote::{AntialiasMode, DrawOptions, DrawTarget, Point, Source};

use super::{FontBackend, FontColor, Result};

impl FontBackend for Font {
    fn default() -> Self {
        SystemSource::new()
            .select_best_match(&[FamilyName::Monospace], &Properties::new())
            .unwrap()
            .load()
            .unwrap()
    }

    fn font_by_name(name: &str) -> Result<Self> {
        SystemSource::new()
            .select_best_match(&[FamilyName::Title(name.to_string())], &Properties::new())?
            .load()
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
        let start = pathfinder_geometry::vector::vec2f(start_pos.x, start_pos.y + font_size);
        let (_, ids, positions) = text
            .chars()
            .filter_map(|c| {
                let id = self.glyph_for_char(c);

                let id = if let Some(id) = id {
                    id
                } else {
                    log::warn!("cannot find glyph for {:?}", c);
                    return None;
                };

                let advance = match self.advance(id) {
                    Ok(x) => x,
                    Err(err) => {
                        log::warn!("cannot advance font for {:?}: {}", c, err);
                        return None;
                    }
                };

                Some((id, advance))
            })
            .fold(
                (start, vec![], vec![]),
                |(start, mut ids, mut positions), (id, advance)| {
                    ids.push(id);
                    positions.push(Point::new(start.x(), start.y()));

                    let delta = advance * font_size / 24. / 96.;

                    (start + delta, ids, positions)
                },
            );

        let color = match color {
            FontColor::Single(color) => color,
            // Take just 1st color in vector, fontkit have a lot of problems in rasterize, so it is not a big deal
            // May be delete this font?
            FontColor::Multiple(ref colors) => colors[0],
        };

        dt.draw_glyphs(
            self,
            font_size,
            &ids,
            &positions,
            &Source::Solid(color),
            opts,
        );
    }

    fn measure_text_width(
        &self,
        dt: &DrawTarget,
        font_size: f32,
        text: &str,
        aa: AntialiasMode,
    ) -> f32 {
        dt.measure_text(self, font_size, text, aa)
            .unwrap_or_else(|e| panic!("failed to measure text: `{:?}`: {}", text, e))
            .size
            .width as f32
    }
}
