use haystack::{Haystack, HaystackMut};
use pattern::{Pattern, ReversePattern};
use cursor::{SharedOrigin, MutOrigin};
use span::{Span};
use std::ops::Range;
use std::iter::{Rev, Map, FusedIterator, Once, once};
use std::fmt;

//------------------------------------------------------------------------------
// Iterator wrappers
//------------------------------------------------------------------------------

macro_rules! wrap_iterators {
    ($(
        yield $item:ty:
        pub struct $name:ident($($wrapped:tt)*);
        pub struct $r_name:ident($($r_wrapped:tt)*);

        yield $m_item:ty:
        pub struct $m_name:ident($($m_wrapped:tt)*);
        pub struct $rm_name:ident($($rm_wrapped:tt)*);
    )+) => {$(
        pub struct $name<'h, H, P>($($wrapped)*)
        where
            H: Haystack + ?Sized + 'h,
            P: Pattern<'h, H>,
        ;

        pub struct $r_name<'h, H, P>($($r_wrapped)*)
        where
            H: Haystack + ?Sized + 'h,
            P: ReversePattern<'h, H>,
        ;

        pub struct $m_name<'h, H, P>($($m_wrapped)*)
        where
            H: HaystackMut + ?Sized + 'h,
            P: Pattern<'h, H>,
        ;

        pub struct $rm_name<'h, H, P>($($rm_wrapped)*)
        where
            H: HaystackMut + ?Sized + 'h,
            P: ReversePattern<'h, H>,
        ;

        wrap_iterators! { @clone $name Pattern Searcher }
        wrap_iterators! { @clone $r_name ReversePattern ReverseSearcher }
        wrap_iterators! { @impl $name Pattern Searcher Haystack $item }
        wrap_iterators! { @impl $r_name ReversePattern ReverseSearcher Haystack $item }
        wrap_iterators! { @impl $m_name Pattern Searcher HaystackMut $m_item }
        wrap_iterators! { @impl $rm_name ReversePattern ReverseSearcher HaystackMut $m_item }
    )+};

    (@clone $name:ident $pattern:ident $searcher:ident) => {
        impl<'h, H, P> Clone for $name<'h, H, P>
        where
            H: Haystack + ?Sized,
            P: $pattern<'h, H>,
            P::$searcher: Clone,
        {
            fn clone(&self) -> Self {
                $name(self.0.clone())
            }
            fn clone_from(&mut self, source: &Self) {
                self.0.clone_from(&source.0);
            }
        }
    };

    (@impl $name:ident $pattern:ident $searcher:ident $haystack:ident $item:ty) => {
        impl<'h, H, P> fmt::Debug for $name<'h, H, P>
        where
            H: $haystack + ?Sized,
            P: $pattern<'h, H>,
            P::$searcher: fmt::Debug,
        {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.debug_tuple(stringify!($name))
                    .field(&self.0)
                    .finish()
            }
        }

        impl<'h, H, P> Iterator for $name<'h, H, P>
        where
            H: $haystack + ?Sized + 'h,
            P: $pattern<'h, H>,
        {
            type Item = $item;
            fn next(&mut self) -> Option<Self::Item> {
                self.0.next()
            }
        }

        impl<'h, H, P> FusedIterator for $name<'h, H, P>
        where
            H: $haystack + ?Sized + 'h,
            P: $pattern<'h, H>,
        {}

        impl<'h, H, P> DoubleEndedIterator for $name<'h, H, P>
        where
            H: $haystack + ?Sized + 'h,
            P: $pattern<'h, H>,
            P::$searcher: DoubleEndedIterator
        {
            fn next_back(&mut self) -> Option<Self::Item> {
                self.0.next_back()
            }
        }
    };
}

//------------------------------------------------------------------------------
// Starts with / Ends with / Contains
//------------------------------------------------------------------------------

pub fn starts_with<'h, H, P>(haystack: &'h H, pattern: P) -> bool
where
    H: Haystack + ?Sized,
    P: Pattern<'h, H>,
{
    pattern.is_prefix_of(Span::new(haystack))
}

