/*

extern crate pattern_3;
extern crate regex;

use regex::Regex;
use pattern_3::{Pattern, Searcher};
use pattern_3::ext::match_ranges;
use std::ops::Range;

struct UnanchoredRegex(Regex);

struct UnanchoredRegexSearcher<'p> {
    regex: &'p Regex,
    allow_empty_match: bool,
}

unsafe impl<'p> Searcher for UnanchoredRegexSearcher<'p> {
    type Hay = str;

    fn search(&mut self, hay: &str) -> Option<Range<usize>> {
        let mut st = 0;
        while st <= hay.len() {
            let m = self.regex.find_at(hay, st)?;
            if m.end() == st {
                if !self.allow_empty_match {
                    self.allow_empty_match = true;
                    st += hay[st..].chars().next()?.len_utf8();
                    continue;
                }
            }

            self.allow_empty_match = false;
            return Some(m.start()..m.end());
        }
        None
    }
}

impl<'p> Pattern<str> for &'p UnanchoredRegex {
    type Searcher = UnanchoredRegexSearcher<'p>;

    fn into_searcher(self) -> Self::Searcher {
        UnanchoredRegexSearcher {
            regex: &self.0,
            allow_empty_match: true,
        }
    }

    fn is_prefix_of(self, hay: &str) -> bool {
        self.0.find(hay).map(|m| m.start()) == Some(0)
    }

    fn trim_start(&mut self, hay: &str) -> usize {
        let mut start = 0;
        for m in self.0.find_iter(hay) {
            if m.start() == start {
                start = m.end();
            } else {
                break;
            }
        }
        start
    }

    // FIXME: Because of issue #20021 we have to supply these two impls
    fn is_suffix_of(self, _: &str) -> bool {
        unreachable!()
    }
    fn trim_end(&mut self, _: &str) -> usize {
        unreachable!()
    }
}


fn do_test(re: &str, haystack: &str, expected: &[(Range<usize>, &str)]) {
    let pattern = UnanchoredRegex(Regex::new(re).unwrap());
    assert_eq!(match_ranges(haystack, &pattern).collect::<Vec<_>>(), expected);
}

#[test]
fn searcher_empty_regex_empty_haystack() {
    do_test(r"", "", &[
        (0..0, ""),
    ]);
}

#[test]
fn searcher_empty_regex() {
    do_test(r"", "ab", &[
        (0..0, ""),
        (1..1, ""),
        (2..2, ""),
    ]);
}

#[test]
fn searcher_empty_haystack() {
    do_test(r"\d", "", &[]);
}

#[test]
fn searcher_one_match() {
    do_test(r"\d", "5", &[
        (0..1, "5"),
    ]);
}

#[test]
fn searcher_no_match() {
    do_test(r"\d", "a", &[]);
}

#[test]
fn searcher_two_adjacent_matches() {
    do_test(r"\d", "56", &[
        (0..1, "5"),
        (1..2, "6"),
    ]);
}

#[test]
fn searcher_two_non_adjacent_matches() {
    do_test(r"\d", "5a6", &[
        (0..1, "5"),
        (2..3, "6"),
    ]);
}

#[test]
fn searcher_reject_first() {
    do_test(r"\d", "a6", &[
        (1..2, "6"),
    ]);
}

#[test]
fn searcher_one_zero_length_matches() {
    do_test(r"\d*", "a1b2", &[
        (0..0, ""),
        (1..2, "1"),
        (3..4, "2"),
    ]);
}

#[test]
fn searcher_many_zero_length_matches() {
    do_test(r"\d*", "a1bbb2", &[
        (0..0, ""),
        (1..2, "1"),
        (3..3, ""),
        (4..4, ""),
        (5..6, "2"),
    ]);
}

#[test]
fn searcher_unicode() {
    do_test(r".+?", "Ⅰ1Ⅱ2", &[
        (0..3, "Ⅰ"),
        (3..4, "1"),
        (4..7, "Ⅱ"),
        (7..8, "2"),
    ]);
}

// Note that `UnanchoredRegex` would ignore all anchors (e.g. `^` and `\b`) at
// the beginning. To demonstrate:
#[test]
fn bug_unanchored() {
    do_test(r"^a", "aaa", &[
        (0..1, "a"),
        (1..2, "a"),
        (2..3, "a"),
    ]);
}

*/
