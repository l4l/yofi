use std::cmp::Reverse;
use std::ffi::CString;

use super::{Entry, EvalInfo};
use crate::usage_cache::Usage;
use crate::DesktopEntry;

const CACHE_PATH: &str = concat!(crate::prog_name!(), ".cache");

pub struct AppsMode {
    entries: Vec<DesktopEntry>,
    term: Vec<CString>,
    usage: Usage,
}

impl AppsMode {
    pub fn new(mut entries: Vec<DesktopEntry>, term: Vec<CString>) -> Self {
        let usage = Usage::from_path(CACHE_PATH);

        entries.sort_by_key(|e| Reverse(usage.entry_count(&e.desktop_fname)));

        Self {
            entries,
            term,
            usage,
        }
    }

    pub fn eval(&mut self, info: EvalInfo<'_>) -> std::convert::Infallible {
        let idx = info.index.unwrap();
        let entry = &self.entries[idx];
        let args = shlex::split(&entry.exec)
            .unwrap()
            .into_iter()
            .filter(|s| !s.starts_with('%')) // TODO: use placeholders somehow
            .map(|s| CString::new(s).unwrap());

        self.usage
            .increment_entry_usage(entry.desktop_fname.clone());
        self.usage.try_update_cache(CACHE_PATH);

        let term = if entry.is_terminal {
            Some(std::mem::replace(&mut self.term, Vec::new()))
        } else {
            None
        };

        crate::exec::exec(term, args, info.input_value)
    }

    pub fn entries_len(&self) -> usize {
        self.entries.len()
    }

    pub fn entry(&self, idx: usize) -> Entry<'_> {
        let entry = &self.entries[idx];

        Entry {
            name: entry.name.as_str(),
            icon: entry.icon.as_ref().map(|i| i.as_image()),
        }
    }

    pub fn text_entries(&self) -> impl Iterator<Item = &str> + super::ExactSizeIterator {
        self.entries.iter().map(|e| e.name_with_keywords.as_str())
    }
}
