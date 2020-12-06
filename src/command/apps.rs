use std::ffi::CString;

use crate::draw::ListItem;
use crate::DesktopEntry;

pub struct AppsCommand {
    entries: Vec<DesktopEntry>,
    term: Vec<CString>,
}

impl AppsCommand {
    pub fn new(entries: Vec<DesktopEntry>, term: Vec<CString>) -> Self {
        Self { entries, term }
    }

    pub fn eval(&mut self, idx: usize) -> std::convert::Infallible {
        let entry = &self.entries[idx];
        let args = shlex::split(&entry.exec)
            .unwrap()
            .into_iter()
            .map(|s| CString::new(s).unwrap())
            .collect::<Vec<_>>();
        let (prog, args) = if entry.is_terminal {
            assert!(
                !self.term.is_empty(),
                "Cannot find terminal, specify `term` in config"
            );
            self.term.extend(args);
            (&self.term[0], &self.term[1..])
        } else {
            (&args[0], &args[1..])
        };
        log::debug!("executing command: {:?} {:?}", prog, args);
        nix::unistd::execvp(prog, args).unwrap();

        unreachable!()
    }

    pub fn entries_len(&self) -> usize {
        self.entries.len()
    }

    pub fn list_item(&self, idx: usize) -> ListItem<'_> {
        ListItem {
            name: self.entries[idx].name.as_str(),
        }
    }

    pub fn text_entries(&self) -> impl Iterator<Item = &str> {
        self.entries.iter().map(|e| e.name.as_str())
    }
}
