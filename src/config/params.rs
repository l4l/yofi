use font_kit::family_name::FamilyName;
use font_kit::loaders::freetype::Font;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use raqote::SolidSource;

use super::Config;
use crate::desktop::{IconConfig, DEFAULT_ICON_SIZE, DEFAULT_THEME};
use crate::draw::{BgParams, InputTextParams, ListParams};
use crate::surface::Params as SurfaceParams;

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
                .or(config.bg_color)
                .map(u32_to_solid_source)
                .unwrap_or_else(|| SolidSource::from_unpremultiplied_argb(0xc0, 0x75, 0x71, 0x5e)),
            font_color: config
                .input_text
                .as_ref()
                .and_then(|c| c.font_color)
                .or(config.font_color)
                .map(u32_to_solid_source)
                .unwrap_or_else(default_font_color),
            margin: config
                .input_text
                .as_ref()
                .and_then(|c| c.margin.clone())
                .unwrap_or_default(),
            padding: config
                .input_text
                .as_ref()
                .and_then(|c| c.padding.clone())
                .unwrap_or_default(),
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
                .or(config.font_color)
                .map(u32_to_solid_source)
                .unwrap_or_else(default_font_color),
            selected_font_color: config
                .list_items
                .as_ref()
                .and_then(|c| c.selected_font_color)
                .map(u32_to_solid_source)
                .unwrap_or_else(|| SolidSource::from_unpremultiplied_argb(0xff, 0xa6, 0xe2, 0x2e)),
            icon_size: config
                .icon
                .as_ref()
                .map(|i| i.size.unwrap_or(DEFAULT_ICON_SIZE)),
            fallback_icon: config
                .icon
                .as_ref()
                .and_then(|i| i.fallback_icon_path.as_ref())
                .map(|path| {
                    crate::icon::Icon::load_icon(&path).expect("cannot load fallback icon")
                }),
            margin: config
                .list_items
                .as_ref()
                .and_then(|c| c.margin.clone())
                .unwrap_or_default(),
            item_spacing: config
                .list_items
                .as_ref()
                .and_then(|c| c.item_spacing)
                .unwrap_or_default(),
            icon_spacing: config
                .list_items
                .as_ref()
                .and_then(|c| c.icon_spacing)
                .unwrap_or_default(),
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

impl<'a> From<&'a Config> for SurfaceParams {
    fn from(config: &'a Config) -> SurfaceParams {
        SurfaceParams {
            width: config.width.unwrap_or(400),
            height: config.height.unwrap_or(512),
            window_offsets: config.window_offsets,
        }
    }
}

impl<'a> From<&'a Config> for Option<IconConfig> {
    fn from(config: &'a Config) -> Option<IconConfig> {
        config.icon.as_ref().map(|c| IconConfig {
            icon_size: c.size.unwrap_or(DEFAULT_ICON_SIZE),
            theme: c
                .theme
                .as_ref()
                .unwrap_or_else(|| once_cell::sync::Lazy::force(&DEFAULT_THEME))
                .clone(),
        })
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
