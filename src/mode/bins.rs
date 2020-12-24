use std::cmp::Reverse;
use std::ffi::CString;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use super::Entry;
use crate::usage_cache::Usage;

const CACHE_PATH: &str = concat!(crate::prog_name!(), ".bincache");

pub struct BinsMode {
    bins: Vec<PathBuf>,
    term: Vec<CString>,
    usage: Usage,
}

impl BinsMode {
    pub fn new(term: Vec<CString>) -> Self {
        let usage = Usage::from_path(CACHE_PATH);

        let mut bins: Vec<_> = std::env::var("PATH")
            .map(|paths| paths.split(':').map(|s| s.to_owned()).collect())
            .unwrap_or_else(|_| vec!["/usr/bin".into()])
            .into_iter()
            .flat_map(std::fs::read_dir)
            .flatten()
            .flatten()
            .filter_map(|f| {
                let meta = f.metadata().ok()?;
                if meta.permissions().mode() & 0o001 > 0 {
                    let p = f.path();
                    p.file_name()?;
                    Some(p)
                } else {
                    None
                }
            })
            .collect();

        bins.sort_by_key(|b| Reverse(usage.entry_count(b.to_str().unwrap())));

        Self { bins, term, usage }
    }

    pub fn eval(&mut self, idx: usize) -> std::convert::Infallible {
        let binary = self.bins[idx].to_str().unwrap();

        self.usage.increment_entry_usage(binary.to_string());
        self.usage.try_update_cache(CACHE_PATH);

        self.term.push(CString::new(binary).unwrap());
        let (prog, args) = (&self.term[0], &self.term[0..]);

        log::debug!("executing command: {:?} {:?}", prog, args);
        nix::unistd::execvp(prog, args).unwrap_or_else(|e| {
            panic!(
                "failed to launch command line `{:?} {:?}`: {}",
                prog, args, e
            )
        });

        unreachable!()
    }

    pub fn entries_len(&self) -> usize {
        self.bins.len()
    }

    pub fn entry(&self, idx: usize) -> Entry<'_> {
        Entry {
            name: self.bins[idx].file_name().and_then(|s| s.to_str()).unwrap(),
            icon: None,
        }
    }

    pub fn text_entries(&self) -> impl Iterator<Item = &str> + super::ExactSizeIterator {
        self.bins
            .iter()
            .map(|e| e.file_name().and_then(|s| s.to_str()).unwrap())
    }
}