pub fn ends_with<'h, H, P>(haystack: &'h H, pattern: P) -> bool
where
    H: Haystack + ?Sized,
    P: ReversePattern<'h, H>,
{
    pattern.is_suffix_of(Span::new(haystack))
}

pub fn contains<'h, H, P>(haystack: &'h H, pattern: P) -> bool
where
    H: Haystack + ?Sized,
    P: Pattern<'h, H>,
{
    pattern.into_searcher(Span::new(haystack)).next().is_some()
}

//------------------------------------------------------------------------------
// Find / Find range
//------------------------------------------------------------------------------

pub fn find<'h, H, P>(haystack: &'h H, pattern: P) -> Option<usize>
where
    H: Haystack + ?Sized,
    P: Pattern<'h, H>,
{
    let origin = SharedOrigin::new(haystack);
    pattern.into_searcher(Span::new(haystack))
        .next()
        .map(|span| span.to_index(origin))
}

pub fn rfind<'h, H, P, M>(haystack: &'h H, pattern: P) -> Option<usize>
where
    H: Haystack + ?Sized,
    P: ReversePattern<'h, H>,
{
    let origin = SharedOrigin::new(haystack);
    pattern.into_reverse_searcher(Span::new(haystack))
        .next()
        .map(|span| span.to_index(origin))
}

pub fn find_range<'h, H, P, M>(haystack: &'h H, pattern: P) -> Option<Range<usize>>
where
    H: Haystack + ?Sized,
    P: Pattern<'h, H>,
{
    let origin = SharedOrigin::new(haystack);
    pattern.into_searcher(Span::new(haystack))
        .next()
        .map(|span| span.to_range(origin))
}

pub fn rfind_range<'h, H, P, M>(haystack: &'h H, pattern: P) -> Option<Range<usize>>
where
    H: Haystack + ?Sized,
    P: ReversePattern<'h, H>,
{
    let origin = SharedOrigin::new(haystack);
    pattern.into_reverse_searcher(Span::new(haystack))
        .next()
        .map(|span| span.to_range(origin))
}

//------------------------------------------------------------------------------
// Trim
//------------------------------------------------------------------------------

pub fn trim_start<'h, H, P>(haystack: &'h H, mut pattern: P) -> &'h H
where
    H: Haystack + ?Sized,
    P: Pattern<'h, H>,
{
    let (mut span, origin) = Span::and_origin(haystack);
    pattern.trim_start(&mut span);
    span.to_haystack(origin)
}

pub fn trim_end<'h, H, P>(haystack: &'h H, mut pattern: P) -> &'h H
where
    H: Haystack + ?Sized,
    P: ReversePattern<'h, H>,
{
    let (mut span, origin) = Span::and_origin(haystack);
    pattern.trim_end(&mut span);
    span.to_haystack(origin)
}

pub fn trim<'h, H, P>(haystack: &'h H, mut pattern: P) -> &'h H
where
    H: Haystack + ?Sized,
    P: ReversePattern<'h, H, ReverseSearcher = Rev<<P as Pattern<'h, H>>::Searcher>>,
    P::Searcher: DoubleEndedIterator,
{
    let (mut span, origin) = Span::and_origin(haystack);
    pattern.trim_start(&mut span);
    pattern.trim_end(&mut span);
    span.to_haystack(origin)
}

pub fn trim_start_mut<'h, H, P>(haystack: &'h mut H, mut pattern: P) -> &'h mut H
where
    H: HaystackMut + ?Sized,
    P: Pattern<'h, H>,
{
    let (mut span, origin) = Span::and_origin_mut(haystack);
    pattern.trim_start(&mut span);
    span.to_haystack_mut(origin)
}

pub fn trim_end_mut<'h, H, P>(haystack: &'h mut H, mut pattern: P) -> &'h mut H
where
    H: HaystackMut + ?Sized,
    P: ReversePattern<'h, H>,
{
    let (mut span, origin) = Span::and_origin_mut(haystack);
    pattern.trim_end(&mut span);
    span.to_haystack_mut(origin)
}

