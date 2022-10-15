use std::path::Path;

use anyhow::Result;
use raqote::{DrawOptions, Point, SolidSource};

use crate::DrawTarget;

mod fdue;
pub type Font = fdue::Font;

pub enum FontColor {
    Multiple(Vec<SolidSource>),
    Single(SolidSource),
}

pub trait FontBackend: Sized {
    fn default() -> Self {
        const DEFAULT_FONT: &str = "DejaVu Sans Mono";
        Self::font_by_name(DEFAULT_FONT)
            .unwrap_or_else(|e| panic!("cannot read the font `{}`: {}", DEFAULT_FONT, e))
    }

    fn font_by_name(name: &str) -> Result<Self>;

    fn font_by_path(path: &Path) -> Result<Self>;

    fn draw(
        &self,
        dt: &mut DrawTarget,
        text: &str,
        font_size: f32,
        start_pos: Point,
        color: FontColor,
        opts: &DrawOptions,
    );
}
