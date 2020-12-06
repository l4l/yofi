use either::Either;
use fuse_rust::{Fuse, SearchResult};

use crate::draw::ListItem;
use crate::input::KeyPress;
use crate::mode::Mode;

struct Preprocessed(Either<Vec<SearchResult>, usize>);

impl Preprocessed {
    fn processed(processed: Vec<SearchResult>) -> Self {
        Self(Either::Left(processed))
    }

    fn unfiltred(len: usize) -> Self {
        Self(Either::Right(len))
    }

    fn len(&self) -> usize {
        match self {
            Self(Either::Left(x)) => x.len(),
            Self(Either::Right(x)) => *x,
        }
    }

    fn index(&self, selected_item: usize) -> usize {
        if selected_item >= self.len() {
            panic!("Internal error: selected_item overflow");
        }

        match self {
            Self(Either::Left(x)) => x[selected_item].index,
            Self(Either::Right(_)) => selected_item,
        }
    }

    fn list_items<'s, 'm: 's>(&'s self, mode: &'m Mode) -> impl Iterator<Item = ListItem<'_>> + '_ {
        match self {
            Self(Either::Left(x)) => Either::Left(x.iter().map(move |r| mode.list_item(r.index))),
            Self(Either::Right(x)) => Either::Right((0..*x).map(move |i| mode.list_item(i))),
        }
        .into_iter()
    }
}

pub struct State {
    input_buf: String,
    selected_item: usize,
    preprocessed: Preprocessed,
    inner: Mode,
}

impl State {
    pub fn new(inner: Mode) -> Self {
        Self {
            input_buf: String::new(),
            selected_item: 0,
            preprocessed: Preprocessed::unfiltred(inner.entries_len()),
            inner,
        }
    }

    pub fn process_event(&mut self, event: KeyPress) -> bool {
        use sctk::seat::keyboard::keysyms;

        match event {
            KeyPress {
                keysym: keysyms::XKB_KEY_Escape,
                ..
            } => return true,
            KeyPress {
                keysym: keysyms::XKB_KEY_BackSpace,
                ..
            } => {
                self.input_buf.pop();
            }
            KeyPress {
                keysym: keysyms::XKB_KEY_Up,
                ..
            } => self.selected_item = self.selected_item.saturating_sub(1),
            KeyPress {
                keysym: keysyms::XKB_KEY_Down,
                ..
            } => self.selected_item = self.inner.entries_len().min(self.selected_item + 1),
            KeyPress {
                keysym: keysyms::XKB_KEY_Return,
                ..
            } => {
                self.inner.eval(self.preprocessed.index(self.selected_item));
            }
            KeyPress {
                keysym: keysyms::XKB_KEY_bracketright,
                ctrl: true,
                ..
            } => self.input_buf.clear(),
            KeyPress { sym: Some(sym), .. } if !sym.is_control() && !event.ctrl => {
                self.input_buf.push(sym)
            }
            _ => log::debug!("unhandled sym: {:?} (ctrl: {})", event.sym, event.ctrl),
        }
        false
    }

    pub fn input_buf(&self) -> &str {
        &self.input_buf
    }

    pub fn selected_item(&self) -> usize {
        self.selected_item
    }

    pub fn processed_entries(&self) -> impl Iterator<Item = ListItem<'_>> {
        self.preprocessed.list_items(&self.inner)
    }

    pub fn process_entries(&mut self) {
        if self.input_buf.is_empty() {
            self.preprocessed = Preprocessed::unfiltred(self.inner.entries_len());
            return;
        }

        self.preprocessed = Preprocessed::processed(
            Fuse::default().search_text_in_iterable(&self.input_buf, self.inner.text_entries()),
        );

        self.selected_item = self
            .preprocessed
            .len()
            .saturating_sub(1)
            .min(self.selected_item);
    }
}