pub fn trim_mut<'h, H, P>(haystack: &'h mut H, mut pattern: P) -> &'h mut H
where
    H: HaystackMut + ?Sized,
    P: ReversePattern<'h, H, ReverseSearcher = Rev<<P as Pattern<'h, H>>::Searcher>>,
    P::Searcher: DoubleEndedIterator,
{
    let (mut span, origin) = Span::and_origin_mut(haystack);
    pattern.trim_start(&mut span);
    pattern.trim_end(&mut span);
    span.to_haystack_mut(origin)
}

//------------------------------------------------------------------------------
// Matches
//------------------------------------------------------------------------------

pub fn matches<'h, H, P>(haystack: &'h H, pattern: P) -> Matches<'h, H, P>
where
    H: Haystack + ?Sized,
    P: Pattern<'h, H>,
{
    let (span, origin) = Span::and_origin(haystack);
    Matches(pattern.into_searcher(span).map(ToHaystack(origin)))
}

pub fn rmatches<'h, H, P>(haystack: &'h H, pattern: P) -> RMatches<'h, H, P>
where
    H: Haystack + ?Sized,
    P: ReversePattern<'h, H>,
{
    let (span, origin) = Span::and_origin(haystack);
    RMatches(pattern.into_reverse_searcher(span).map(ToHaystack(origin)))
}

pub fn match_indices<'h, H, P>(haystack: &'h H, pattern: P) -> MatchIndices<'h, H, P>
where
    H: Haystack + ?Sized,
    P: Pattern<'h, H>,
{
    let (span, origin) = Span::and_origin(haystack);
    MatchIndices(pattern.into_searcher(span).map(ToHaystackIndex(origin)))
}

pub fn rmatch_indices<'h, H, P>(haystack: &'h H, pattern: P) -> RMatchIndices<'h, H, P>
where
    H: Haystack + ?Sized,
    P: ReversePattern<'h, H>,
{
    let (span, origin) = Span::and_origin(haystack);
    RMatchIndices(pattern.into_reverse_searcher(span).map(ToHaystackIndex(origin)))
}

pub fn match_ranges<'h, H, P>(haystack: &'h H, pattern: P) -> MatchRanges<'h, H, P>
where
    H: Haystack + ?Sized,
    P: Pattern<'h, H>,
{
    let (span, origin) = Span::and_origin(haystack);
    MatchRanges(pattern.into_searcher(span).map(ToHaystackRange(origin)))
}

pub fn rmatch_ranges<'h, H, P>(haystack: &'h H, pattern: P) -> RMatchRanges<'h, H, P>
where
    H: Haystack + ?Sized,
    P: ReversePattern<'h, H>,
{
    let (span, origin) = Span::and_origin(haystack);
    RMatchRanges(pattern.into_reverse_searcher(span).map(ToHaystackRange(origin)))
}

pub fn matches_mut<'h, H, P>(haystack: &'h mut H, pattern: P) -> MatchesMut<'h, H, P>
where
    H: HaystackMut + ?Sized,
    P: Pattern<'h, H>,
{
    let (span, origin) = Span::and_origin_mut(haystack);
    MatchesMut(pattern.into_searcher(span).map(ToHaystackMut(origin)))
}

pub fn rmatches_mut<'h, H, P>(haystack: &'h mut H, pattern: P) -> RMatchesMut<'h, H, P>
where
    H: HaystackMut + ?Sized,
    P: ReversePattern<'h, H>,
{
    let (span, origin) = Span::and_origin_mut(haystack);
    RMatchesMut(pattern.into_reverse_searcher(span).map(ToHaystackMut(origin)))
}

pub fn match_indices_mut<'h, H, P>(haystack: &'h mut H, pattern: P) -> MatchIndicesMut<'h, H, P>
where
    H: HaystackMut + ?Sized,
    P: Pattern<'h, H>,
{
    let (span, origin) = Span::and_origin_mut(haystack);
    MatchIndicesMut(pattern.into_searcher(span).map(ToHaystackIndexMut(origin)))
}

pub fn rmatch_indices_mut<'h, H, P>(haystack: &'h mut H, pattern: P) -> RMatchIndicesMut<'h, H, P>
where
    H: HaystackMut + ?Sized,
    P: ReversePattern<'h, H>,
{
    let (span, origin) = Span::and_origin_mut(haystack);
    RMatchIndicesMut(pattern.into_reverse_searcher(span).map(ToHaystackIndexMut(origin)))
}

