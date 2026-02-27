use anyhow::{Context, Result};

use super::{Entry, EvalInfo};

pub struct DialogMode {
    lines: Vec<String>,
}

impl DialogMode {
    pub fn new() -> Result<Self> {
        std::io::stdin()
            .lines()
            .collect::<Result<_, _>>()
            .context("failed to read stdin")
            .map(|lines| Self { lines })
    }

    pub fn from_lines(lines: Vec<String>) -> Self {
        Self { lines }
    }

    pub fn eval(&mut self, info: EvalInfo<'_>) -> Result<std::convert::Infallible> {
        let value = info
            .index
            .and_then(|idx| Some(self.lines.get(idx)?.as_str()))
            .unwrap_or(info.input_value.source);
        println!("{value}");
        std::process::exit(0);
    }

    pub fn entries_len(&self) -> usize {
        self.lines.len()
    }

    pub fn subentries_len(&self, _: usize) -> usize {
        0
    }

    pub fn entry(&self, idx: usize, _: usize) -> Entry<'_> {
        Entry {
            name: self.lines[idx].as_ref(),
            subname: None,
            icon: None,
        }
    }

    pub fn text_entries(&self) -> impl super::ExactSizeIterator<Item = &str> {
        self.lines.iter().map(|e| e.as_str())
    }
}
