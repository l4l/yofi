use std::borrow::Borrow;
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub struct Usage(HashMap<String, usize>);

impl Usage {
    pub fn from_path(path: impl AsRef<Path>) -> Self {
        let usage = crate::desktop::xdg_dirs()
            .place_cache_file(path)
            .and_then(File::open)
            .map_err(|e| log::error!("cannot open cache file: {}", e))
            .map(BufReader::new)
            .into_iter()
            .flat_map(|rdr| {
                rdr.lines()
                    .filter(|l| l.as_ref().map(|l| !l.is_empty()).unwrap_or(true))
                    .map(|l| {
                        let line = l.map_err(|e| log::error!("unable to read the line: {}", e))?;
                        let mut iter = line.split(' ');
                        let (count, entry) = (iter.next().ok_or(())?, iter.next().ok_or(())?);

                        let count = count.parse().map_err(|e| {
                            log::error!("unable to parse count (\"{}\"): {}", count, e)
                        })?;

                        Ok((entry.to_string(), count))
                    })
            })
            .collect::<Result<_, ()>>()
            .unwrap_or_default();

        Self(usage)
    }

    pub fn entry_count<Q: ?Sized>(&self, entry: &Q) -> usize
    where
        String: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.0.get(entry).copied().unwrap_or(0)
    }

    pub fn increment_entry_usage(&mut self, entry: String) {
        *self.0.entry(entry).or_default() += 1;
    }

    pub fn try_update_cache(&self, path: impl AsRef<Path>) {
        if let Err(e) = crate::desktop::xdg_dirs()
            .place_cache_file(path)
            .and_then(File::create)
            .and_then(|mut f| {
                let mut buf = vec![];

                for (entry, count) in &self.0 {
                    let s = format!("{} ", count);
                    buf.extend(s.as_bytes());
                    buf.extend(entry.as_bytes());
                    buf.push(b'\n');
                }

                f.write_all(&buf)
            })
        {
            log::error!("failed to update cache: {}", e);
        }
    }
}