pub fn match_ranges_mut<'h, H, P>(haystack: &'h mut H, pattern: P) -> MatchRangesMut<'h, H, P>
where
    H: HaystackMut + ?Sized,
    P: Pattern<'h, H>,
{
    let (span, origin) = Span::and_origin_mut(haystack);
    MatchRangesMut(pattern.into_searcher(span).map(ToHaystackRangeMut(origin)))
}

pub fn rmatch_ranges_mut<'h, H, P>(haystack: &'h mut H, pattern: P) -> RMatchRangesMut<'h, H, P>
where
    H: HaystackMut + ?Sized,
    P: ReversePattern<'h, H>,
{
    let (span, origin) = Span::and_origin_mut(haystack);
    RMatchRangesMut(pattern.into_reverse_searcher(span).map(ToHaystackRangeMut(origin)))
}


struct ToHaystack<'h, H>(SharedOrigin<'h, H>)
where
    H: Haystack + ?Sized + 'h;

struct ToHaystackIndex<'h, H>(SharedOrigin<'h, H>)
where
    H: Haystack + ?Sized + 'h;

struct ToHaystackRange<'h, H>(SharedOrigin<'h, H>)
where
    H: Haystack + ?Sized + 'h;

struct ToHaystackMut<'h, H>(MutOrigin<'h, H>)
where
    H: HaystackMut + ?Sized + 'h;

struct ToHaystackIndexMut<'h, H>(MutOrigin<'h, H>)
where
    H: HaystackMut + ?Sized + 'h;

struct ToHaystackRangeMut<'h, H>(MutOrigin<'h, H>)
where
    H: HaystackMut + ?Sized + 'h;

