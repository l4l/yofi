use std::cmp::Reverse;
use std::collections::HashMap;
use std::ffi::CString;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::rc::Rc;

use super::{Entry, EvalInfo};
use crate::usage_cache::Usage;

const CACHE_PATH: &str = concat!(crate::prog_name!(), ".bincache");

pub struct BinsMode {
    bins: Vec<Rc<Path>>,
    entry_name_cache: HashMap<Rc<Path>, String>,
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
                if meta.is_file() && meta.permissions().mode() & 0o001 > 0 {
                    let p = f.path();
                    p.file_name()?;
                    Some(p)
                } else {
                    None
                }
            })
            .map(Rc::<Path>::from)
            .collect();

        bins.sort_by(|x, y| {
            let x_usage_count = usage.entry_count(x.to_str().unwrap());
            let y_usage_count = usage.entry_count(y.to_str().unwrap());

            Reverse(x_usage_count)
                .cmp(&Reverse(y_usage_count))
                .then_with(|| x.cmp(y))
        });
        bins.dedup();

        let mut fname_counts = HashMap::<_, u8>::new();

        for b in &bins {
            let count = fname_counts.entry(b.file_name().unwrap()).or_default();
            *count = count.saturating_add(1);
        }

        let mut entry_name_cache = HashMap::new();

        for bin in &bins {
            let fname = bin.file_name().unwrap();
            if fname_counts.get(fname).filter(|&&cnt| cnt > 1).is_none() {
                continue;
            }

            let fname_str = fname.to_str().unwrap();

            entry_name_cache.insert(
                Rc::clone(bin),
                format!("{} ({})", fname_str, bin.to_str().unwrap()),
            );
        }

        Self {
            bins,
            entry_name_cache,
            term,
            usage,
        }
    }

    pub fn eval(&mut self, info: EvalInfo<'_>) -> std::convert::Infallible {
        let binary = if let Some(idx) = info.index {
            self.bins[idx].to_str().unwrap()
        } else {
            info.search_string
        };

        self.usage.increment_entry_usage(binary.to_string());
        self.usage.try_update_cache(CACHE_PATH);

        crate::exec::exec(
            Some(std::mem::take(&mut self.term)),
            vec![CString::new(binary).expect("invalid binary")],
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
        let fname = bin.file_name().unwrap();

        let name = if let Some(name) = self.entry_name_cache.get(bin) {
            name.as_str()
        } else {
            fname.to_str().unwrap()
        };

        Entry {
            name,
            subname: None,
            icon: None,
        }
    }

    pub fn text_entries(&self) -> impl Iterator<Item = &str> + super::ExactSizeIterator {
        self.bins
            .iter()
            .map(|e| e.file_name().and_then(|s| s.to_str()).unwrap())
    }
}
