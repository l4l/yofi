use std::path::Path;

use super::*;
use crate::desktop::{IconConfig, DEFAULT_THEME};
use crate::draw::{BgParams, InputTextParams, ListParams};
use crate::font::{Font, FontBackend};
use crate::icon::Icon;
use crate::surface::Params as SurfaceParams;

macro_rules! select_conf {
    ($config:ident, $inner:ident, $field:ident) => {
        $config
            .$inner
            .$field
            .clone()
            .or_else(|| $config.$field.clone())
    };
}

impl<'a> From<&'a Config> for InputTextParams {
    fn from(config: &'a Config) -> InputTextParams {
        let font_color = select_conf!(config, input_text, font_color).unwrap_or(DEFAULT_FONT_COLOR);

        InputTextParams {
            font: select_conf!(config, input_text, font)
                .map(font_by_name)
                .unwrap_or_else(Font::default),
            font_size: select_conf!(config, input_text, font_size).unwrap_or(DEFAULT_FONT_SIZE),
            bg_color: select_conf!(config, input_text, bg_color).unwrap_or(DEFAULT_INPUT_BG_COLOR),
            font_color,
            prompt_color: config.input_text.prompt_color.unwrap_or_else(|| {
                let [r, g, b, a] = font_color.to_rgba();
                Color::from_rgba(r, g, b, (a / 4).wrapping_mul(3))
            }),
            prompt: config.input_text.prompt.clone(),
            password: config.input_text.password,
            margin: config.input_text.margin.clone(),
            padding: config.input_text.padding.clone(),
        }
    }
}

impl<'a> From<&'a Config> for ListParams {
    fn from(config: &'a Config) -> ListParams {
        ListParams {
            font: select_conf!(config, list_items, font)
                .map(font_by_name)
                .unwrap_or_else(Font::default),
            font_size: select_conf!(config, list_items, font_size).unwrap_or(DEFAULT_FONT_SIZE),
            font_color: select_conf!(config, list_items, font_color).unwrap_or(DEFAULT_FONT_COLOR),
            selected_font_color: config
                .list_items
                .selected_font_color
                .unwrap_or(DEFAULT_SELECTED_FONT_COLOR),
            match_color: config.list_items.match_color,
            icon_size: config.icon.as_ref().map(|c| c.size),
            fallback_icon: config
                .icon
                .as_ref()
                .and_then(|i| i.fallback_icon_path.as_ref())
                .map(|path| Icon::load_icon(&path).expect("cannot load fallback icon")),
            margin: config.list_items.margin.clone(),
            hide_actions: config.list_items.hide_actions,
            action_left_margin: config.list_items.action_left_margin,
            item_spacing: config.list_items.item_spacing,
            icon_spacing: config.list_items.icon_spacing,
        }
    }
}

impl<'a> From<&'a Config> for BgParams {
    fn from(config: &'a Config) -> BgParams {
        BgParams {
            color: config.bg_color.unwrap_or(DEFAULT_BG_COLOR),
        }
    }
}

impl<'a> From<&'a Config> for SurfaceParams {
    fn from(config: &'a Config) -> SurfaceParams {
        SurfaceParams {
            width: config.width,
            height: config.height,
            force_window: config.force_window,
            window_offsets: config.window_offsets,
            scale: config.scale,
        }
    }
}

impl<'a> From<&'a Config> for Option<IconConfig> {
    fn from(config: &'a Config) -> Option<IconConfig> {
        config.icon.as_ref().map(|c| IconConfig {
            icon_size: c.size,
            theme: c
                .theme
                .as_ref()
                .unwrap_or_else(|| once_cell::sync::Lazy::force(&DEFAULT_THEME))
                .clone(),
        })
    }
}

fn font_by_name(name: String) -> Font {
    let path = Path::new(name.as_str());
    if path.is_absolute() && path.exists() {
        Font::font_by_path(path)
    } else {
        Font::font_by_name(name.as_str())
    }
    .unwrap_or_else(|e| panic!("cannot find font {}: {}", name, e))
}