impl_unboxed_functions! {
    [<'h, H: Haystack + ?Sized>]
    ToHaystack<'h, H> = |&self, span: Span<'h, H>| -> &'h H {
        span.to_haystack(self.0)
    }

    [<'h, H: Haystack + ?Sized>]
    ToHaystackIndex<'h, H> = |&self, span: Span<'h, H>| -> (usize, &'h H) {
        (span.to_index(self.0), span.to_haystack(self.0))
    }

    [<'h, H: Haystack + ?Sized>]
    ToHaystackRange<'h, H> = |&self, span: Span<'h, H>| -> (Range<usize>, &'h H) {
        (span.to_range(self.0), span.to_haystack(self.0))
    }

    [<'h, H: HaystackMut + ?Sized>]
    ToHaystackMut<'h, H> = |&self, span: Span<'h, H>| -> &'h mut H {
        span.to_haystack_mut(self.0)
    }

    [<'h, H: HaystackMut + ?Sized>]
    ToHaystackIndexMut<'h, H> = |&self, span: Span<'h, H>| -> (usize, &'h mut H) {
        (span.to_index(self.0), span.to_haystack_mut(self.0))
    }

    [<'h, H: HaystackMut + ?Sized>]
    ToHaystackRangeMut<'h, H> = |&self, span: Span<'h, H>| -> (Range<usize>, &'h mut H) {
        (span.to_range(self.0), span.to_haystack_mut(self.0))
    }
}

wrap_iterators! {
    yield (&'h H):
    pub struct Matches(Map<P::Searcher, ToHaystack<'h, H>>);
    pub struct RMatches(Map<P::ReverseSearcher, ToHaystack<'h, H>>);

    yield (&'h mut H):
    pub struct MatchesMut(Map<P::Searcher, ToHaystackMut<'h, H>>);
    pub struct RMatchesMut(Map<P::ReverseSearcher, ToHaystackMut<'h, H>>);

    yield (usize, &'h H):
    pub struct MatchIndices(Map<P::Searcher, ToHaystackIndex<'h, H>>);
    pub struct RMatchIndices(Map<P::ReverseSearcher, ToHaystackIndex<'h, H>>);

    yield (usize, &'h mut H):
    pub struct MatchIndicesMut(Map<P::Searcher, ToHaystackIndexMut<'h, H>>);
    pub struct RMatchIndicesMut(Map<P::ReverseSearcher, ToHaystackIndexMut<'h, H>>);

    yield (Range<usize>, &'h H):
    pub struct MatchRanges(Map<P::Searcher, ToHaystackRange<'h, H>>);
    pub struct RMatchRanges(Map<P::ReverseSearcher, ToHaystackRange<'h, H>>);

    yield (Range<usize>, &'h mut H):
    pub struct MatchRangesMut(Map<P::Searcher, ToHaystackRangeMut<'h, H>>);
    pub struct RMatchRangesMut(Map<P::ReverseSearcher, ToHaystackRangeMut<'h, H>>);
}

//------------------------------------------------------------------------------
// Splits
//------------------------------------------------------------------------------

/*

struct SplitInternal<H, S>
where
    H: Haystack + ?Sized,
{
    remaining: Span<H>,
    searcher: S,
    reversed: bool,
    allow_trailing_empty: bool,
    finished: bool,
}

impl<H, S> Clone for SplitInternal<H, S>
where
    H: Haystack + ?Sized,
    S: Clone,
{
    fn clone(&self) -> Self {
        SplitInternal {
            remaining: self.remaining,
            searcher: self.searcher.clone(),
            reversed: self.reversed,
            allow_trailing_empty: self.allow_trailing_empty,
            finished: self.finished,
        }
    }
}

impl<H, S> fmt::Debug for SplitInternal<H, S>
where
    H: Haystack + ?Sized,
    H::StartCursor: fmt::Debug,
    H::EndCursor: fmt::Debug,
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SplitInternal")
            .field("remaining", &self.remaining)
            .field("searcher", &self.searcher)
            .field("reversed", &self.reversed)
            .field("allow_trailing_empty", &self.allow_trailing_empty)
            .field("finished", &self.finished)
            .finish()
    }
}

impl<H, S> SplitInternal<H, S>
where
    H: Haystack + ?Sized,
    S: Iterator<Item = Span<H>>,
{
    #[inline]
    fn get_end(&mut self) -> Option<Span<H>> {
        if self.finished {
            return None;
        }
        if !self.allow_trailing_empty && self.remaining.is_empty() {
            return None;
        }
        self.finished = true;
        Some(self.remaining)
    }

    fn get_next<F>(&mut self, reversed: bool, f: F) -> Option<Span<H>>
    where
        F: (FnOnce(&mut S) -> Option<Span<H>>) + Copy
    {
        if self.finished {
            return None;
        }

        if reversed && !self.allow_trailing_empty {
            self.allow_trailing_empty = true;
            if let Some(span) = self.get_next(reversed, f) {
                if !span.is_empty() {
                    return Some(span);
                }
            }
            if self.finished {
                return None;
            }
        }

        if let Some(span) = f(&mut self.searcher) {
            let (left, _, right) = self.remaining.split_twice(span.start(), span.end());
            if reversed {
                self.remaining = left;
                Some(right)
            } else {
                self.remaining = right;
                Some(left)
            }
        } else {
            if reversed {
                self.finished = true;
                Some(self.remaining)
            } else {
                self.get_end()
            }
        }
    }
}

impl<H, S> Iterator for SplitInternal<H, S>
where
    H: Haystack + ?Sized,
    S: Iterator<Item = Span<H>>,
{
    type Item = Span<H>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let reversed = self.reversed;
        self.get_next(reversed, S::next)
    }
}

impl<H, S> FusedIterator for SplitInternal<H, S>
where
    H: Haystack + ?Sized,
    S: Iterator<Item = Span<H>>,
{}

impl<H, S> DoubleEndedIterator for SplitInternal<H, S>
where
    H: Haystack + ?Sized,
    S: DoubleEndedIterator<Item = Span<H>>,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let reversed = self.reversed;
        self.get_next(!reversed, S::next_back)
    }
}

*/

// pub fn split<'h, H, P>(haystack: &'h H, pattern: P) -> Split<'h, H, P>
// where
//     H: Haystack + ?Sized,
//     P: Pattern<'h, M, H>,
// {
//     Split(SplitInternal {
//         remaining: Span::full(haystack),
//         searcher: pattern.into_searcher(haystack),
//         reversed: false,
//         allow_trailing_empty: true,
//         finished: false,
//     }.map(IntoHaystack))
// }

