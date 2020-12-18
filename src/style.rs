use std::fmt::Display;
use std::marker::PhantomData;
use std::str::FromStr;

use serde::de::{Deserializer, Visitor};
use serde::Deserialize;

#[derive(Clone, Default)]
pub struct Padding {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

#[derive(Clone, Default)]
pub struct Margin {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

impl Padding {
    pub const fn all(val: f32) -> Self {
        Self {
            top: val,
            bottom: val,
            left: val,
            right: val,
        }
    }

    pub const fn from_pair(vertical: f32, horizontal: f32) -> Self {
        Self {
            top: vertical,
            bottom: vertical,
            left: horizontal,
            right: horizontal,
        }
    }
}

impl Margin {
    pub const fn all(val: f32) -> Self {
        Self {
            top: val,
            bottom: val,
            left: val,
            right: val,
        }
    }

    pub const fn from_pair(vertical: f32, horizontal: f32) -> Self {
        Self {
            top: vertical,
            bottom: vertical,
            left: horizontal,
            right: horizontal,
        }
    }
}

impl<'de> Deserialize<'de> for Padding {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        d.deserialize_str(StringVisitor(PhantomData))
    }
}

impl<'de> Deserialize<'de> for Margin {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        d.deserialize_str(StringVisitor(PhantomData))
    }
}

struct StringVisitor<T>(PhantomData<T>);

impl<'de, T> Visitor<'de> for StringVisitor<T>
where
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    type Value = T;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("unexpected value")
    }

    fn visit_str<E>(self, value: &str) -> Result<T, E>
    where
        E: serde::de::Error,
    {
        FromStr::from_str(value).map_err(serde::de::Error::custom)
    }
}

impl FromStr for Padding {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let values = s
            .split(' ')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.parse::<f32>().map_err(|_| "invalid float value"))
            .collect::<Result<Vec<f32>, _>>()?;

        match values.len() {
            1 => Ok(Self::all(values[0])),
            2 => Ok(Self::from_pair(values[0], values[1])),
            4 => Ok(Self {
                top: values[0],
                bottom: values[2],
                left: values[3],
                right: values[1],
            }),
            _ => Err("padding should consists of either 1, 2 or 4 floats"),
        }
    }
}

impl FromStr for Margin {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let values = s
            .split(' ')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.parse::<f32>().map_err(|_| "invalid float value"))
            .collect::<Result<Vec<f32>, _>>()?;

        match values.len() {
            1 => Ok(Self::all(values[0])),
            2 => Ok(Self::from_pair(values[0], values[1])),
            4 => Ok(Self {
                top: values[0],
                bottom: values[2],
                left: values[3],
                right: values[1],
            }),
            _ => Err("margin should consists of either 1, 2 or 4 floats"),
        }
    }
}
