use haystack::{Hay, Haystack, Span};
use pattern::{Pattern, Searcher, ReverseSearcher, DoubleEndedSearcher, Checker, ReverseChecker, DoubleEndedChecker};
use std::iter::FusedIterator;
use std::ops::Range;
use std::fmt;

macro_rules! generate_clone_and_debug {
    ($name:ident, $field:tt) => {
        impl<H, S> Clone for $name<H, S>
        where
            H: Haystack + Clone,
            H::Span: Clone,
            S: Clone,
        {
            fn clone(&self) -> Self {
                $name { $field: self.$field.clone() }
            }
            fn clone_from(&mut self, src: &Self) {
                self.$field.clone_from(&src.$field);
            }
        }

        impl<H, S> fmt::Debug for $name<H, S>
        where
            H: Haystack + fmt::Debug,
            H::Span: fmt::Debug,
            S: fmt::Debug,
        {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.debug_tuple(stringify!($name))
                    .field(&self.$field)
                    .finish()
            }
        }
    }
}

macro_rules! generate_pattern_iterators {
    {
        // Forward iterator
        forward:
            $(#[$forward_iterator_attribute:meta])*
            struct $forward_iterator:ident;

        // Reverse iterator
        reverse:
            $(#[$reverse_iterator_attribute:meta])*
            struct $reverse_iterator:ident;

        // Stability of all generated items
        stability:
            $(#[$common_stability_attribute:meta])*

        // Internal almost-iterator that is being delegated to
        internal:
            $internal_iterator:ident yielding ($iterty:ty);

        // Kind of delegation - either single ended or double ended
        delegate $($t:tt)*
    } => {
        $(#[$forward_iterator_attribute])*
        $(#[$common_stability_attribute])*
        pub struct $forward_iterator<H, S>($internal_iterator<H, S>)
        where
            H: Haystack;

        generate_clone_and_debug!($forward_iterator, 0);

        $(#[$common_stability_attribute])*
        impl<H, S> Iterator for $forward_iterator<H, S>
        where
            H: Haystack,
            S: Searcher<Hay = H::Hay>,
        {
            type Item = $iterty;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                self.0.next()
            }
        }

        $(#[$reverse_iterator_attribute])*
        $(#[$common_stability_attribute])*
        pub struct $reverse_iterator<H, S>($internal_iterator<H, S>)
        where
            H: Haystack;

        generate_clone_and_debug!($reverse_iterator, 0);

        $(#[$common_stability_attribute])*
        impl<H, S> Iterator for $reverse_iterator<H, S>
        where
            H: Haystack,
            S: ReverseSearcher<Hay = H::Hay>,
        {
            type Item = $iterty;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                self.0.next_back()
            }
        }

        // #[stable(feature = "fused", since = "1.26.0")]
        impl<H, S> FusedIterator for $forward_iterator<H, S>
        where
            H: Haystack,
            S: Searcher<Hay = H::Hay>,
        {}

        // #[stable(feature = "fused", since = "1.26.0")]
        impl<H, S> FusedIterator for $reverse_iterator<H, S>
        where
            H: Haystack,
            S: ReverseSearcher<Hay = H::Hay>,
        {}

        generate_pattern_iterators!($($t)* with $(#[$common_stability_attribute])*,
                                                $forward_iterator,
                                                $reverse_iterator);
    };
    {
        double ended; with $(#[$common_stability_attribute:meta])*,
                           $forward_iterator:ident,
                           $reverse_iterator:ident
    } => {
        $(#[$common_stability_attribute])*
        impl<H, S> DoubleEndedIterator for $forward_iterator<H, S>
        where
            H: Haystack,
            S: DoubleEndedSearcher<Hay = H::Hay>,
        {
            #[inline]
            fn next_back(&mut self) -> Option<Self::Item> {
                self.0.next_back()
            }
        }

        $(#[$common_stability_attribute])*
        impl<H, S> DoubleEndedIterator for $reverse_iterator<H, S>
        where
            H: Haystack,
            S: DoubleEndedSearcher<Hay = H::Hay>,
        {
            #[inline]
            fn next_back(&mut self) -> Option<Self::Item> {
                self.0.next()
            }
        }
    };
    {
        single ended; with $(#[$common_stability_attribute:meta])*,
                           $forward_iterator:ident,
                           $reverse_iterator:ident
    } => {}
}

//------------------------------------------------------------------------------
// Starts with / Ends with
//------------------------------------------------------------------------------

pub fn starts_with<H, P>(haystack: H, pattern: P) -> bool
where
    H: Haystack,
    P: Pattern<H>,
{
    pattern.into_checker().is_prefix_of(haystack.borrow())
}

pub fn ends_with<H, P>(haystack: H, pattern: P) -> bool
where
    H: Haystack,
    P: Pattern<H>,
    P::Checker: ReverseChecker,
{
    pattern.into_checker().is_suffix_of(haystack.borrow())
}

//------------------------------------------------------------------------------
// Trim
//------------------------------------------------------------------------------

pub fn trim_start<H, P>(haystack: H, pattern: P) -> H
where
    H: Haystack,
    P: Pattern<H>,
{
    let range = {
        let hay = haystack.borrow();
        let start = pattern.into_checker().trim_start(hay);
        let end = hay.end_index();
        start..end
    };
    unsafe { haystack.slice_unchecked(range) }
}

pub fn trim_end<H, P>(haystack: H, pattern: P) -> H
where
    H: Haystack,
    P: Pattern<H>,
    P::Checker: ReverseChecker,
{
    let range = {
        let hay = haystack.borrow();
        let start = hay.start_index();
        let end = pattern.into_checker().trim_end(hay);
        start..end
    };
    unsafe { haystack.slice_unchecked(range) }
}

pub fn trim<H, P>(haystack: H, pattern: P) -> H
where
    H: Haystack,
    P: Pattern<H>,
    P::Checker: DoubleEndedChecker,
{
    let mut checker = pattern.into_checker();
    let range = {
        let hay = haystack.borrow();
        let end = checker.trim_end(hay);
        let hay = unsafe { Hay::slice_unchecked(hay, hay.start_index()..end) };
        let start = checker.trim_start(hay);
        start..end
    };
    unsafe { haystack.slice_unchecked(range) }
}

//------------------------------------------------------------------------------
// Matches
//------------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct MatchesInternal<H, S>
where
    H: Haystack,
{
    searcher: S,
    rest: H::Span,
}

impl<H, S> MatchesInternal<H, S>
where
    H: Haystack,
    S: Searcher<Hay = H::Hay>,
{
    #[inline]
    fn next_spanned(&mut self) -> Option<H::Span> {
        let rest = self.rest.take();
        let range = self.searcher.search(rest.borrow())?;
        let [_, middle, right] = rest.split_around(range);
        self.rest = right;
        Some(middle)
    }

    #[inline]
    fn next(&mut self) -> Option<H> {
        Some(Span::into(self.next_spanned()?))
    }
}

impl<H, S> MatchesInternal<H, S>
where
    H: Haystack,
    S: ReverseSearcher<Hay = H::Hay>,
{
    #[inline]
    fn next_back_spanned(&mut self) -> Option<H::Span> {
        let rest = self.rest.take();
        let range = self.searcher.rsearch(rest.borrow())?;
        let [left, middle, _] = rest.split_around(range);
        self.rest = left;
        Some(middle)
    }

    #[inline]
    fn next_back(&mut self) -> Option<H> {
        Some(Span::into(self.next_back_spanned()?))
    }
}

generate_pattern_iterators! {
    forward:
        struct Matches;
    reverse:
        struct RMatches;
    stability:
    internal:
        MatchesInternal yielding (H);
    delegate double ended;
}

pub fn matches<H, P>(haystack: H, pattern: P) -> Matches<H, P::Searcher>
where
    H: Haystack,
    P: Pattern<H>,
{
    Matches(MatchesInternal {
        searcher: pattern.into_searcher(),
        rest: haystack.into(),
    })
}

pub fn rmatches<H, P>(haystack: H, pattern: P) -> RMatches<H, P::Searcher>
where
    H: Haystack,
    P: Pattern<H>,
    P::Searcher: ReverseSearcher,
{
    RMatches(MatchesInternal {
        searcher: pattern.into_searcher(),
        rest: haystack.into(),
    })
}

pub fn contains<H, P>(haystack: H, pattern: P) -> bool
where
    H: Haystack,
    P: Pattern<H>,
{
    pattern.into_searcher()
        .search(H::Span::from(haystack).borrow())
        .is_some()
}

//------------------------------------------------------------------------------
// MatchIndices
//------------------------------------------------------------------------------

struct MatchIndicesInternal<H: Haystack, S> {
    inner: MatchesInternal<H, S>,
}

generate_clone_and_debug!(MatchIndicesInternal, inner);

impl<H, S> MatchIndicesInternal<H, S>
where
    H: Haystack,
    S: Searcher<Hay = H::Hay>,
{
    #[inline]
    fn next(&mut self) -> Option<(<H::Hay as Hay>::Index, H)> {
        let span = self.inner.next_spanned()?;
        let index = span.original_range().start;
        Some((index, Span::into(span)))
    }
}

impl<H, S> MatchIndicesInternal<H, S>
where
    H: Haystack,
    S: ReverseSearcher<Hay = H::Hay>,
{
    #[inline]
    fn next_back(&mut self) -> Option<(<H::Hay as Hay>::Index, H)> {
        let span = self.inner.next_back_spanned()?;
        let index = span.original_range().start;
        Some((index, Span::into(span)))
    }
}

generate_pattern_iterators! {
    forward:
        struct MatchIndices;
    reverse:
        struct RMatchIndices;
    stability:
    internal:
        MatchIndicesInternal yielding ((<H::Hay as Hay>::Index, H));
    delegate double ended;
}

pub fn match_indices<H, P>(haystack: H, pattern: P) -> MatchIndices<H, P::Searcher>
where
    H: Haystack,
    P: Pattern<H>,
{
    MatchIndices(MatchIndicesInternal {
        inner: matches(haystack, pattern).0,
    })
}

pub fn rmatch_indices<H, P>(haystack: H, pattern: P) -> RMatchIndices<H, P::Searcher>
where
    H: Haystack,
    P: Pattern<H>,
    P::Searcher: ReverseSearcher,
{
    RMatchIndices(MatchIndicesInternal {
        inner: rmatches(haystack, pattern).0,
    })
}

pub fn find<H, P>(haystack: H, pattern: P) -> Option<<H::Hay as Hay>::Index>
where
    H: Haystack,
    P: Pattern<H>,
{
    pattern.into_searcher()
        .search(H::Span::from(haystack).borrow())
        .map(|r| r.start)
}

pub fn rfind<H, P>(haystack: H, pattern: P) -> Option<<H::Hay as Hay>::Index>
where
    H: Haystack,
    P: Pattern<H>,
    P::Searcher: ReverseSearcher,
{
    pattern.into_searcher()
        .rsearch(H::Span::from(haystack).borrow())
        .map(|r| r.start)
}

//------------------------------------------------------------------------------
// MatchRanges
//------------------------------------------------------------------------------

struct MatchRangesInternal<H, S>
where
    H: Haystack,
{
    inner: MatchesInternal<H, S>,
}

generate_clone_and_debug!(MatchRangesInternal, inner);

impl<H, S> MatchRangesInternal<H, S>
where
    H: Haystack,
    S: Searcher<Hay = H::Hay>,
{
    #[inline]
    fn next(&mut self) -> Option<(Range<<H::Hay as Hay>::Index>, H)> {
        let span = self.inner.next_spanned()?;
        let range = span.original_range();
        Some((range, Span::into(span)))
    }
}

impl<H, S> MatchRangesInternal<H, S>
where
    H: Haystack,
    S: ReverseSearcher<Hay = H::Hay>,
{
    #[inline]
    fn next_back(&mut self) -> Option<(Range<<H::Hay as Hay>::Index>, H)> {
        let span = self.inner.next_back_spanned()?;
        let range = span.original_range();
        Some((range, Span::into(span)))
    }
}

generate_pattern_iterators! {
    forward:
        struct MatchRanges;
    reverse:
        struct RMatchRanges;
    stability:
    internal:
        MatchRangesInternal yielding ((Range<<H::Hay as Hay>::Index>, H));
    delegate double ended;
}

pub fn match_ranges<H, P>(haystack: H, pattern: P) -> MatchRanges<H, P::Searcher>
where
    H: Haystack,
    P: Pattern<H>,
{
    MatchRanges(MatchRangesInternal {
        inner: matches(haystack, pattern).0,
    })
}

pub fn rmatch_ranges<H, P>(haystack: H, pattern: P) -> RMatchRanges<H, P::Searcher>
where
    H: Haystack,
    P: Pattern<H>,
    P::Searcher: ReverseSearcher,
{
    RMatchRanges(MatchRangesInternal {
        inner: rmatches(haystack, pattern).0,
    })
}

pub fn find_range<H, P>(haystack: H, pattern: P) -> Option<Range<<H::Hay as Hay>::Index>>
where
    H: Haystack,
    P: Pattern<H>,
{
    pattern.into_searcher()
        .search(H::Span::from(haystack).borrow())
}

pub fn rfind_range<H, P>(haystack: H, pattern: P) -> Option<Range<<H::Hay as Hay>::Index>>
where
    H: Haystack,
    P: Pattern<H>,
    P::Searcher: ReverseSearcher,
{
    pattern.into_searcher()
        .rsearch(H::Span::from(haystack).borrow())
}

//------------------------------------------------------------------------------
// Split
//------------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct SplitInternal<H, S>
where
    H: Haystack,
{
    searcher: S,
    rest: H::Span,
    finished: bool,
    allow_trailing_empty: bool,
}

impl<H, S> SplitInternal<H, S>
where
    H: Haystack,
    S: Searcher<Hay = H::Hay>,
{
    #[inline]
    fn next(&mut self) -> Option<H> {
        if self.finished {
            return None;
        }

        let mut rest = self.rest.take();
        match self.searcher.search(rest.borrow()) {
            Some(subrange) => {
                let [left, _, right] = rest.split_around(subrange);
                self.rest = right;
                rest = left;
            }
            None => {
                self.finished = true;
                if !self.allow_trailing_empty && rest.is_empty() {
                    return None;
                }
            }
        }
        Some(Span::into(rest))
    }
}

impl<H, S> SplitInternal<H, S>
where
    H: Haystack,
    S: ReverseSearcher<Hay = H::Hay>,
{
    #[inline]
    fn next_back(&mut self) -> Option<H> {
        if self.finished {
            return None;
        }

        let rest = self.rest.take();
        let after = match self.searcher.rsearch(rest.borrow()) {
            Some(range) => {
                let [left, _, right] = rest.split_around(range);
                self.rest = left;
                right
            }
            None => {
                self.finished = true;
                rest
            }
        };

        if !self.allow_trailing_empty {
            self.allow_trailing_empty = true;
            if after.is_empty() {
                return self.next_back();
            }
        }

        Some(Span::into(after))
    }
}

generate_pattern_iterators! {
    forward:
        struct Split;
    reverse:
        struct RSplit;
    stability:
    internal:
        SplitInternal yielding (H);
    delegate double ended;
}

generate_pattern_iterators! {
    forward:
        struct SplitTerminator;
    reverse:
        struct RSplitTerminator;
    stability:
    internal:
        SplitInternal yielding (H);
    delegate double ended;
}

pub fn split<H, P>(haystack: H, pattern: P) -> Split<H, P::Searcher>
where
    H: Haystack,
    P: Pattern<H>,
{
    Split(SplitInternal {
        searcher: pattern.into_searcher(),
        rest: haystack.into(),
        finished: false,
        allow_trailing_empty: true,
    })
}

pub fn rsplit<H, P>(haystack: H, pattern: P) -> RSplit<H, P::Searcher>
where
    H: Haystack,
    P: Pattern<H>,
    P::Searcher: ReverseSearcher,
{
    RSplit(SplitInternal {
        searcher: pattern.into_searcher(),
        rest: haystack.into(),
        finished: false,
        allow_trailing_empty: true,
    })
}

pub fn split_terminator<H, P>(haystack: H, pattern: P) -> SplitTerminator<H, P::Searcher>
where
    H: Haystack,
    P: Pattern<H>,
{
    SplitTerminator(SplitInternal {
        searcher: pattern.into_searcher(),
        rest: haystack.into(),
        finished: false,
        allow_trailing_empty: false,
    })
}

pub fn rsplit_terminator<H, P>(haystack: H, pattern: P) -> RSplitTerminator<H, P::Searcher>
where
    H: Haystack,
    P: Pattern<H>,
    P::Searcher: ReverseSearcher,
{
    RSplitTerminator(SplitInternal {
        searcher: pattern.into_searcher(),
        rest: haystack.into(),
        finished: false,
        allow_trailing_empty: false,
    })
}

//------------------------------------------------------------------------------
// SplitN
//------------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct SplitNInternal<H, S>
where
    H: Haystack,
{
    searcher: S,
    rest: H::Span,
    n: usize,
}

impl<H, S> SplitNInternal<H, S>
where
    H: Haystack,
    S: Searcher<Hay = H::Hay>,
{
    #[inline]
    fn next(&mut self) -> Option<H> {
        let mut rest = self.rest.take();
        match self.n {
            0 => {
                return None;
            }
            1 => {
                self.n = 0;
            }
            n => {
                match self.searcher.search(rest.borrow()) {
                    Some(range) => {
                        let [left, _, right] = rest.split_around(range);
                        self.n = n - 1;
                        self.rest = right;
                        rest = left;
                    }
                    None => {
                        self.n = 0;
                    }
                }
            }
        }
        Some(Span::into(rest))
    }
}

impl<H, S> SplitNInternal<H, S>
where
    H: Haystack,
    S: ReverseSearcher<Hay = H::Hay>,
{
    #[inline]
    fn next_back(&mut self) -> Option<H> {
        let mut rest = self.rest.take();
        match self.n {
            0 => {
                return None;
            }
            1 => {
                self.n = 0;
            }
            n => {
                match self.searcher.rsearch(rest.borrow()) {
                    Some(range) => {
                        let [left, _, right] = rest.split_around(range);
                        self.n = n - 1;
                        self.rest = left;
                        rest = right;
                    }
                    None => {
                        self.n = 0;
                    }
                }
            }
        }
        Some(Span::into(rest))
    }
}

generate_pattern_iterators! {
    forward:
        struct SplitN;
    reverse:
        struct RSplitN;
    stability:
    internal:
        SplitNInternal yielding (H);
    delegate single ended;
}

pub fn splitn<H, P>(haystack: H, n: usize, pattern: P) -> SplitN<H, P::Searcher>
where
    H: Haystack,
    P: Pattern<H>,
{
    SplitN(SplitNInternal {
        searcher: pattern.into_searcher(),
        rest: haystack.into(),
        n,
    })
}

pub fn rsplitn<H, P>(haystack: H, n: usize, pattern: P) -> RSplitN<H, P::Searcher>
where
    H: Haystack,
    P: Pattern<H>,
    P::Searcher: ReverseSearcher,
{
    RSplitN(SplitNInternal {
        searcher: pattern.into_searcher(),
        rest: haystack.into(),
        n,
    })
}

//------------------------------------------------------------------------------
// Replace
//------------------------------------------------------------------------------

pub fn replace_with<H, P, F, W>(src: H, from: P, mut replacer: F, mut writer: W)
where
    H: Haystack,
    P: Pattern<H>,
    F: FnMut(H) -> H,
    W: FnMut(H),
{
    let mut searcher = from.into_searcher();
    let mut src = H::Span::from(src);
    while let Some(range) = searcher.search(src.borrow()) {
        let [left, middle, right] = src.split_around(range);
        writer(Span::into(left));
        writer(replacer(Span::into(middle)));
        src = right;
    }
    writer(Span::into(src));
}

pub fn replacen_with<H, P, F, W>(src: H, from: P, mut replacer: F, mut n: usize, mut writer: W)
where
    H: Haystack,
    P: Pattern<H>,
    F: FnMut(H) -> H,
    W: FnMut(H),
{
    let mut searcher = from.into_searcher();
    let mut src = H::Span::from(src);
    loop {
        if n == 0 {
            break;
        }
        n -= 1;
        if let Some(range) = searcher.search(src.borrow()) {
            let [left, middle, right] = src.split_around(range);
            writer(Span::into(left));
            writer(replacer(Span::into(middle)));
            src = right;
        } else {
            break;
        }
    }
    writer(Span::into(src));
}
