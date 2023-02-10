use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use once_cell::unsync::Lazy;

use super::*;
use crate::desktop::IconConfig;
use crate::draw::{BgParams, InputTextParams, ListParams};
use crate::font::{Font, FontBackend, InnerFont};
use crate::icon::Icon;
use crate::surface::Params as SurfaceParams;

macro_rules! select_conf {
    ($config:ident, $inner:ident, $field:ident) => {
        $config
            .$inner
            .$field
            .as_ref()
            .or_else(|| $config.$field.as_ref())
    };
}

impl<'a> From<&'a Config> for InputTextParams<'a> {
    fn from(config: &'a Config) -> InputTextParams<'a> {
        let font_color = select_conf!(config, input_text, font_color)
            .copied()
            .unwrap_or(DEFAULT_FONT_COLOR);

        InputTextParams {
            font: select_conf!(config, input_text, font)
                .map(font_by_name)
                .unwrap_or_else(default_font),
            font_size: select_conf!(config, input_text, font_size)
                .copied()
                .unwrap_or(DEFAULT_FONT_SIZE),
            bg_color: select_conf!(config, input_text, bg_color)
                .copied()
                .unwrap_or(DEFAULT_INPUT_BG_COLOR),
            font_color,
            prompt_color: config.input_text.prompt_color.unwrap_or_else(|| {
                let [r, g, b, a] = font_color.to_rgba();
                Color::from_rgba(r, g, b, (a / 4).wrapping_mul(3))
            }),
            prompt: config.input_text.prompt.as_deref(),
            password: config.input_text.password,
            margin: config.input_text.margin.clone(),
            padding: config.input_text.padding.clone(),
            radius: config.input_text.corner_radius.clone(),
        }
    }
}

impl<'a> From<&'a Config> for ListParams {
    fn from(config: &'a Config) -> ListParams {
        ListParams {
            font: select_conf!(config, list_items, font)
                .map(font_by_name)
                .unwrap_or_else(default_font),
            font_size: select_conf!(config, list_items, font_size)
                .copied()
                .unwrap_or(DEFAULT_FONT_SIZE),
            font_color: select_conf!(config, list_items, font_color)
                .copied()
                .unwrap_or(DEFAULT_FONT_COLOR),
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
                .map(|path| Icon::new(&path)),
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
            width: config.width,
            height: config.height,
            radius: config.corner_radius.clone(),
            color: config.bg_color.unwrap_or(DEFAULT_BG_COLOR),
            border_color: config.bg_border_color.unwrap_or(DEFAULT_BG_BORDER_COLOR),
            border_width: config.bg_border_width.unwrap_or(2.0),
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
            theme: c.theme.clone(),
        })
    }
}

fn default_font() -> Font {
    std::thread_local! {
        static DEFAULT_FONT: Lazy<Font> = Lazy::new(|| Rc::new(InnerFont::default()));
    }
    DEFAULT_FONT.with(|f| Rc::clone(f))
}

fn font_by_name(name: impl AsRef<str>) -> Font {
    std::thread_local! {
        static LOADED_FONTS: RefCell<HashMap<String, Font>> = RefCell::new(HashMap::new());
    }

    let name = name.as_ref();

    if let Some(font) = LOADED_FONTS.with(|fonts| fonts.borrow().get(name).cloned()) {
        return font;
    }

    let path = Path::new(name);
    let font = if path.is_absolute() && path.exists() {
        InnerFont::font_by_path(path)
    } else {
        InnerFont::font_by_name(name)
    };
    let font = Rc::new(font.unwrap_or_else(|e| panic!("cannot find font {}: {}", name, e)));
    LOADED_FONTS.with(|fonts| fonts.borrow_mut().insert(name.to_owned(), font.clone()));
    font
}
