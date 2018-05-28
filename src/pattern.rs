//! Pattern traits.

use span::Span;
use std::iter::FusedIterator;
use haystack::Haystack;

// Axioms:
//
// 1. `p == start_to_end_cursor(end_to_start_cursor(p))`
// 2. `p == end_to_start_cursor(start_to_end_cursor(p))`
// 3. `start_cursor_to_offset(p) == end_cursor_to_offset(start_to_end_cursor(p))`
// 4. `end_cursor_to_offset(p) == start_cursor_to_offset(end_to_start_cursor(p))`
// 5. If `start_cursor_to_offset(b) == end_cursor_to_offset(e)`, then `b.eq_or_before(e)`
// 6. `cursor_range().0 == start_to_end_cursor(cursor_range().0)`
// 7. `cursor_range().1 == end_to_start_cursor(cursor_range().1)`



/// A pattern
// FIXME: The trait should be `Pattern<H>` with `Searcher` being a GAT.
pub unsafe trait Pattern<'h, H>: Sized
where
    H: Haystack + ?Sized,
{
    type Searcher: FusedIterator<Item = Span<'h, H>>;

    fn into_searcher(self, span: Span<'h, H>) -> Self::Searcher;

    fn is_prefix_of(self, span: Span<'_, H>) -> bool;

    fn trim_start(&mut self, span: &mut Span<'_, H>);
}

/// A pattern which can be searched from backward.
///
/// # Double-ended pattern
///
/// A pattern is "double-ended" when its searcher implements
/// [`DoubleEndedIterator`]. We consider a pattern `P` is double-ended when the
/// following conditions are met:
///
/// 1. `P::Searcher` implements `DoubleEndedIterator`
/// 2. `P::ReverseSearcher` is exactly `Rev<P::Searcher>`
/// 3. `P::into_reverse_searcher` is implemented exactly as
///     `self.into_searcher(span).rev()`.
pub unsafe trait ReversePattern<'h, H>: Pattern<'h, H>
where
    H: Haystack + ?Sized,
{
    type ReverseSearcher: FusedIterator<Item = Span<'h, H>>;

    fn into_reverse_searcher(self, span: Span<'h, H>) -> Self::ReverseSearcher;

    fn is_suffix_of(self, span: Span<'_, H>) -> bool;

    fn trim_end(&mut self, span: &mut Span<'_, H>);
}
