use font_kit::family_name::FamilyName;
use font_kit::loaders::freetype::Font;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use raqote::SolidSource;

use super::Config;
use crate::draw::{BgParams, InputTextParams, ListParams};

impl<'a> From<&'a Config> for InputTextParams {
    fn from(config: &'a Config) -> InputTextParams {
        InputTextParams {
            font: config
                .input_text
                .as_ref()
                .and_then(|c| c.font.clone())
                .or_else(|| config.font.clone())
                .map(font_by_name)
                .unwrap_or_else(default_font),
            bg_color: config
                .input_text
                .as_ref()
                .and_then(|c| c.bg_color)
                .or_else(|| config.bg_color)
                .map(u32_to_solid_source)
                .unwrap_or_else(|| SolidSource::from_unpremultiplied_argb(0xc0, 0x75, 0x71, 0x5e)),
            font_color: config
                .input_text
                .as_ref()
                .and_then(|c| c.font_color)
                .or_else(|| config.font_color)
                .map(u32_to_solid_source)
                .unwrap_or_else(default_font_color),
        }
    }
}

impl<'a> From<&'a Config> for ListParams {
    fn from(config: &'a Config) -> ListParams {
        ListParams {
            font: config
                .list_items
                .as_ref()
                .and_then(|c| c.font.clone())
                .or_else(|| config.font.clone())
                .map(font_by_name)
                .unwrap_or_else(default_font),
            font_color: config
                .list_items
                .as_ref()
                .and_then(|c| c.font_color)
                .or_else(|| config.font_color)
                .map(u32_to_solid_source)
                .unwrap_or_else(default_font_color),
            selected_font_color: config
                .list_items
                .as_ref()
                .and_then(|c| c.selected_font_color)
                .map(u32_to_solid_source)
                .unwrap_or_else(|| SolidSource::from_unpremultiplied_argb(0xff, 0xa6, 0xe2, 0x2e)),
        }
    }
}

impl<'a> From<&'a Config> for BgParams {
    fn from(config: &'a Config) -> BgParams {
        BgParams {
            color: config
                .bg_color
                .map(u32_to_solid_source)
                .unwrap_or_else(|| SolidSource::from_unpremultiplied_argb(0xee, 0x27, 0x28, 0x22)),
        }
    }
}

std::thread_local! {
    static FONT: Font = SystemSource::new()
        .select_best_match(&[FamilyName::SansSerif], &Properties::new())
        .unwrap()
        .load()
        .unwrap();
}

fn default_font() -> Font {
    FONT.with(Clone::clone)
}

fn default_font_color() -> SolidSource {
    SolidSource::from_unpremultiplied_argb(0xff, 0xf8, 0xf8, 0xf2)
}

fn font_by_name(name: String) -> Font {
    SystemSource::new()
        .select_best_match(&[FamilyName::Title(name)], &Properties::new())
        .unwrap()
        .load()
        .unwrap()
}

fn u32_to_solid_source(x: u32) -> SolidSource {
    let bytes = x.to_be_bytes();
    SolidSource::from_unpremultiplied_argb(bytes[3], bytes[0], bytes[1], bytes[2])
}
