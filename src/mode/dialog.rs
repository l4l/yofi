use std::io::{BufRead, BufReader};

use crate::draw::ListItem;

pub struct DialogMode {
    lines: Vec<String>,
}

impl DialogMode {
    pub fn new() -> Self {
        let stdin = std::io::stdin();
        let rdr = stdin.lock();

        Self {
            lines: BufReader::new(rdr)
                .lines()
                .collect::<Result<_, _>>()
                .expect("Failed to read stdin"),
        }
    }

    pub fn eval(&mut self, idx: usize) -> std::convert::Infallible {
        println!("{}", &self.lines[idx]);
        std::process::exit(0);
    }

    pub fn entries_len(&self) -> usize {
        self.lines.len()
    }

    pub fn list_item(&self, idx: usize) -> ListItem<'_> {
        ListItem {
            name: self.lines[idx].as_str(),
            icon: None,
        }
    }

    pub fn text_entries(&self) -> impl Iterator<Item = &str> {
        self.lines.iter().map(|e| e.as_str())
    }
}
