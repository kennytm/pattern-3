//! Pattern traits.

use haystack::Hay;

use std::ops::Range;

pub unsafe trait Searcher: Sized {
    type Hay: Hay + ?Sized;

    fn search(&mut self, hay: &Self::Hay) -> Option<Range<<Self::Hay as Hay>::Index>>;
}

pub unsafe trait ReverseSearcher: Searcher {
    fn rsearch(&mut self, hay: &Self::Hay) -> Option<Range<<Self::Hay as Hay>::Index>>;
}

pub unsafe trait DoubleEndedSearcher: ReverseSearcher {}

/// A pattern
pub trait Pattern<H>: Sized
where
    H: Hay + ?Sized,
{
    type Searcher: Searcher<Hay = H>;

    fn into_searcher(self) -> Self::Searcher;

    fn is_prefix_of(self, hay: &H) -> bool;

    fn trim_start(&mut self, hay: &H) -> H::Index;

    fn is_suffix_of(self, hay: &H) -> bool
    where
        Self::Searcher: ReverseSearcher;

    fn trim_end(&mut self, haystack: &H) -> H::Index
    where
        Self::Searcher: ReverseSearcher;
}
