use std::cmp::Reverse;
use std::collections::HashMap;
use std::ffi::CString;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

use super::Entry;
use crate::{desktop, DesktopEntry};

pub struct AppsMode {
    entries: Vec<DesktopEntry>,
    term: Vec<CString>,
    usage: HashMap<String, usize>,
}

impl AppsMode {
    pub fn new(mut entries: Vec<DesktopEntry>, term: Vec<CString>) -> Self {
        let usage: HashMap<String, usize> = desktop::xdg_dirs()
            .place_cache_file(concat!(crate::prog_name!(), ".cache"))
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
                        let (count, progname) = (iter.next().ok_or(())?, iter.next().ok_or(())?);

                        let count = count.parse().map_err(|e| {
                            log::error!("unable to parse count (\"{}\"): {}", count, e)
                        })?;

                        Ok((progname.to_string(), count))
                    })
            })
            .collect::<Result<_, ()>>()
            .unwrap_or_default();

        entries.sort_by_key(|e| Reverse(usage.get(&e.desktop_fname).unwrap_or(&0)));

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

        *self.usage.entry(entry.desktop_fname.clone()).or_default() += 1;

        if let Err(e) = desktop::xdg_dirs()
            .place_cache_file(concat!(crate::prog_name!(), ".cache"))
            .and_then(File::create)
            .and_then(|mut f| {
                let mut buf = vec![];

                for (progname, count) in &self.usage {
                    let s = format!("{} ", count);
                    buf.extend(s.as_bytes());
                    buf.extend(progname.as_bytes());
                    buf.push(b'\n');
                }

                f.write_all(&buf)
            })
        {
            log::error!("failed to update cache: {}", e);
        }

        log::debug!("executing command: {:?} {:?}", prog, args);
        nix::unistd::execvp(prog, args).unwrap();

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
