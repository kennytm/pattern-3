use haystack::{Hay, Haystack, IndexHaystack};
use pattern::{Pattern, Searcher, ReverseSearcher, DoubleEndedSearcher};
use std::iter::FusedIterator;
use std::ops::Range;
use std::mem;

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
            $internal_iterator:ident of $h:ident yielding ($iterty:ty);

        // Kind of delegation - either single ended or double ended
        delegate $($t:tt)*
    } => {
        $(#[$forward_iterator_attribute])*
        $(#[$common_stability_attribute])*
        #[derive(Debug, Clone)]
        pub struct $forward_iterator<H, S>($internal_iterator<H, S>) where H: $h;

        $(#[$common_stability_attribute])*
        impl<H, S> Iterator for $forward_iterator<H, S>
        where
            H: $h,
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
        #[derive(Debug, Clone)]
        pub struct $reverse_iterator<H, S>($internal_iterator<H, S>) where H: $h;

        $(#[$common_stability_attribute])*
        impl<H, S> Iterator for $reverse_iterator<H, S>
        where
            H: $h,
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
            H: $h,
            S: Searcher<Hay = H::Hay>,
        {}

        // #[stable(feature = "fused", since = "1.26.0")]
        impl<H, S> FusedIterator for $reverse_iterator<H, S>
        where
            H: $h,
            S: ReverseSearcher<Hay = H::Hay>,
        {}

        generate_pattern_iterators!($($t)* of $h with $(#[$common_stability_attribute])*,
                                                $forward_iterator,
                                                $reverse_iterator);
    };
    {
        double ended; of $h:ident with $(#[$common_stability_attribute:meta])*,
                           $forward_iterator:ident,
                           $reverse_iterator:ident
    } => {
        $(#[$common_stability_attribute])*
        impl<H, S> DoubleEndedIterator for $forward_iterator<H, S>
        where
            H: $h,
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
            H: $h,
            S: DoubleEndedSearcher<Hay = H::Hay>,
        {
            #[inline]
            fn next_back(&mut self) -> Option<Self::Item> {
                self.0.next()
            }
        }
    };
    {
        single ended; of $h:ident with $(#[$common_stability_attribute:meta])*,
                           $forward_iterator:ident,
                           $reverse_iterator:ident
    } => {}
}

//------------------------------------------------------------------------------
// Starts with / Ends with
//------------------------------------------------------------------------------

pub fn starts_with<H, P>(haystack: &H, pattern: P) -> bool
where
    H: Hay + ?Sized,
    P: Pattern<H>,
{
    pattern.is_prefix_of(haystack)
}

pub fn ends_with<H, P>(haystack: &H, pattern: P) -> bool
where
    H: Hay + ?Sized,
    P: Pattern<H>,
    P::Searcher: ReverseSearcher,
{
    pattern.is_suffix_of(haystack)
}

//------------------------------------------------------------------------------
// Trim
//------------------------------------------------------------------------------

pub fn trim_start<H, P>(haystack: H, mut pattern: P) -> H
where
    H: Haystack,
    P: Pattern<H::Hay>,
{
    unsafe {
        let start = pattern.trim_start(haystack.borrow());
        haystack.trim_start_unchecked(start)
    }
}

pub fn trim_end<H, P>(haystack: H, mut pattern: P) -> H
where
    H: Haystack,
    P: Pattern<H::Hay>,
    P::Searcher: ReverseSearcher,
{
    unsafe {
        let end = pattern.trim_end(haystack.borrow());
        haystack.trim_end_unchecked(end)
    }
}

pub fn trim<H, P>(haystack: H, mut pattern: P) -> H
where
    H: Haystack,
    P: Pattern<H::Hay>,
    P::Searcher: DoubleEndedSearcher,
{
    unsafe {
        let start = pattern.trim_start(haystack.borrow());
        let haystack = haystack.trim_start_unchecked(start);
        let end = pattern.trim_end(haystack.borrow());
        haystack.trim_end_unchecked(end)
    }
}

//------------------------------------------------------------------------------
// Matches
//------------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct MatchesInternal<H, S> {
    searcher: S,
    rest: H,
}

impl<H, S> MatchesInternal<H, S>
where
    H: Haystack,
    S: Searcher<Hay = H::Hay>,
{
    #[inline]
    fn next(&mut self) -> Option<H> {
        let rest = mem::replace(&mut self.rest, H::default());
        let range = self.searcher.search(rest.borrow())?;
        let (_, middle, right) = unsafe { rest.split_around_unchecked(range) };
        self.rest = right;
        Some(middle)
    }
}

impl<H, S> MatchesInternal<H, S>
where
    H: Haystack,
    S: ReverseSearcher<Hay = H::Hay>,
{
    #[inline]
    fn next_back(&mut self) -> Option<H> {
        let rest = mem::replace(&mut self.rest, H::default());
        let range = self.searcher.rsearch(rest.borrow())?;
        let (left, middle, _) = unsafe { rest.split_around_unchecked(range) };
        self.rest = left;
        Some(middle)
    }
}

generate_pattern_iterators! {
    forward:
        struct Matches;
    reverse:
        struct RMatches;
    stability:
    internal:
        MatchesInternal of Haystack yielding (H);
    delegate double ended;
}

pub fn matches<H, P>(haystack: H, pattern: P) -> Matches<H, P::Searcher>
where
    H: Haystack,
    P: Pattern<H::Hay>,
{
    Matches(MatchesInternal {
        searcher: pattern.into_searcher(),
        rest: haystack,
    })
}

pub fn rmatches<H, P>(haystack: H, pattern: P) -> RMatches<H, P::Searcher>
where
    H: Haystack,
    P: Pattern<H::Hay>,
    P::Searcher: ReverseSearcher,
{
    RMatches(MatchesInternal {
        searcher: pattern.into_searcher(),
        rest: haystack,
    })
}

pub fn contains<H, P>(hay: &H, pattern: P) -> bool
where
    H: Hay + ?Sized,
    P: Pattern<H>,
{
    pattern.into_searcher().search(hay).is_some()
}

//------------------------------------------------------------------------------
// MatchIndices
//------------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct MatchIndicesInternal<H, S>
where
    H: IndexHaystack,
{
    origin: H::Origin,
    inner: MatchesInternal<H, S>,
}

impl<H, S> MatchIndicesInternal<H, S>
where
    H: IndexHaystack,
    S: Searcher<Hay = H::Hay>,
{
    #[inline]
    fn next(&mut self) -> Option<(<H::Hay as Hay>::Index, H)> {
        let m = self.inner.next()?;
        let index = unsafe { m.range_from_origin(self.origin).start };
        Some((index, m))
    }
}

impl<H, S> MatchIndicesInternal<H, S>
where
    H: IndexHaystack,
    S: ReverseSearcher<Hay = H::Hay>,
{
    #[inline]
    fn next_back(&mut self) -> Option<(<H::Hay as Hay>::Index, H)> {
        let m = self.inner.next_back()?;
        let index = unsafe { m.range_from_origin(self.origin).start };
        Some((index, m))
    }
}

generate_pattern_iterators! {
    forward:
        struct MatchIndices;
    reverse:
        struct RMatchIndices;
    stability:
    internal:
        MatchIndicesInternal of IndexHaystack yielding ((<H::Hay as Hay>::Index, H));
    delegate double ended;
}

pub fn match_indices<H, P>(haystack: H, pattern: P) -> MatchIndices<H, P::Searcher>
where
    H: IndexHaystack,
    P: Pattern<H::Hay>,
{
    MatchIndices(MatchIndicesInternal {
        origin: haystack.origin(),
        inner: matches(haystack, pattern).0,
    })
}

pub fn rmatch_indices<H, P>(haystack: H, pattern: P) -> RMatchIndices<H, P::Searcher>
where
    H: IndexHaystack,
    P: Pattern<H::Hay>,
    P::Searcher: ReverseSearcher,
{
    RMatchIndices(MatchIndicesInternal {
        origin: haystack.origin(),
        inner: rmatches(haystack, pattern).0,
    })
}

pub fn find<H, P>(haystack: H, pattern: P) -> Option<<H::Hay as Hay>::Index>
where
    H: IndexHaystack,
    P: Pattern<H::Hay>,
{
    match_indices(haystack, pattern).next().map(|s| s.0)
}

pub fn rfind<H, P>(haystack: H, pattern: P) -> Option<<H::Hay as Hay>::Index>
where
    H: IndexHaystack,
    P: Pattern<H::Hay>,
    P::Searcher: ReverseSearcher,
{
    rmatch_indices(haystack, pattern).next().map(|s| s.0)
}

//------------------------------------------------------------------------------
// MatchRanges
//------------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct MatchRangesInternal<H, S>
where
    H: IndexHaystack,
{
    origin: H::Origin,
    inner: MatchesInternal<H, S>,
}

impl<H, S> MatchRangesInternal<H, S>
where
    H: IndexHaystack,
    S: Searcher<Hay = H::Hay>,
{
    #[inline]
    fn next(&mut self) -> Option<(Range<<H::Hay as Hay>::Index>, H)> {
        let m = self.inner.next()?;
        let range = unsafe { m.range_from_origin(self.origin) };
        Some((range, m))
    }
}

impl<H, S> MatchRangesInternal<H, S>
where
    H: IndexHaystack,
    S: ReverseSearcher<Hay = H::Hay>,
{
    #[inline]
    fn next_back(&mut self) -> Option<(Range<<H::Hay as Hay>::Index>, H)> {
        let m = self.inner.next_back()?;
        let range = unsafe { m.range_from_origin(self.origin) };
        Some((range, m))
    }
}

generate_pattern_iterators! {
    forward:
        struct MatchRanges;
    reverse:
        struct RMatchRanges;
    stability:
    internal:
        MatchRangesInternal of IndexHaystack yielding ((Range<<H::Hay as Hay>::Index>, H));
    delegate double ended;
}

pub fn match_ranges<H, P>(haystack: H, pattern: P) -> MatchRanges<H, P::Searcher>
where
    H: IndexHaystack,
    P: Pattern<H::Hay>,
{
    MatchRanges(MatchRangesInternal {
        origin: haystack.origin(),
        inner: matches(haystack, pattern).0,
    })
}

pub fn rmatch_ranges<H, P>(haystack: H, pattern: P) -> RMatchRanges<H, P::Searcher>
where
    H: IndexHaystack,
    P: Pattern<H::Hay>,
    P::Searcher: ReverseSearcher,
{
    RMatchRanges(MatchRangesInternal {
        origin: haystack.origin(),
        inner: rmatches(haystack, pattern).0,
    })
}

pub fn find_range<H, P>(haystack: H, pattern: P) -> Option<Range<<H::Hay as Hay>::Index>>
where
    H: IndexHaystack,
    P: Pattern<H::Hay>,
{
    match_ranges(haystack, pattern).next().map(|s| s.0)
}

pub fn rfind_range<H, P>(haystack: H, pattern: P) -> Option<Range<<H::Hay as Hay>::Index>>
where
    H: IndexHaystack,
    P: Pattern<H::Hay>,
    P::Searcher: ReverseSearcher,
{
    rmatch_ranges(haystack, pattern).next().map(|s| s.0)
}

//------------------------------------------------------------------------------
// Split
//------------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct SplitInternal<H, S> {
    searcher: S,
    rest: H,
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

        let rest = mem::replace(&mut self.rest, H::default());
        match self.searcher.search(rest.borrow()) {
            Some(range) => {
                let (left, _, right) = unsafe { rest.split_around_unchecked(range) };
                self.rest = right;
                Some(left)
            }
            None => {
                self.finished = true;
                if !self.allow_trailing_empty && rest.borrow().is_empty() {
                    None
                } else {
                    Some(rest)
                }
            }
        }
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

        let rest = mem::replace(&mut self.rest, H::default());
        let after = match self.searcher.rsearch(rest.borrow()) {
            Some(range) => {
                let (left, _, right) = unsafe { rest.split_around_unchecked(range) };
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
            if after.borrow().is_empty() {
                return self.next_back();
            }
        }

        Some(after)
    }
}

generate_pattern_iterators! {
    forward:
        struct Split;
    reverse:
        struct RSplit;
    stability:
    internal:
        SplitInternal of Haystack yielding (H);
    delegate double ended;
}

generate_pattern_iterators! {
    forward:
        struct SplitTerminator;
    reverse:
        struct RSplitTerminator;
    stability:
    internal:
        SplitInternal of Haystack yielding (H);
    delegate double ended;
}

pub fn split<H, P>(haystack: H, pattern: P) -> Split<H, P::Searcher>
where
    H: Haystack,
    P: Pattern<H::Hay>,
{
    Split(SplitInternal {
        searcher: pattern.into_searcher(),
        rest: haystack,
        finished: false,
        allow_trailing_empty: true,
    })
}

pub fn rsplit<H, P>(haystack: H, pattern: P) -> RSplit<H, P::Searcher>
where
    H: Haystack,
    P: Pattern<H::Hay>,
    P::Searcher: ReverseSearcher,
{
    RSplit(SplitInternal {
        searcher: pattern.into_searcher(),
        rest: haystack,
        finished: false,
        allow_trailing_empty: true,
    })
}

pub fn split_terminator<H, P>(haystack: H, pattern: P) -> SplitTerminator<H, P::Searcher>
where
    H: Haystack,
    P: Pattern<H::Hay>,
{
    SplitTerminator(SplitInternal {
        searcher: pattern.into_searcher(),
        rest: haystack,
        finished: false,
        allow_trailing_empty: false,
    })
}

pub fn rsplit_terminator<H, P>(haystack: H, pattern: P) -> RSplitTerminator<H, P::Searcher>
where
    H: Haystack,
    P: Pattern<H::Hay>,
    P::Searcher: ReverseSearcher,
{
    RSplitTerminator(SplitInternal {
        searcher: pattern.into_searcher(),
        rest: haystack,
        finished: false,
        allow_trailing_empty: false,
    })
}

//------------------------------------------------------------------------------
// SplitN
//------------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct SplitNInternal<H, S> {
    searcher: S,
    rest: H,
    n: usize,
}

impl<H, S> SplitNInternal<H, S>
where
    H: Haystack,
    S: Searcher<Hay = H::Hay>,
{
    #[inline]
    fn next(&mut self) -> Option<H> {
        let rest = mem::replace(&mut self.rest, H::default());
        match self.n {
            0 => {
                None
            }
            1 => {
                self.n = 0;
                Some(rest)
            }
            n => {
                match self.searcher.search(rest.borrow()) {
                    Some(range) => {
                        let (left, _, right) = unsafe { rest.split_around_unchecked(range) };
                        self.n = n - 1;
                        self.rest = right;
                        Some(left)
                    }
                    None => {
                        self.n = 0;
                        Some(rest)
                    }
                }
            }
        }
    }
}

impl<H, S> SplitNInternal<H, S>
where
    H: Haystack,
    S: ReverseSearcher<Hay = H::Hay>,
{
    #[inline]
    fn next_back(&mut self) -> Option<H> {
        let rest = mem::replace(&mut self.rest, H::default());
        match self.n {
            0 => {
                None
            }
            1 => {
                self.n = 0;
                Some(rest)
            }
            n => {
                match self.searcher.rsearch(rest.borrow()) {
                    Some(range) => {
                        let (left, _, right) = unsafe { rest.split_around_unchecked(range) };
                        self.n = n - 1;
                        self.rest = left;
                        Some(right)
                    }
                    None => {
                        self.n = 0;
                        Some(rest)
                    }
                }
            }
        }
    }
}

generate_pattern_iterators! {
    forward:
        struct SplitN;
    reverse:
        struct RSplitN;
    stability:
    internal:
        SplitNInternal of Haystack yielding (H);
    delegate single ended;
}

pub fn splitn<H, P>(haystack: H, n: usize, pattern: P) -> SplitN<H, P::Searcher>
where
    H: Haystack,
    P: Pattern<H::Hay>,
{
    SplitN(SplitNInternal {
        searcher: pattern.into_searcher(),
        rest: haystack,
        n,
    })
}

pub fn rsplitn<H, P>(haystack: H, n: usize, pattern: P) -> RSplitN<H, P::Searcher>
where
    H: Haystack,
    P: Pattern<H::Hay>,
    P::Searcher: ReverseSearcher,
{
    RSplitN(SplitNInternal {
        searcher: pattern.into_searcher(),
        rest: haystack,
        n,
    })
}

//------------------------------------------------------------------------------
// Replace
//------------------------------------------------------------------------------

pub fn replace_with<H, P, F, W>(mut src: H, from: P, mut replacer: F, mut writer: W)
where
    H: Haystack,
    P: Pattern<H::Hay>,
    F: FnMut(H) -> H,
    W: FnMut(H),
{
    let mut searcher = from.into_searcher();
    while let Some(range) = searcher.search(src.borrow()) {
        let (left, middle, right) = unsafe { src.split_around_unchecked(range) };
        writer(left);
        writer(replacer(middle));
        src = right;
    }
    writer(src);
}

pub fn replacen_with<H, P, F, W>(mut src: H, from: P, mut replacer: F, mut n: usize, mut writer: W)
where
    H: Haystack,
    P: Pattern<H::Hay>,
    F: FnMut(H) -> H,
    W: FnMut(H),
{
    let mut searcher = from.into_searcher();
    loop {
        if n == 0 {
            break;
        }
        n -= 1;
        if let Some(range) = searcher.search(src.borrow()) {
            let (left, middle, right) = unsafe { src.split_around_unchecked(range) };
            writer(left);
            writer(replacer(middle));
            src = right;
        } else {
            break;
        }
    }
    writer(src);
}
