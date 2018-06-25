//! Pattern traits.

use haystack::{Haystack, Hay, SharedSpan};

use std::ops::Range;

pub unsafe trait Searcher: Sized {
    type Hay: Hay + ?Sized;

    fn search(
        &mut self,
        span: SharedSpan<'_, Self::Hay>,
    ) -> Option<Range<<Self::Hay as Hay>::Index>>;
}

pub unsafe trait Checker: Sized {
    type Hay: Hay + ?Sized;

    fn is_prefix_of(self, hay: &Self::Hay) -> bool;

    fn trim_start(&mut self, hay: &Self::Hay) -> <Self::Hay as Hay>::Index;
}

pub unsafe trait ReverseSearcher: Searcher {
    fn rsearch(
        &mut self,
        span: SharedSpan<'_, Self::Hay>,
    ) -> Option<Range<<Self::Hay as Hay>::Index>>;
}

pub unsafe trait ReverseChecker: Checker {
    fn is_suffix_of(self, hay: &Self::Hay) -> bool;

    fn trim_end(&mut self, hay: &Self::Hay) -> <Self::Hay as Hay>::Index;
}

pub unsafe trait DoubleEndedSearcher: ReverseSearcher {}

pub unsafe trait DoubleEndedChecker: ReverseChecker {}

/// A pattern
pub trait Pattern<H: Haystack>: Sized {
    type Searcher: Searcher<Hay = H::Hay>;
    type Checker: Checker<Hay = H::Hay>;

    fn into_searcher(self) -> Self::Searcher;

    fn into_checker(self) -> Self::Checker;
}
