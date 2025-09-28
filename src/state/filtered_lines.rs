use either::Either;
use sublime_fuzzy::Match;

use crate::draw::ListItem;
use crate::mode::Mode;

pub type ContinuousMatch<'a> = sublime_fuzzy::ContinuousMatches<'a>;

pub struct FilteredLines(Either<Vec<(usize, Match)>, usize>);

fn order_items(m1: &Match, m2: &Match) -> std::cmp::Ordering {
    m2.score()
        .cmp(&m1.score())
        .then_with(|| m1.matched_indices().cmp(m2.matched_indices()))
}

impl FilteredLines {
    pub fn searched<'a>(entries: impl Iterator<Item = &'a str>, search_string: &str) -> Self {
        let mut v = entries
            .enumerate()
            .filter_map(|(i, e)| Some((i, sublime_fuzzy::best_match(search_string, e)?)))
            .collect::<Vec<_>>();
        v.sort_by(|(_, m1), (_, m2)| order_items(m1, m2));
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

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use test_case::test_case;

    use super::*;

    #[test_case(vec!["asd"], "asd", vec!["asd"])]
    #[test_case(vec!["xy", "yx"], "xy", vec!["xy"])]
    #[test_case(vec!["xy", "yx"], "x", vec!["xy", "yx"])]
    #[test_case(vec!["xy", "yx"], "y", vec!["yx", "xy"])]
    #[test_case(vec!["ab-cd", "ac-bd"], "cd", vec!["ab-cd", "ac-bd"])]
    #[test_case(vec!["ab-cd", "cd-ab"], "ab", vec!["ab-cd", "cd-ab"])]
    fn test_order_items(input: Vec<&str>, query: &str, expected: Vec<&str>) {
        let result = input
            .into_iter()
            .filter_map(|x| Some(dbg!(x, sublime_fuzzy::best_match(query, x)?)))
            .sorted_by(|(_, m1), (_, m2)| order_items(m1, m2))
            .map(|(x, _)| (x))
            .collect::<Vec<_>>();

        assert_eq!(result, expected)
    }
}
