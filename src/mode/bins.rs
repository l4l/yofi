use std::cmp::Reverse;
use std::collections::HashMap;
use std::ffi::CString;
use std::os::unix::fs::PermissionsExt;

use anyhow::{Context, Result};

use super::{Entry, EvalInfo};
use crate::usage_cache::Usage;

const CACHE_PATH: &str = concat!(crate::prog_name!(), ".bincache");

#[derive(PartialEq, Eq, Hash)]
struct Binary {
    path: String,
    fname: String,
}

pub struct BinsMode {
    bins: Vec<Binary>,
    entry_name_cache: HashMap<String, String>,
    term: Vec<CString>,
    usage: Usage,
}

impl BinsMode {
    pub fn new(term: Vec<CString>) -> Self {
        let usage = Usage::from_path(CACHE_PATH);

        let paths = std::env::var("PATH")
            .map(|paths| paths.split(':').map(|s| s.to_owned()).collect())
            .unwrap_or_else(|_| vec!["/usr/bin".into()]);
        let mut bins: Vec<_> = paths
            .into_iter()
            .flat_map(|p| std::fs::read_dir(&p).map_err(|e| log::warn!("failed to read {p}: {e}")))
            .flatten()
            .flatten()
            .filter_map(|f| {
                let meta = f.metadata().ok()?;
                if f.path().is_file() && meta.permissions().mode() & 0o001 > 0 {
                    let p = f.path();
                    Some(Binary {
                        path: p.to_str()?.to_owned(),
                        fname: p.file_name()?.to_str()?.to_owned(),
                    })
                } else {
                    None
                }
            })
            .collect();

        bins.sort_by(|x, y| {
            let x_usage_count = usage.entry_count(&x.path);
            let y_usage_count = usage.entry_count(&y.path);

            Reverse(x_usage_count)
                .cmp(&Reverse(y_usage_count))
                .then_with(|| x.path.cmp(&y.path))
        });
        bins.dedup();

        let mut fname_counts = HashMap::<_, u8>::new();

        for b in &bins {
            let count = fname_counts.entry(b.fname.clone()).or_default();
            *count = count.saturating_add(1);
        }

        let mut entry_name_cache = HashMap::new();

        for bin in &bins {
            if fname_counts
                .get(&bin.fname)
                .filter(|&&cnt| cnt > 1)
                .is_none()
            {
                log::warn!("file name {} found multiple times, skipping", bin.fname);
                continue;
            }

            entry_name_cache.insert(bin.path.clone(), format!("{} ({})", bin.fname, bin.path));
        }

        Self {
            bins,
            entry_name_cache,
            term,
            usage,
        }
    }

    pub fn eval(&mut self, info: EvalInfo<'_>) -> Result<std::convert::Infallible> {
        let binary = if let Some(idx) = info.index {
            self.bins[idx].path.as_str()
        } else {
            info.search_string
        };

        self.usage.increment_entry_usage(binary.to_string());
        self.usage.try_update_cache(CACHE_PATH);

        crate::exec::exec(
            Some(std::mem::take(&mut self.term)),
            std::iter::once(CString::new(binary).context("invalid binary name")?),
            info.input_value,
        )
    }

    pub fn entries_len(&self) -> usize {
        self.bins.len()
    }

    pub fn subentries_len(&self, _: usize) -> usize {
        0
    }

    pub fn entry(&self, idx: usize, _: usize) -> Entry<'_> {
        let bin = &self.bins[idx];

        let name = if let Some(name) = self.entry_name_cache.get(&bin.path) {
            name.as_str()
        } else {
            bin.fname.as_str()
        };

        Entry {
            name,
            subname: None,
            icon: None,
        }
    }

    pub fn text_entries(&self) -> impl super::ExactSizeIterator<Item = &str> {
        self.bins.iter().map(|e| e.fname.as_str())
    }
}
