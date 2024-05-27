use std::ffi::CString;
use std::path::PathBuf;

use anyhow::{Context, Result};
use defaults::Defaults;
use serde::Deserialize;

use crate::style::{Margin, Padding, Radius};
use crate::Color;

const DEFAULT_CONFIG_NAMES: [&str; 2] = [
    concat!(crate::prog_name!(), ".toml"),
    concat!(crate::prog_name!(), ".config"),
];

const DEFAULT_ICON_SIZE: u16 = 16;
const DEFAULT_FONT_SIZE: u16 = 24;

const DEFAULT_FONT_COLOR: Color = Color::from_rgba(0xf8, 0xf8, 0xf2, 0xff);
const DEFAULT_BG_COLOR: Color = Color::from_rgba(0x27, 0x28, 0x22, 0xee);
const DEFAULT_INPUT_BG_COLOR: Color = Color::from_rgba(0x75, 0x71, 0x5e, 0xc0);
const DEFAULT_SELECTED_FONT_COLOR: Color = Color::from_rgba(0xa6, 0xe2, 0x2e, 0xff);

const DEFAULT_BG_BORDER_COLOR: Color = Color::from_rgba(0x13, 0x14, 0x11, 0xff);
const DEFAULT_BG_BORDER_WIDTH: f32 = 2.0;

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
    bg_border_color: Option<Color>,
    bg_border_width: Option<f32>,
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

    pub fn override_prompt(&mut self, prompt: String) {
        self.input_text.prompt = Some(prompt);
    }

    pub fn override_password(&mut self) {
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

fn default_config_path() -> Result<Option<PathBuf>> {
    let xdg_dirs =
        xdg::BaseDirectories::with_prefix(crate::prog_name!()).context("failed to get xdg dirs")?;

    for (index, &filename) in DEFAULT_CONFIG_NAMES.iter().enumerate() {
        let file = xdg_dirs.get_config_file(filename);
        if file
            .try_exists()
            .with_context(|| format!("reading default config at {}", file.display()))?
        {
            if index != 0 {
                eprintln!("warning: yofi.config is deprecated, please rename your configuration file to yofi.toml");
            }
            return Ok(Some(file));
        }
    }

    Ok(None)
}

impl Config {
    pub fn load(path: Option<PathBuf>) -> Result<Self> {
        let path = match path {
            Some(p) => p,
            None => match default_config_path()? {
                Some(path) => path,
                None => return Ok(Config::default()),
            },
        };
        match std::fs::read_to_string(&path) {
            Ok(c) => toml::from_str(&c).context("invalid config"),
            Err(err) if matches!(err.kind(), std::io::ErrorKind::NotFound) => Ok(Config::default()),
            Err(e) => {
                Err(anyhow::Error::new(e).context(format!("config read at {}", path.display())))
            }
        }
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
