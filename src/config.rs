use std::ffi::CString;
use std::path::PathBuf;

use defaults::Defaults;
use serde::Deserialize;

use crate::style::{Margin, Padding, Radius};
use crate::Color;

const DEFAULT_CONFIG_NAME: &str = concat!(crate::prog_name!(), ".config");

const DEFAULT_ICON_SIZE: u16 = 16;
const DEFAULT_FONT_SIZE: u16 = 24;

const DEFAULT_FONT_COLOR: Color = Color::from_rgba(0xf8, 0xf8, 0xf2, 0xff);
const DEFAULT_BG_COLOR: Color = Color::from_rgba(0x27, 0x28, 0x22, 0xee);
const DEFAULT_INPUT_BG_COLOR: Color = Color::from_rgba(0x75, 0x71, 0x5e, 0xc0);
const DEFAULT_SELECTED_FONT_COLOR: Color = Color::from_rgba(0xa6, 0xe2, 0x2e, 0xff);

mod params;

#[derive(Defaults, Deserialize)]
#[serde(default)]
pub struct Config {
    #[def = "400"]
    width: u32,
    #[def = "512"]
    height: u32,
    #[def = "false"]
    force_window: bool,
    window_offsets: Option<(i32, i32)>,
    scale: Option<u16>,
    term: Option<String>,
    font: Option<String>,
    font_size: Option<u16>,
    bg_color: Option<Color>,
    font_color: Option<Color>,
    #[def = "Radius::all(0.0)"]
    corner_radius: Radius,

    icon: Option<Icon>,

    input_text: InputText,
    list_items: ListItems,
}

impl Config {
    pub fn disable_icons(&mut self) {
        self.icon = None;
    }

    pub fn set_prompt(&mut self, prompt: String) {
        self.input_text.prompt = Some(prompt);
    }

    pub fn set_password(&mut self) {
        self.input_text.password = true;
    }
}

#[derive(Defaults, Deserialize)]
#[serde(default)]
struct InputText {
    font: Option<String>,
    font_size: Option<u16>,
    bg_color: Option<Color>,
    font_color: Option<Color>,
    prompt_color: Option<Color>,
    prompt: Option<String>,
    password: bool,
    #[def = "Margin::all(5.0)"]
    margin: Margin,
    #[def = "Padding::from_pair(1.7, -4.0)"]
    padding: Padding,
    #[def = "Radius::all(f32::MAX)"]
    corner_radius: Radius,
}

#[derive(Defaults, Deserialize)]
#[serde(default)]
struct ListItems {
    font: Option<String>,
    font_size: Option<u16>,
    font_color: Option<Color>,
    selected_font_color: Option<Color>,
    match_color: Option<Color>,
    #[def = "Margin { top: 10.0, ..Margin::from_pair(5.0, 15.0) }"]
    margin: Margin,
    #[def = "false"]
    hide_actions: bool,
    #[def = "60.0"]
    action_left_margin: f32,
    #[def = "2.0"]
    item_spacing: f32,
    #[def = "10.0"]
    icon_spacing: f32,
}

#[derive(Defaults, Deserialize)]
#[serde(default)]
struct Icon {
    #[def = "DEFAULT_ICON_SIZE"]
    size: u16,
    theme: Option<String>,
    fallback_icon_path: Option<PathBuf>,
}

fn config_path() -> PathBuf {
    xdg::BaseDirectories::with_prefix(crate::prog_name!())
        .expect("failed to get xdg dirs")
        .place_config_file(DEFAULT_CONFIG_NAME)
        .expect("cannot create configuration directory")
}

impl Config {
    pub fn load(path: Option<PathBuf>) -> Self {
        std::fs::read_to_string(path.unwrap_or_else(config_path))
            .map(|config_content| {
                toml::from_str(&config_content).unwrap_or_else(|e| panic!("Invalid config: {}", e))
            })
            .unwrap_or_default()
    }

    pub fn param<'a, T>(&'a self) -> T
    where
        T: From<&'a Self>,
    {
        self.into()
    }

    pub fn terminal_command(&self) -> Vec<CString> {
        if let Some(cmd) = self.term.as_ref() {
            shlex::split(cmd)
                .unwrap()
                .into_iter()
                .map(|s| CString::new(s).unwrap())
                .collect::<Vec<_>>()
        } else {
            vec![]
        }
    }
}
