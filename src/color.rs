use anyhow::Context;
use raqote::SolidSource;
use serde::Deserialize;

#[derive(Deserialize, Clone, Copy)]
#[cfg_attr(test, derive(Debug, PartialEq))]
#[serde(from = "ColorDeser")]
pub struct Color(u32);

impl Color {
    pub const fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(u32::from_be_bytes([r, g, b, a]))
    }

    pub const fn to_rgba(self) -> [u8; 4] {
        self.0.to_be_bytes()
    }

    pub fn as_source(self) -> SolidSource {
        let [r, g, b, a] = self.to_rgba();
        SolidSource::from_unpremultiplied_argb(a, r, g, b)
    }
}

impl std::ops::Deref for Color {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<ColorDeser> for Color {
    fn from(value: ColorDeser) -> Self {
        match value {
            ColorDeser::Int(x) => Self(x),
            ColorDeser::String(ColorDeserString(c)) => c,
        }
    }
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum ColorDeser {
    Int(u32),
    String(ColorDeserString),
}

#[derive(serde::Deserialize)]
#[serde(try_from = "String")]
struct ColorDeserString(Color);

impl TryFrom<String> for ColorDeserString {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let part = match value.chars().next() {
            None => anyhow::bail!("color cannot be empty"),
            Some('#') => value.split_at(1).1,
            Some(_) => {
                anyhow::bail!("color can be either decimal or hex number prefixed with '#'")
            }
        };

        let decoded = u32::from_str_radix(part, 16).context("parse hex number");
        Ok(Self(match (decoded, part.len()) {
            (Ok(d), 3) => {
                let (r, g, b) = (
                    ((d & 0xf00) >> 8) as u8,
                    ((d & 0xf0) >> 4) as u8,
                    (d & 0xf) as u8,
                );
                Color::from_rgba(r << 4 | r, g << 4 | g, b << 4 | b, 0xff)
            }
            (Ok(d), 6) => Color(d << 8 | 0xff),
            (Ok(d), 8) => Color(d),
            (e, _) => anyhow::bail!(
                "hex color can only be specified in #RGB, #RRGGBB, or #RRGGBBAA format, {e:?}"
            ),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(r##"x = 1234"##, Color(1234); "decimal number")]
    #[test_case(r##"x = 0x5432"##, Color(0x5432); "hex number")]
    #[test_case(r##"x = "#123""##, Color::from_rgba(0x11, 0x22, 0x33, 0xff); "3-sym css")]
    #[test_case(r##"x = "#123456""##, Color::from_rgba(0x12, 0x34, 0x56, 0xff); "6-sym css")]
    #[test_case(r##"x = "#12345678""##, Color::from_rgba(0x12, 0x34, 0x56, 0x78); "8-sym css")]
    fn deser_color(s: &str, expected: Color) {
        #[derive(Deserialize)]
        struct T {
            x: Color,
        }
        let c = toml::from_str::<T>(s).unwrap().x;
        assert_eq!(c, expected);
    }
}
