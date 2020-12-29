use std::ffi::CString;
use std::path::PathBuf;

use serde::Deserialize;

use crate::style::{Margin, Padding};

const DEFAULT_CONFIG_PATH: &str = concat!(crate::prog_name!(), ".config");

mod params;

#[derive(Default, Deserialize)]
pub struct Config {
    width: Option<u32>,
    height: Option<u32>,
    scale: Option<u16>,
    window_offsets: Option<(i32, i32)>,
    term: Option<String>,
    font: Option<String>,
    font_size: Option<u16>,
    bg_color: Option<u32>,
    font_color: Option<u32>,

    icon: Option<Icon>,

    input_text: Option<InputText>,
    list_items: Option<ListItems>,
}

impl Config {
    pub fn disable_icons(&mut self) {
        self.icon = None;
    }
}

#[derive(Deserialize)]
struct InputText {
    font: Option<String>,
    font_size: Option<u16>,
    bg_color: Option<u32>,
    font_color: Option<u32>,
    margin: Option<Margin>,
    padding: Option<Padding>,
}

#[derive(Deserialize)]
struct ListItems {
    font: Option<String>,
    font_size: Option<u16>,
    font_color: Option<u32>,
    selected_font_color: Option<u32>,
    match_color: Option<u32>,
    margin: Option<Margin>,
    item_spacing: Option<f32>,
    icon_spacing: Option<f32>,
}

#[derive(Deserialize)]
struct Icon {
    size: Option<u16>,
    theme: Option<String>,
    fallback_icon_path: Option<PathBuf>,
}

fn config_path() -> PathBuf {
    xdg::BaseDirectories::with_prefix(crate::prog_name!())
        .expect("failed to get xdg dirs")
        .place_config_file(DEFAULT_CONFIG_PATH)
        .expect("cannot create configuration directory")
}

impl Config {
    pub fn load(path: Option<PathBuf>) -> Self {
        std::fs::read_to_string(path.unwrap_or_else(config_path))
            .map(|config_content| toml::from_str(&config_content).expect("invalid config"))
            .unwrap_or_default()
    }

    pub fn param<T>(&self) -> T
    where
        T: for<'a> From<&'a Self>,
    {
        self.into()
    }

    pub fn terminal_command(&self) -> Vec<CString> {
        if let Some(cmd) = self.term.as_ref() {
            shlex::split(&cmd)
                .unwrap()
                .into_iter()
                .map(|s| CString::new(s).unwrap())
                .collect::<Vec<_>>()
        } else if let Ok(term) = std::env::var("TERM") {
            vec![CString::new(term).unwrap()]
        } else {
            vec![]
        }
    }
}
