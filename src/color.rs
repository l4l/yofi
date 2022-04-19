use std::convert::TryInto;

use raqote::SolidSource;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct Color(#[serde(deserialize_with = "deserialize_color")] u32);

impl Color {
    pub const fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(u32::from_be_bytes([r, g, b, a]))
    }

    pub fn as_source(self) -> SolidSource {
        let [r, g, b, a] = self.to_be_bytes();
        SolidSource::from_unpremultiplied_argb(a, r, g, b)
    }
}

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
