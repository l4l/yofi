use either::Either;
use fzyr::LocateResult;

use crate::draw::ListItem;
use crate::input::KeyPress;
use crate::input_parser::InputValue;
use crate::mode::{EvalInfo, Mode};

struct Preprocessed(Either<Vec<LocateResult>, usize>);

impl Preprocessed {
    fn processed(processed: Vec<LocateResult>) -> Self {
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

    fn index(&self, selected_item: usize) -> Option<usize> {
        if self.len() == 0 {
            return None;
        }

        if selected_item >= self.len() {
            panic!("Internal error: selected_item overflow");
        }

        Some(match self {
            Self(Either::Left(x)) => x[selected_item].candidate_index,
            Self(Either::Right(_)) => selected_item,
        })
    }

    fn list_items<'s, 'm: 's>(
        &'s self,
        mode: &'m Mode,
        item: usize,
        subitem: usize,
    ) -> impl Iterator<Item = ListItem<'_>> + '_ {
        match self {
            Self(Either::Left(x)) => Either::Left(x.iter().enumerate().map(move |(idx, r)| {
                let e = mode.entry(r.candidate_index, if idx == item { subitem } else { 0 });
                ListItem {
                    name: e.name,
                    subname: e.subname,
                    icon: e.icon,
                    match_mask: Some(&r.match_mask),
                }
            })),
            Self(Either::Right(x)) => Either::Right((0..*x).enumerate().map(move |(idx, i)| {
                let e = mode.entry(i, if idx == item { subitem } else { 0 });
                ListItem {
                    name: e.name,
                    subname: e.subname,
                    icon: e.icon,
                    match_mask: None,
                }
            })),
        }
        .into_iter()
    }
}

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

        let parsed = crate::input_parser::parser(&self.raw_input)
            .map(|(left, cmd)| {
                if !left.is_empty() {
                    log::error!(
                        "Non-terminating parse, cmd: {:?}, left: {:?}",
                        self.raw_input,
                        left
                    );
                }
                cmd
            })
            .unwrap_or_else(|e| {
                log::error!("failed to parse command {:?}: {}", self.raw_input, e);
                crate::input_parser::InputValue {
                    source: &self.raw_input,
                    search_string: &self.raw_input,
                    args: None,
                    env_vars: None,
                    workind_dir: None,
                }
            });

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
    preprocessed: Preprocessed,
    inner: Mode,
}

impl State {
    pub fn new(inner: Mode) -> Self {
        Self {
            input_buffer: InputBuffer::new(),
            skip_offset: 0,
            selected_item: 0,
            selected_subitem: 0,
            preprocessed: Preprocessed::unfiltred(inner.entries_len()),
            inner,
        }
    }

    pub fn process_event(&mut self, event: KeyPress) -> bool {
        use sctk::seat::keyboard::keysyms;

        match dbg!(event) {
            KeyPress {
                keysym: keysyms::XKB_KEY_Escape,
                ..
            }
            | KeyPress {
                keysym: keysyms::XKB_KEY_c,
                ctrl: true,
                ..
            } => return true,
            KeyPress {
                keysym: keysyms::XKB_KEY_BackSpace,
                ctrl: false,
                ..
            } => self.input_buffer.update_input(|input| {
                input.pop();
            }),
            KeyPress {
                keysym: keysyms::XKB_KEY_Up,
                ..
            }
            | KeyPress {
                keysym: keysyms::XKB_KEY_k,
                ctrl: true,
                ..
            }
            | KeyPress {
                keysym: keysyms::XKB_KEY_Tab,
                shift: true,
                ..
            }
            | KeyPress {
                keysym: keysyms::XKB_KEY_ISO_Left_Tab,
                shift: true,
                ..
            }
            | KeyPress {
                keysym: keysyms::XKB_KEY_KP_Tab,
                shift: true,
                ..
            } => {
                self.selected_subitem = 0;
                self.selected_item = self.selected_item.saturating_sub(1);
            }
            KeyPress {
                keysym: keysyms::XKB_KEY_Down,
                ..
            }
            | KeyPress {
                keysym: keysyms::XKB_KEY_j,
                ctrl: true,
                ..
            }
            | KeyPress {
                keysym: keysyms::XKB_KEY_Tab,
                ..
            }
            | KeyPress {
                keysym: keysyms::XKB_KEY_ISO_Left_Tab,
                ..
            }
            | KeyPress {
                keysym: keysyms::XKB_KEY_KP_Tab,
                ..
            } => {
                self.selected_subitem = 0;
                self.selected_item = self
                    .inner
                    .entries_len()
                    .saturating_sub(1)
                    .min(self.selected_item + 1);
            }
            KeyPress {
                keysym: keysyms::XKB_KEY_Left,
                ..
            } => self.selected_subitem = self.selected_subitem.saturating_sub(1),
            KeyPress {
                keysym: keysyms::XKB_KEY_Right,
                ..
            } => {
                self.selected_subitem = self
                    .inner
                    .subentries_len(self.preprocessed.index(self.selected_item).unwrap_or(0))
                    .min(self.selected_subitem + 1)
            }
            KeyPress {
                keysym: keysyms::XKB_KEY_Return,
                ..
            } => {
                let info = EvalInfo {
                    index: self.preprocessed.index(self.selected_item),
                    subindex: self.selected_subitem,
                    input_value: self.input_buffer.parsed_input(),
                };
                self.inner.eval(info);
            }
            KeyPress {
                keysym: keysyms::XKB_KEY_bracketright,
                ctrl: true,
                ..
            } => self.input_buffer.update_input(|input| input.clear()),
            KeyPress {
                keysym: keysyms::XKB_KEY_w,
                ctrl: true,
                ..
            }
            | KeyPress {
                keysym: keysyms::XKB_KEY_BackSpace,
                ctrl: true,
                ..
            } => self.input_buffer.update_input(|input| {
                if let Some(pos) = input.rfind(|x: char| !x.is_alphanumeric()) {
                    input.truncate(pos);
                } else {
                    input.clear();
                }
            }),
            KeyPress {
                sym: Some(sym),
                ctrl,
                ..
            } if !sym.is_control() && !ctrl => self.input_buffer.update_input(|input| {
                println!("add `{sym}`");
                input.push(sym);
            }),
            event => log::debug!("unhandled sym: {:?} (ctrl: {})", event.sym, event.ctrl),
        }

        false
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

    pub fn processed_entries(&self) -> impl Iterator<Item = ListItem<'_>> {
        self.preprocessed
            .list_items(&self.inner, self.selected_item, self.selected_subitem)
    }

    pub fn process_entries(&mut self) {
        self.preprocessed = if self.input_buffer.search_string().is_empty() {
            Preprocessed::unfiltred(self.inner.entries_len())
        } else {
            Preprocessed::processed(fzyr::locate_serial(
                self.input_buffer.search_string(),
                self.inner.text_entries(),
            ))
        };

        self.selected_item = self
            .preprocessed
            .len()
            .saturating_sub(1)
            .min(self.selected_item);
    }
}
