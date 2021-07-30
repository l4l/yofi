use std::convert::TryInto;
use std::ffi::CString;
use std::path::PathBuf;

use serde::Deserialize;

use crate::style::{Margin, Padding};

const DEFAULT_CONFIG_PATH: &str = concat!(crate::prog_name!(), ".config");

mod params;

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct Color(#[serde(deserialize_with = "deserialize_color")] u32);

impl std::ops::Deref for Color {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn deserialize_color<'de, D: serde::Deserializer<'de>>(d: D) -> Result<u32, D::Error> {
    struct ColorDeHelper;

    impl<'de> serde::de::Visitor<'de> for ColorDeHelper {
        type Value = u32;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                formatter,
                "invalid color value, must be either numerical or css-like hex value with # prefix"
            )
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            value.try_into().map_err(serde::de::Error::custom)
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let part = match value.chars().next() {
                None => return Err(serde::de::Error::custom("color cannot be empty")),
                Some('#') => value.split_at(1).1,
                Some(_) => {
                    return Err(serde::de::Error::custom(
                        "color can be either decimal or hex number prefixed with '#'",
                    ))
                }
            };

            let decoded = u32::from_str_radix(part, 16).map_err(serde::de::Error::custom);
            match part.len() {
                3 => {
                    let decoded = decoded?;
                    let (r, g, b) = ((decoded & 0xf00) >> 8, (decoded & 0xf0) >> 4, decoded & 0xf);
                    Ok((r << 4 | r) << 24 | (g << 4 | g) << 16 | (b << 4 | b) << 8 | 0xff)
                }
                6 => decoded.map(|d| d << 8 | 0xff),
                8 => decoded,
                _ => Err(serde::de::Error::custom(
                    "hex color can only be specified in #RGB, #RRGGBB, or #RRGGBBAA format",
                )),
            }
        }
    }

    d.deserialize_any(ColorDeHelper)
}

#[derive(Default, Deserialize)]
pub struct Config {
    width: Option<u32>,
    height: Option<u32>,
    force_window: Option<bool>,
    window_offsets: Option<(i32, i32)>,
    scale: Option<u16>,
    term: Option<String>,
    font: Option<String>,
    font_size: Option<u16>,
    bg_color: Option<Color>,
    font_color: Option<Color>,

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
    bg_color: Option<Color>,
    font_color: Option<Color>,
    margin: Option<Margin>,
    padding: Option<Padding>,
}

#[derive(Deserialize)]
struct ListItems {
    font: Option<String>,
    font_size: Option<u16>,
    font_color: Option<Color>,
    selected_font_color: Option<Color>,
    match_color: Option<Color>,
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
            .map(|config_content| {
                toml::from_str(&config_content).unwrap_or_else(|e| panic!("Invalid config: {}", e))
            })
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
            shlex::split(cmd)
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
