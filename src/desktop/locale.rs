use std::ffi::CStr;

use once_cell::sync::OnceCell;
use regex::Regex;

pub struct Locale<'a> {
    lang: Option<&'a str>,
    country: Option<&'a str>,
    modifier: Option<&'a str>,
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

                let re = Regex::new(
                    r#"(?x)
                      ^
                      ([[:alpha:]]+) # lang
                      (?:_([[:alpha:]]+))? # country
                      (?:\.[^@]*)? # encoding
                      (?:@(.*))? # modifier
                      $"#,
                )
                .unwrap();

                let c = re.captures(s)?;

                Some(Self {
                    lang: c.get(1).map(|m| &s[m.range()]),
                    country: c.get(2).map(|m| &s[m.range()]),
                    modifier: c.get(3).map(|m| &s[m.range()]),
                })
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
