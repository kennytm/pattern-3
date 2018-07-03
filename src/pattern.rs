//! Pattern traits.

use haystack::{Haystack, Hay, Span};

use std::ops::Range;

/// A searcher for a [`Pattern`].
///
/// This trait provides methods for searching for non-overlapping matches of a
/// pattern starting from the front (left) of a hay.
///
/// # Safety
///
/// This trait is marked unsafe because the range returned by the
/// [`.search()`](Searcher::search) method is required to lie on valid codeword
/// boundaries in the haystack. This enables consumers of this trait to slice
/// the haystack without additional runtime checks.
pub unsafe trait Searcher<A: Hay + ?Sized> {
    /// Searches for the first range which the pattern can be found in the span.
    ///
    /// The hay and the restricted range for searching can be recovered by
    /// calling `span`[`.into_parts()`](Span::into_parts). The returned range
    /// should be relative to the hay and must be contained within the
    /// restricted range from the span.
    ///
    /// If the pattern is not found, this method should return `None`.
    fn search(&mut self, span: Span<&A>) -> Option<Range<A::Index>>;
}

/// A checker for a [`Pattern`].
///
/// This trait provides methods for checking if a pattern matches the beginning
/// of a hay.
///
/// # Safety
///
/// This trait is marked unsafe because the indices returned by
/// [`.check()`](Checker::check) and [`.trim_start()`](Checker::trim_start)
/// methods are required to lie on valid codeward boundaries in the haystack.
/// This enables consumers of this trait to slice the haystack without
/// additional runtime checks.
pub unsafe trait Checker<A: Hay + ?Sized> {
    /// Checks if the pattern can be found at the beginning of the span.
    ///
    /// The hay and the restricted range for searching can be recovered by
    /// calling `span`[`.into_parts()`](Span::into_parts). If a pattern can be
    /// found starting at `range.start`, this method should return the end index
    /// of the pattern relative to the hay.
    ///
    /// If the pattern cannot be found at the beginning of the span, this method
    /// should return `None`.
    fn check(&mut self, span: Span<&A>) -> Option<A::Index>;

    /// Repeatedly removes prefixes of the hay which matches the pattern.
    ///
    /// A fast generic implementation in terms of [`.check()`] is provided by
    /// default. Nevertheless, many patterns allow a higher-performance
    /// specialization.
    #[inline]
    fn trim_start(&mut self, hay: &A) -> A::Index {
        let mut offset = hay.start_index();
        let mut span = Span::from(hay);
        while let Some(pos) = self.check(span.clone()) {
            offset = pos;
            let (hay, range) = span.into_parts();
            if pos == range.start {
                break;
            }
            span = unsafe { Span::from_parts(hay, pos..range.end) };
        }
        offset
    }
}

/// A searcher which can be searched from the end.
///
/// This trait provides methods for searching for non-overlapping matches of a
/// pattern starting from the back (right) of a hay.
pub unsafe trait ReverseSearcher<A: Hay + ?Sized>: Searcher<A> {
    /// Searches for the last range which the pattern can be found in the span.
    ///
    /// The hay and the restricted range for searching can be recovered by
    /// calling `span`[`.into_parts()`](Span::into_parts). The returned range
    /// should be relative to the hay and must be contained within the
    /// restricted range from the span.
    ///
    /// If the pattern is not found, this method should return `None`.
    fn rsearch(&mut self, span: Span<&A>) -> Option<Range<A::Index>>;
}

/// A checker for the end of a hay.
///
/// This trait provides methods for checking if a pattern matches the end of a
/// hay.
pub unsafe trait ReverseChecker<A: Hay + ?Sized>: Checker<A> {
    /// Checks if the pattern can be found at the end of the span.
    ///
    /// The hay and the restricted range for searching can be recovered by
    /// calling `span`[`.into_parts()`](Span::into_parts). If a pattern can be
    /// found ending at `range.end`, this method should return the start index
    /// of the pattern relative to the hay.
    ///
    /// If the pattern cannot be found at the end of the span, this method
    /// should return `None`.
    fn rcheck(&mut self, hay: Span<&A>) -> Option<A::Index>;

    /// Repeatedly removes suffixes of the hay which matches the pattern.
    ///
    /// A fast generic implementation in terms of [`.rcheck()`] is provided by
    /// default. Nevertheless, many patterns allow a higher-performance
    /// specialization.
    #[inline]
    fn trim_end(&mut self, hay: &A) -> A::Index {
        let mut offset = hay.end_index();
        let mut span = Span::from(hay);
        while let Some(pos) = self.rcheck(span.clone()) {
            offset = pos;
            let (hay, range) = span.into_parts();
            if pos == range.end {
                break;
            }
            span = unsafe { Span::from_parts(hay, range.start..pos) };
        }
        offset
    }
}

pub unsafe trait DoubleEndedSearcher<A: Hay + ?Sized>: ReverseSearcher<A> {}

pub unsafe trait DoubleEndedChecker<A: Hay + ?Sized>: ReverseChecker<A> {}

/// A pattern.
pub trait Pattern<H: Haystack>: Sized {
    /// The searcher associated with this pattern.
    type Searcher: Searcher<H::Hay>;

    /// The checker associated with this pattern.
    type Checker: Checker<H::Hay>;

    /// Produces a searcher for this pattern.
    fn into_searcher(self) -> Self::Searcher;

    /// Produces a checker for this pattern.
    fn into_checker(self) -> Self::Checker;
}


/// Searcher of an empty pattern.
///
/// This searcher will find all empty subslices between any codewords in a
/// haystack.
#[derive(Clone, Debug, Default)]
pub struct EmptySearcher {
    consumed_start: bool,
    consumed_end: bool,
}

unsafe impl<A: Hay + ?Sized> Searcher<A> for EmptySearcher {
    #[inline]
    fn search(&mut self, span: Span<&A>) -> Option<Range<A::Index>> {
        let (hay, range) = span.into_parts();
        let start = if !self.consumed_start {
            self.consumed_start = true;
            range.start
        } else if range.start == range.end {
            return None;
        } else {
            unsafe { hay.next_index(range.start) }
        };
        Some(start..start)
    }
}

unsafe impl<A: Hay + ?Sized> Checker<A> for EmptySearcher {
    #[inline]
    fn check(&mut self, span: Span<&A>) -> Option<A::Index> {
        let (_, range) = span.into_parts();
        Some(range.start)
    }

    #[inline]
    fn trim_start(&mut self, hay: &A) -> A::Index {
        hay.start_index()
    }
}

unsafe impl<A: Hay + ?Sized> ReverseSearcher<A> for EmptySearcher {
    #[inline]
    fn rsearch(&mut self, span: Span<&A>) -> Option<Range<A::Index>> {
        let (hay, range) = span.into_parts();
        let end = if !self.consumed_end {
            self.consumed_end = true;
            range.end
        } else if range.start == range.end {
            return None;
        } else {
            unsafe { hay.prev_index(range.end) }
        };
        Some(end..end)
    }
}

unsafe impl<A: Hay + ?Sized> ReverseChecker<A> for EmptySearcher {
    #[inline]
    fn rcheck(&mut self, span: Span<&A>) -> Option<A::Index> {
        let (_, range) = span.into_parts();
        Some(range.end)
    }

    #[inline]
    fn trim_end(&mut self, hay: &A) -> A::Index {
        hay.end_index()
    }
}

unsafe impl<A: Hay + ?Sized> DoubleEndedSearcher<A> for EmptySearcher {}
unsafe impl<A: Hay + ?Sized> DoubleEndedChecker<A> for EmptySearcher {}
