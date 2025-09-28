use crate::draw::ListItem;
use crate::input_parser::InputValue;
use crate::mode::{EvalInfo, Mode};
pub use filtered_lines::ContinuousMatch;
use filtered_lines::FilteredLines;

mod filtered_lines;

struct InputBuffer {
    raw_input: String,
    parsed_input: InputValue<'static>,
}

impl InputBuffer {
    pub fn new() -> Self {
        Self {
            raw_input: String::new(),
            parsed_input: InputValue::empty(),
        }
    }

    pub fn update_input(&mut self, f: impl FnOnce(&mut String)) {
        f(&mut self.raw_input);

        let parsed = crate::input_parser::parse(&self.raw_input);

        // This transmute is needed for extending `raw_input` lifetime
        // to a static one thus making it possible to cache parsed result.
        // Safety: this is safe, because it's internal structure invariant
        // that `parsed_input` never outlives `raw_input`, nor used after
        // its update.
        self.parsed_input = unsafe { std::mem::transmute(parsed) };
    }

    pub fn raw_input(&self) -> &str {
        self.raw_input.as_str()
    }

    pub fn parsed_input<'a>(&self) -> &InputValue<'a> {
        &self.parsed_input
    }

    pub fn search_string(&self) -> &str {
        self.parsed_input.search_string
    }
}

pub struct State {
    input_buffer: InputBuffer,
    skip_offset: usize,
    selected_item: usize,
    selected_subitem: usize,
    filtered_lines: FilteredLines,
    inner: Mode,
}

impl State {
    pub fn new(inner: Mode) -> Self {
        Self {
            input_buffer: InputBuffer::new(),
            skip_offset: 0,
            selected_item: 0,
            selected_subitem: 0,
            filtered_lines: FilteredLines::unfiltred(inner.entries_len()),
            inner,
        }
    }

    pub fn remove_input_char(&mut self) {
        self.input_buffer.update_input(|input| {
            input.pop();
        })
    }

    pub fn remove_input_word(&mut self) {
        self.input_buffer.update_input(|input| {
            if let Some(pos) = input.rfind(|x: char| !x.is_alphanumeric()) {
                input.truncate(pos);
            } else {
                input.clear();
            }
        })
    }

    pub fn append_to_input(&mut self, s: &str) {
        self.input_buffer.update_input(|input| input.push_str(s))
    }

    pub fn clear_input(&mut self) {
        self.input_buffer.update_input(|input| input.clear())
    }

    pub fn eval_input(&mut self, with_fork: bool) -> anyhow::Result<()> {
        let info = EvalInfo {
            index: self.filtered_lines.index(self.selected_item),
            subindex: self.selected_subitem,
            input_value: self.input_buffer.parsed_input(),
        };
        if with_fork {
            self.inner.fork_eval(info)
        } else {
            self.inner
                .eval(info)
                .map(|_: std::convert::Infallible| unreachable!())
        }
    }

    pub fn next_item(&mut self) {
        self.selected_subitem = 0;
        self.selected_item = self
            .inner
            .entries_len()
            .saturating_sub(1)
            .min(self.selected_item + 1);
    }

    pub fn prev_item(&mut self) {
        self.selected_subitem = 0;
        self.selected_item = self.selected_item.saturating_sub(1);
    }

    pub fn next_subitem(&mut self) {
        self.selected_subitem = self
            .inner
            .subentries_len(self.filtered_lines.index(self.selected_item).unwrap_or(0))
            .min(self.selected_subitem + 1)
    }

    pub fn prev_subitem(&mut self) {
        self.selected_subitem = self.selected_subitem.saturating_sub(1)
    }

    pub fn raw_input(&self) -> &str {
        self.input_buffer.raw_input()
    }

    pub fn skip_offset(&self) -> usize {
        self.skip_offset
    }

    pub fn update_skip_offset(&mut self, x: usize) {
        self.skip_offset = x;
    }

    pub fn selected_item(&self) -> usize {
        self.selected_item
    }

    pub fn has_subitems(&self) -> bool {
        // For now either all items has subname or none.
        self.inner.subentries_len(self.selected_item) > 0
    }

    pub fn processed_entries(&self) -> impl ExactSizeIterator<Item = ListItem<'_>> {
        self.filtered_lines
            .list_items(&self.inner, self.selected_item, self.selected_subitem)
    }

    pub fn process_entries(&mut self) {
        self.filtered_lines = if self.input_buffer.search_string().is_empty() {
            FilteredLines::unfiltred(self.inner.entries_len())
        } else {
            FilteredLines::searched(self.inner.text_entries(), self.input_buffer.search_string())
        };

        self.selected_item = self
            .filtered_lines
            .len()
            .saturating_sub(1)
            .min(self.selected_item);
    }
}
