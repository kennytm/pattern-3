extern crate pattern_3;
extern crate regex;

use regex::Regex;
use pattern_3::*;
use pattern_3::ext::{match_ranges, trim_start};
use std::ops::Range;

struct RegexWrapper(Regex);

struct RegexSearcher<'p> {
    regex: &'p Regex,
    allow_empty_match: bool,
}

unsafe impl<'p> Searcher<str> for RegexSearcher<'p> {
    fn search(&mut self, span: Span<&str>) -> Option<Range<usize>> {
        let (hay, range) = span.into_parts();
        let mut st = range.start;
        while st <= range.end {
            let m = self.regex.find_at(hay, st)?;
            if m.end() == st {
                if !self.allow_empty_match {
                    self.allow_empty_match = true;
                    if st == range.end {
                        return None;
                    }
                    st = unsafe { hay.next_index(st) };
                    continue;
                }
            }
            self.allow_empty_match = false;
            return Some(m.start()..m.end());
        }
        None
    }

    fn consume(&mut self, span: Span<&str>) -> Option<usize> {
        let (hay, range) = span.into_parts();
        let m = self.regex.find_at(hay, range.start)?;
        if m.start() == range.start {
            Some(m.end())
        } else {
            None
        }
    }
}

impl<'h, 'p> Pattern<&'h str> for &'p RegexWrapper {
    type Searcher = RegexSearcher<'p>;

    fn into_searcher(self) -> RegexSearcher<'p> {
        RegexSearcher {
            regex: &self.0,
            allow_empty_match: true,
        }
    }
}

fn do_test(re: &str, haystack: &str, expected: &[(Range<usize>, &str)]) {
    let pattern = RegexWrapper(Regex::new(re).unwrap());
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

#[test]
fn test_anchored() {
    do_test(r"^a", "aaa", &[
        (0..1, "a"),
    ]);
}

#[test]
fn test_word_bounary() {
    do_test(r"\b", "hello::world", &[
        (0..0, ""),
        (5..5, ""),
        (7..7, ""),
        (12..12, ""),
    ]);
}

#[test]
fn test_trim_start() {
    assert_eq!(trim_start("aaabbb", &RegexWrapper(Regex::new("a*").unwrap())), "bbb");
    assert_eq!(trim_start("aaabbb", &RegexWrapper(Regex::new("^a").unwrap())), "aabbb");
}
