use std::ffi::CStr;

use once_cell::sync::OnceCell;
use regex::Regex;

#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct Locale<'a> {
    lang: Option<&'a str>,
    country: Option<&'a str>,
    modifier: Option<&'a str>,
}

#[allow(clippy::needless_raw_string_hashes)]
const LOCALE_REGEX: &str = r#"(?x)
                             ^
                             ([[:alpha:]]+) # lang
                             (?:_([[:alpha:]]+))? # country
                             (?:\.[^@]*)? # encoding
                             (?:@(.*))? # modifier
                             $"#;

impl<'a> Locale<'a> {
    fn from_caputres(s: &'a str, captures: regex::Captures<'_>) -> Self {
        Self {
            lang: captures.get(1).map(|m| &s[m.range()]),
            country: captures.get(2).map(|m| &s[m.range()]),
            modifier: captures.get(3).map(|m| &s[m.range()]),
        }
    }
}

impl Locale<'static> {
    pub fn current<'a>() -> &'a Self {
        static LOCALE: OnceCell<Option<Locale<'static>>> = OnceCell::new();
        LOCALE
            .get_or_init(|| {
                let s = unsafe {
                    let ptr = libc::setlocale(libc::LC_MESSAGES, b"\0".as_ptr().cast());
                    if ptr.is_null() {
                        return None;
                    }
                    CStr::from_ptr(ptr)
                }
                .to_str()
                .ok()?;

                let re = Regex::new(LOCALE_REGEX).unwrap();

                let c = re.captures(s)?;

                Some(Self::from_caputres(s, c))
            })
            .as_ref()
            .unwrap_or(&Self {
                lang: None,
                country: None,
                modifier: None,
            })
    }

    pub fn keys(&self) -> impl Iterator<Item = impl AsRef<str>> + '_ {
        static LOCALE_ITERS: OnceCell<Vec<String>> = OnceCell::new();
        LOCALE_ITERS
            .get_or_init(|| {
                let mut v = vec![];
                if let Some(((l, c), m)) = self.lang.zip(self.country).zip(self.modifier) {
                    v.push(format!("{}_{}@{}", l, c, m));
                }
                if let Some((l, c)) = self.lang.zip(self.country) {
                    v.push(format!("{}_{}", l, c));
                }
                if let Some((l, m)) = self.lang.zip(self.modifier) {
                    v.push(format!("{}@{}", l, m));
                }
                if let Some(l) = self.lang {
                    v.push(l.to_string());
                }

                v
            })
            .clone()
            .into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use test_case::test_case;

    #[test]
    fn regex_compiles() {
        let _ = Regex::new(LOCALE_REGEX).unwrap();
    }

    #[test]
    fn regex_doesnt_match_empty() {
        let re = Regex::new(LOCALE_REGEX).unwrap();
        assert!(re.captures("").is_none());
    }

    impl Locale<'static> {
        fn new(
            lang: impl Into<Option<&'static str>>,
            country: impl Into<Option<&'static str>>,
            modifier: impl Into<Option<&'static str>>,
        ) -> Self {
            Self {
                lang: lang.into(),
                country: country.into(),
                modifier: modifier.into(),
            }
        }
    }

    #[test_case("qw", Locale::new("qw", None, None); "lang")]
    #[test_case("qw_ER", Locale::new("qw", "ER", None); "lang, country")]
    #[test_case("qw_ER.ty", Locale::new("qw", "ER", None); "lang, country, encoding")]
    #[test_case(
        "qw_ER.ty@ui",
        Locale::new("qw", "ER", "ui");
        "lang, country, encoding, modifier"
    )]
    #[test_case("qw@ui", Locale::new("qw", None, "ui"); "lang, modifier")]
    fn regex_matches(s: &str, x: Locale<'static>) {
        let re = Regex::new(LOCALE_REGEX).unwrap();
        let c = re.captures(s).unwrap();

        let m = c.get(0).unwrap();
        assert_eq!(m.start(), 0);
        assert_eq!(m.end(), s.len());

        assert_eq!(Locale::from_caputres(s, c), x);
    }
}
