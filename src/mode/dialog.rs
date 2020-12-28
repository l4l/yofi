use std::io::{BufRead, BufReader};

use super::{Entry, EvalInfo};

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

    pub fn eval(&mut self, info: EvalInfo<'_>) -> std::convert::Infallible {
        let idx = info.index.expect("invalid index");
        println!("{}", &self.lines[idx]);
        std::process::exit(0);
    }

    pub fn entries_len(&self) -> usize {
        self.lines.len()
    }

    pub fn entry(&self, idx: usize) -> Entry<'_> {
        Entry {
            name: self.lines[idx].as_str(),
            icon: None,
        }
    }

    pub fn text_entries(&self) -> impl Iterator<Item = &str> + super::ExactSizeIterator {
        self.lines.iter().map(|e| e.as_str())
    }
}
