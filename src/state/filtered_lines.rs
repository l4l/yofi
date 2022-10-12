use either::Either;
use sublime_fuzzy::Match;

use crate::draw::ListItem;
use crate::mode::Mode;

pub type ContinuousMatch<'a> = sublime_fuzzy::ContinuousMatches<'a>;

pub struct FilteredLines(Either<Vec<(usize, Match)>, usize>);

impl FilteredLines {
    pub fn searched<'a>(entries: impl Iterator<Item = &'a str>, search_string: &str) -> Self {
        let mut v = entries
            .enumerate()
            .filter_map(|(i, e)| Some((i, sublime_fuzzy::best_match(search_string, e)?)))
            .collect::<Vec<_>>();
        v.sort_by_key(|(_, m)| std::cmp::Reverse(m.score()));
        Self(Either::Left(v))
    }

    pub fn unfiltred(len: usize) -> Self {
        Self(Either::Right(len))
    }

    pub fn len(&self) -> usize {
        match self {
            Self(Either::Left(x)) => x.len(),
            Self(Either::Right(x)) => *x,
        }
    }

    pub fn index(&self, selected_item: usize) -> Option<usize> {
        if self.len() == 0 {
            return None;
        }

        if selected_item >= self.len() {
            panic!("Internal error: selected_item overflow");
        }

        Some(match self {
            Self(Either::Left(x)) => x[selected_item].0,
            Self(Either::Right(_)) => selected_item,
        })
    }

    pub fn list_items<'s, 'm: 's>(
        &'s self,
        mode: &'m Mode,
        item: usize,
        subitem: usize,
    ) -> impl Iterator<Item = ListItem<'_>> + '_ {
        match self {
            Self(Either::Left(x)) => {
                Either::Left(x.iter().enumerate().map(move |(idx, (item_idx, s_match))| {
                    let e = mode.entry(*item_idx, if idx == item { subitem } else { 0 });
                    ListItem {
                        name: e.name,
                        subname: e.subname,
                        icon: e.icon,
                        match_mask: Some(s_match.continuous_matches()),
                    }
                }))
            }
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
