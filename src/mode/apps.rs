use std::cmp::Reverse;
use std::ffi::CString;

use super::Entry;
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

    pub fn eval(&mut self, idx: usize) -> std::convert::Infallible {
        let entry = &self.entries[idx];
        let args = shlex::split(&entry.exec)
            .unwrap()
            .into_iter()
            .filter(|s| !s.starts_with('%')) // TODO: use placeholders somehow
            .map(|s| CString::new(s).unwrap())
            .collect::<Vec<_>>();
        let (prog, args) = if entry.is_terminal {
            assert!(
                !self.term.is_empty(),
                "Cannot find terminal, specify `term` in config"
            );
            self.term.extend(args);
            (&self.term[0], &self.term[0..])
        } else {
            (&args[0], &args[0..])
        };

        self.usage
            .increment_entry_usage(entry.desktop_fname.clone());
        self.usage.try_update_cache(CACHE_PATH);

        log::debug!("executing command: {:?} {:?}", prog, args);
        nix::unistd::execvp(prog, args).unwrap_or_else(|e| {
            panic!(
                "failed to launch desktop file {:?} with command line `{:?} {:?}`: {}",
                entry.path, prog, args, e
            )
        });

        unreachable!()
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

    pub fn text_entries(&self) -> impl Iterator<Item = &str> {
        self.entries.iter().map(|e| e.name.as_str())
    }
}
