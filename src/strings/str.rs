use pattern::*;
use haystack::SharedSpan;
use slices::slice::{TwoWaySearcher, SliceChecker};
use std::ops::Range;

#[derive(Clone, Debug, Default)]
struct EmptySearcher {
    consumed_start: bool,
    consumed_end: bool,
}

impl EmptySearcher {
    #[inline]
    fn next(&mut self, hay: &str, range: Range<usize>) -> Option<Range<usize>> {
        let mut start = range.start;
        if !self.consumed_start {
            self.consumed_start = true;
        } else {
            start += hay[range].chars().next()?.len_utf8();
        }
        Some(start..start)
    }

    #[inline]
    fn next_back(&mut self, hay: &str, range: Range<usize>) -> Option<Range<usize>> {
        let mut end = range.end;
        if !self.consumed_end {
            self.consumed_end = true;
        } else {
            end -= hay[range].chars().next_back()?.len_utf8();
        }
        Some(end..end)
    }
}

#[derive(Debug, Clone)]
enum StrSearcherImpl<'p> {
    TwoWay(TwoWaySearcher<'p, u8>),
    Empty(EmptySearcher),
}

#[derive(Debug, Clone)]
pub struct StrSearcher<'p>(StrSearcherImpl<'p>);

#[derive(Debug, Clone)]
pub struct StrChecker<'p>(&'p str);

unsafe impl<'p> Searcher for StrSearcher<'p> {
    type Hay = str;

    #[inline]
    fn search(&mut self, span: SharedSpan<'_, str>) -> Option<Range<usize>> {
        let (hay, range) = span.into_parts();
        match &mut self.0 {
            StrSearcherImpl::TwoWay(searcher) => searcher.next(hay.as_bytes(), range),
            StrSearcherImpl::Empty(searcher) => searcher.next(hay, range),
        }
    }
}

unsafe impl<'p> Checker for StrChecker<'p> {
    type Hay = str;

    #[inline]
    fn is_prefix_of(self, haystack: &str) -> bool {
        SliceChecker(self.0.as_bytes()).is_prefix_of(haystack.as_bytes())
    }

    #[inline]
    fn trim_start(&mut self, haystack: &str) -> usize {
        SliceChecker(self.0.as_bytes()).trim_start(haystack.as_bytes())
    }
}

unsafe impl<'p> ReverseSearcher for StrSearcher<'p> {
    #[inline]
    fn rsearch(&mut self, span: SharedSpan<'_, str>) -> Option<Range<usize>> {
        let (hay, range) = span.into_parts();
        match &mut self.0 {
            StrSearcherImpl::TwoWay(searcher) => searcher.next_back(hay.as_bytes(), range),
            StrSearcherImpl::Empty(searcher) => searcher.next_back(hay, range),
        }
    }
}

unsafe impl<'p> ReverseChecker for StrChecker<'p> {
    #[inline]
    fn is_suffix_of(self, haystack: &str) -> bool {
        SliceChecker(self.0.as_bytes()).is_suffix_of(haystack.as_bytes())
    }

    #[inline]
    fn trim_end(&mut self, haystack: &str) -> usize {
        SliceChecker(self.0.as_bytes()).trim_end(haystack.as_bytes())
    }
}

macro_rules! impl_pattern {
    (<[$($gen:tt)*]> ($ty:ty) for $pat:ty) => {
        impl<$($gen)*> Pattern<$ty> for $pat {
            type Searcher = StrSearcher<'p>;
            type Checker = StrChecker<'p>;

            #[inline]
            fn into_searcher(self) -> Self::Searcher {
                StrSearcher(if self.is_empty() {
                    StrSearcherImpl::Empty(EmptySearcher::default())
                } else {
                    StrSearcherImpl::TwoWay(TwoWaySearcher::new(self.as_bytes()))
                })
            }

            #[inline]
            fn into_checker(self) -> Self::Checker {
                StrChecker(self)
            }
        }
    }
}

impl_pattern!(<['h, 'p]> (&'h str) for &'p str);
impl_pattern!(<['h, 'p]> (&'h str) for &'p String);
impl_pattern!(<['h, 'q, 'p]> (&'h str) for &'q &'p str);
impl_pattern!(<['h, 'p]> (&'h mut str) for &'p str);
impl_pattern!(<['h, 'p]> (&'h mut str) for &'p String);
impl_pattern!(<['h, 'q, 'p]> (&'h mut str) for &'q &'p str);
