use anyhow::Result;
use raqote::{AntialiasMode, DrawOptions, DrawTarget, Point, SolidSource};

#[cfg(all(feature = "font-fontkit", feature = "font-rusttype"))]
std::compile_error!("Multiple font backends are not supported. Choose only a single backend");

#[cfg(feature = "font-fontkit")]
mod fontkit;
#[cfg(feature = "font-fontkit")]
pub type Font = fontkit::Font;

#[cfg(feature = "font-rusttype")]
mod rtype;
#[cfg(feature = "font-rusttype")]
pub type Font = rtype::Font;

pub trait FontBackend: Sized {
    fn default() -> Self {
        const DEFAULT_FONT: &str = "DejaVu Sans Mono";
        Self::font_by_name(DEFAULT_FONT)
            .unwrap_or_else(|e| panic!("cannot read the font `{}`: {}", DEFAULT_FONT, e))
    }

    fn font_by_name(name: &str) -> Result<Self>;

    fn draw(
        &self,
        dt: &mut DrawTarget,
        text: &str,
        font_size: f32,
        start_pos: Point,
        color: SolidSource,
        opts: &DrawOptions,
    );

    fn measure_text_width(
        &self,
        dt: &DrawTarget,
        font_size: f32,
        text: &str,
        aa: AntialiasMode,
    ) -> f32;
}
