use std::ffi::CString;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const CONFIG_FILENAME: &str = "yofi.config";

mod params;

#[derive(Default, Serialize, Deserialize)]
pub struct Config {
    width: Option<u32>,
    height: Option<u32>,
    window_offsets: Option<(i32, i32)>,
    term: Option<String>,
    font: Option<String>,
    bg_color: Option<u32>,
    font_color: Option<u32>,

    input_text: Option<InputText>,
    list_items: Option<ListItems>,
}

#[derive(Serialize, Deserialize)]
struct InputText {
    font: Option<String>,
    bg_color: Option<u32>,
    font_color: Option<u32>,
}

#[derive(Serialize, Deserialize)]
struct ListItems {
    font: Option<String>,
    font_color: Option<u32>,
    selected_font_color: Option<u32>,
}

fn config_path() -> PathBuf {
    xdg::BaseDirectories::with_prefix("yofi")
        .unwrap()
        .place_config_file(CONFIG_FILENAME)
        .expect("cannot create configuration directory")
}

impl Config {
    pub fn load() -> Self {
        std::fs::read_to_string(config_path())
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
