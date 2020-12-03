use fuse_rust::SearchResult;
use sctk::seat::keyboard::keysyms;

use crate::input::KeyPress;
use crate::DesktopEntry;

pub struct State {
    input_buf: String,
    selected_item: usize,
    processed_entries: Vec<SearchResult>,
    entries: Vec<DesktopEntry>,
}

impl State {
    pub fn from_entries(entries: Vec<DesktopEntry>) -> Self {
        Self {
            input_buf: String::new(),
            selected_item: 0,
            processed_entries: vec![],
            entries,
        }
    }

    pub fn process_event(&mut self, event: KeyPress) -> bool {
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
            } => self.selected_item = self.entries.len().min(self.selected_item + 1),
            KeyPress {
                keysym: keysyms::XKB_KEY_Return,
                ..
            } => {
                if self.selected_item >= self.processed_entries.len() {
                    return false;
                }
                let entry = &self.entries[self.processed_entries[self.selected_item].index];
                let args = shlex::split(&entry.exec)
                    .unwrap()
                    .into_iter()
                    .map(|s| std::ffi::CString::new(s).unwrap())
                    .collect::<Vec<_>>();
                nix::unistd::execvp(&args[0], &args[1..]).unwrap();
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

    pub fn processed_entries(&self) -> impl Iterator<Item = &DesktopEntry> {
        self.processed_entries
            .iter()
            .map(move |r| &self.entries[r.index])
    }

    pub fn process_entries(&mut self) {
        self.processed_entries = fuse_rust::Fuse::default().search_text_in_iterable(
            &self.input_buf,
            self.entries.iter().map(|e| e.name.as_str()),
        );

        self.selected_item = self
            .processed_entries
            .len()
            .saturating_sub(1)
            .min(self.selected_item);
    }
}
