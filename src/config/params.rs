use font_kit::family_name::FamilyName;
use font_kit::loaders::freetype::Font;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use raqote::SolidSource;

use super::Config;
use crate::desktop::{IconConfig, DEFAULT_THEME};
use crate::draw::{BgParams, InputTextParams, ListParams};
use crate::icon::Icon;
use crate::style::{Margin, Padding};
use crate::surface::Params as SurfaceParams;

const DEFAULT_FONT_SIZE: u16 = 24;
const DEFAULT_ICON_SIZE: u16 = 16;

macro_rules! select_conf {
    ($config:ident, $base:ident, $field:ident) => {
        select_conf!(noglob: $config, $base, $field).or_else(|| $config.$field.clone())
    };
    (noglob: $config:ident, $base:ident, $field:ident) => {
        $config.$base.as_ref().and_then(|c| c.$field.clone())
    };
}

impl<'a> From<&'a Config> for InputTextParams {
    fn from(config: &'a Config) -> InputTextParams {
        InputTextParams {
            font: select_conf!(config, input_text, font)
                .map(font_by_name)
                .unwrap_or_else(default_font),
            font_size: select_conf!(config, input_text, font_size).unwrap_or(DEFAULT_FONT_SIZE),
            bg_color: select_conf!(config, input_text, bg_color)
                .map(u32_to_solid_source)
                .unwrap_or_else(|| SolidSource::from_unpremultiplied_argb(0xc0, 0x75, 0x71, 0x5e)),
            font_color: select_conf!(config, input_text, font_color)
                .map(u32_to_solid_source)
                .unwrap_or_else(default_font_color),
            margin: select_conf!(noglob: config, input_text, margin)
                .unwrap_or_else(|| Margin::all(5.0)),
            padding: select_conf!(noglob: config, input_text, padding)
                .unwrap_or_else(|| Padding::from_pair(1.7, -4.0)),
        }
    }
}

impl<'a> From<&'a Config> for ListParams {
    fn from(config: &'a Config) -> ListParams {
        ListParams {
            font: select_conf!(config, list_items, font)
                .map(font_by_name)
                .unwrap_or_else(default_font),
            font_size: select_conf!(config, list_items, font_size).unwrap_or(DEFAULT_FONT_SIZE),
            font_color: select_conf!(config, list_items, font_color)
                .map(u32_to_solid_source)
                .unwrap_or_else(default_font_color),
            selected_font_color: select_conf!(noglob: config, list_items, selected_font_color)
                .map(u32_to_solid_source)
                .unwrap_or_else(|| SolidSource::from_unpremultiplied_argb(0xff, 0xa6, 0xe2, 0x2e)),
            match_color: select_conf!(noglob: config, list_items, match_color)
                .map(u32_to_solid_source),
            icon_size: config
                .icon
                .as_ref()
                .map(|c| c.size.unwrap_or(DEFAULT_ICON_SIZE))
                .unwrap_or(0),
            fallback_icon: select_conf!(noglob: config, icon, fallback_icon_path)
                .map(|path| Icon::load_icon(&path).expect("cannot load fallback icon")),
            margin: select_conf!(noglob: config, list_items, margin).unwrap_or_else(|| Margin {
                top: 10.0,
                ..Margin::from_pair(5.0, 15.0)
            }),
            item_spacing: select_conf!(noglob: config, list_items, item_spacing).unwrap_or(2.0),
            icon_spacing: select_conf!(noglob: config, list_items, icon_spacing).unwrap_or(10.0),
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
