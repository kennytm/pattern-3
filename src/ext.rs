use haystack::{Haystack, IndexHaystack};
use pattern::{Pattern, Searcher, ReverseSearcher, DoubleEndedSearcher};
use std::iter::FusedIterator;
use std::ops::Range;
use std::fmt;


/// This macro generates a Clone impl for string pattern API
/// wrapper types of the form X<H, P>
macro_rules! derive_pattern_clone {
    (clone $t:ident of $h:ident with |$s:ident| $e:expr) => {
        impl<H, P> Clone for $t<H, P>
        where
            H: $h + Clone,
            P: Pattern<H>,
            P::Searcher: Clone,
        {
            fn clone(&self) -> Self {
                let $s = self;
                $e
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
            $internal_iterator:ident of $h:ident yielding ($iterty:ty);

        // Kind of delegation - either single ended or double ended
        delegate $($t:tt)*
    } => {
        $(#[$forward_iterator_attribute])*
        $(#[$common_stability_attribute])*
        pub struct $forward_iterator<H, P>($internal_iterator<H, P>)
        where
            H: $h,
            P: Pattern<H>;

        $(#[$common_stability_attribute])*
        impl<H, P> fmt::Debug for $forward_iterator<H, P>
        where
            H: $h + fmt::Debug,
            P: Pattern<H>,
            P::Searcher: fmt::Debug,
        {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.debug_tuple(stringify!($forward_iterator))
                    .field(&self.0)
                    .finish()
            }
        }

        $(#[$common_stability_attribute])*
        impl<H, P> Iterator for $forward_iterator<H, P>
        where
            H: $h,
            P: Pattern<H>,
        {
            type Item = $iterty;

            #[inline]
            fn next(&mut self) -> Option<$iterty> {
                self.0.next()
            }
        }

        $(#[$common_stability_attribute])*
        impl<H, P> Clone for $forward_iterator<H, P>
        where
            H: $h + Clone,
            P: Pattern<H>,
            P::Searcher: Clone,
        {
            fn clone(&self) -> Self {
                $forward_iterator(self.0.clone())
            }
        }

        $(#[$reverse_iterator_attribute])*
        $(#[$common_stability_attribute])*
        pub struct $reverse_iterator<H, P>($internal_iterator<H, P>)
        where
            H: $h,
            P: Pattern<H>;

        $(#[$common_stability_attribute])*
        impl<H, P> fmt::Debug for $reverse_iterator<H, P>
        where
            H: $h + fmt::Debug,
            P: Pattern<H>,
            P::Searcher: fmt::Debug,
        {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.debug_tuple(stringify!($reverse_iterator))
                    .field(&self.0)
                    .finish()
            }
        }

        $(#[$common_stability_attribute])*
        impl<H, P> Iterator for $reverse_iterator<H, P>
        where
            H: $h,
            P: Pattern<H>,
            P::Searcher: ReverseSearcher<H>,
        {
            type Item = $iterty;

            #[inline]
            fn next(&mut self) -> Option<$iterty> {
                self.0.next_back()
            }
        }

        $(#[$common_stability_attribute])*
        impl<H, P> Clone for $reverse_iterator<H, P>
        where
            H: $h + Clone,
            P: Pattern<H>,
            P::Searcher: Clone,
        {
            fn clone(&self) -> Self {
                $reverse_iterator(self.0.clone())
            }
        }

        // #[stable(feature = "fused", since = "1.26.0")]
        impl<H, P> FusedIterator for $forward_iterator<H, P>
        where
            H: $h,
            P: Pattern<H>,
        {}

        // #[stable(feature = "fused", since = "1.26.0")]
        impl<H, P> FusedIterator for $reverse_iterator<H, P>
        where
            H: $h,
            P: Pattern<H>,
            P::Searcher: ReverseSearcher<H>,
        {}

        generate_pattern_iterators!($($t)* of $h with $(#[$common_stability_attribute])*,
                                                $forward_iterator,
                                                $reverse_iterator, $iterty);
    };
    {
        double ended; of $h:ident with $(#[$common_stability_attribute:meta])*,
                           $forward_iterator:ident,
                           $reverse_iterator:ident, $iterty:ty
    } => {
        $(#[$common_stability_attribute])*
        impl<H, P> DoubleEndedIterator for $forward_iterator<H, P>
        where
            H: $h,
            P: Pattern<H>,
            P::Searcher: DoubleEndedSearcher<H>,
        {
            #[inline]
            fn next_back(&mut self) -> Option<$iterty> {
                self.0.next_back()
            }
        }

        $(#[$common_stability_attribute])*
        impl<H, P> DoubleEndedIterator for $reverse_iterator<H, P>
        where
            H: $h,
            P: Pattern<H>,
            P::Searcher: DoubleEndedSearcher<H>,
        {
            #[inline]
            fn next_back(&mut self) -> Option<$iterty> {
                self.0.next()
            }
        }
    };
    {
        single ended; of $h:ident with $(#[$common_stability_attribute:meta])*,
                           $forward_iterator:ident,
                           $reverse_iterator:ident, $iterty:ty
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
    pattern.is_prefix_of(haystack)
}

pub fn ends_with<H, P>(haystack: H, pattern: P) -> bool
where
    H: Haystack,
    P: Pattern<H>,
    P::Searcher: ReverseSearcher<H>
{
    pattern.is_suffix_of(haystack)
}

//------------------------------------------------------------------------------
// Trim
//------------------------------------------------------------------------------

pub fn trim_start<H, P>(mut haystack: H, mut pattern: P) -> H
where
    H: Haystack,
    P: Pattern<H>,
{
    pattern.trim_start(&mut haystack);
    haystack
}

pub fn trim_end<H, P>(mut haystack: H, mut pattern: P) -> H
where
    H: Haystack,
    P: Pattern<H>,
    P::Searcher: ReverseSearcher<H>,
{
    pattern.trim_end(&mut haystack);
    haystack
}

pub fn trim<H, P>(mut haystack: H, mut pattern: P) -> H
where
    H: Haystack,
    P: Pattern<H>,
    P::Searcher: DoubleEndedSearcher<H>,
{
    pattern.trim_start(&mut haystack);
    pattern.trim_end(&mut haystack);
    haystack
}

//------------------------------------------------------------------------------
// Matches
//------------------------------------------------------------------------------

struct MatchesInternal<H, P>
where
    H: Haystack,
    P: Pattern<H>,
{
    searcher: P::Searcher,
    rest: H,
}

impl<H, P> fmt::Debug for MatchesInternal<H, P>
where
    H: Haystack + fmt::Debug,
    P: Pattern<H>,
    P::Searcher: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MatchesInternal")
            .field("searcher", &self.searcher)
            .field("rest", &self.rest)
            .finish()
    }
}

impl<H, P> MatchesInternal<H, P>
where
    H: Haystack,
    P: Pattern<H>,
{
    #[inline]
    fn next(&mut self) -> Option<H> {
        self.searcher.search(&mut self.rest).1
    }
}

impl<H, P> MatchesInternal<H, P>
where
    H: Haystack,
    P: Pattern<H>,
    P::Searcher: ReverseSearcher<H>,
{
    #[inline]
    fn next_back(&mut self) -> Option<H> {
        self.searcher.rsearch(&mut self.rest).0
    }
}

derive_pattern_clone! {
    clone MatchesInternal of Haystack
    with |s| MatchesInternal {
        searcher: s.searcher.clone(),
        rest: s.rest.clone(),
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

pub fn matches<H, P>(haystack: H, pattern: P) -> Matches<H, P>
where
    H: Haystack,
    P: Pattern<H>,
{
    Matches(MatchesInternal {
        searcher: pattern.into_searcher(),
        rest: haystack,
    })
}

pub fn rmatches<H, P>(haystack: H, pattern: P) -> RMatches<H, P>
where
    H: Haystack,
    P: Pattern<H>,
    P::Searcher: ReverseSearcher<H>,
{
    RMatches(MatchesInternal {
        searcher: pattern.into_searcher(),
        rest: haystack,
    })
}

pub fn contains<H, P>(haystack: H, pattern: P) -> bool
where
    H: Haystack,
    P: Pattern<H>,
{
    matches(haystack, pattern).next().is_some()
}

//------------------------------------------------------------------------------
// MatchIndices
//------------------------------------------------------------------------------

struct MatchIndicesInternal<H, P>
where
    H: IndexHaystack,
    P: Pattern<H>,
{
    origin: H::Origin,
    inner: MatchesInternal<H, P>,
}

impl<H, P> fmt::Debug for MatchIndicesInternal<H, P>
where
    H: IndexHaystack + fmt::Debug,
    P: Pattern<H>,
    P::Searcher: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MatchIndicesInternal")
            .field("origin", &self.origin)
            .field("inner", &self.inner)
            .finish()
    }
}

impl<H, P> MatchIndicesInternal<H, P>
where
    H: IndexHaystack,
    P: Pattern<H>,
{
    #[inline]
    fn next(&mut self) -> Option<(H::Index, H)> {
        let m = self.inner.next()?;
        let index = unsafe { m.range_from_origin(self.origin).start };
        Some((index, m))
    }
}

impl<H, P> MatchIndicesInternal<H, P>
where
    H: IndexHaystack,
    P: Pattern<H>,
    P::Searcher: ReverseSearcher<H>,
{
    #[inline]
    fn next_back(&mut self) -> Option<(H::Index, H)> {
        let m = self.inner.next_back()?;
        let index = unsafe { m.range_from_origin(self.origin).start };
        Some((index, m))
    }
}

derive_pattern_clone! {
    clone MatchIndicesInternal of IndexHaystack
    with |s| MatchIndicesInternal {
        origin: s.origin,
        inner: s.inner.clone(),
    }
}

generate_pattern_iterators! {
    forward:
        struct MatchIndices;
    reverse:
        struct RMatchIndices;
    stability:
    internal:
        MatchIndicesInternal of IndexHaystack yielding ((H::Index, H));
    delegate double ended;
}

pub fn match_indices<H, P>(haystack: H, pattern: P) -> MatchIndices<H, P>
where
    H: IndexHaystack,
    P: Pattern<H>,
{
    MatchIndices(MatchIndicesInternal {
        origin: haystack.origin(),
        inner: matches(haystack, pattern).0,
    })
}

pub fn rmatch_indices<H, P>(haystack: H, pattern: P) -> RMatchIndices<H, P>
where
    H: IndexHaystack,
    P: Pattern<H>,
    P::Searcher: ReverseSearcher<H>,
{
    RMatchIndices(MatchIndicesInternal {
        origin: haystack.origin(),
        inner: rmatches(haystack, pattern).0,
    })
}

pub fn find<H, P>(haystack: H, pattern: P) -> Option<H::Index>
where
    H: IndexHaystack,
    P: Pattern<H>,
{
    match_indices(haystack, pattern).next().map(|s| s.0)
}

pub fn rfind<H, P>(haystack: H, pattern: P) -> Option<H::Index>
where
    H: IndexHaystack,
    P: Pattern<H>,
    P::Searcher: ReverseSearcher<H>,
{
    rmatch_indices(haystack, pattern).next().map(|s| s.0)
}

//------------------------------------------------------------------------------
// MatchRanges
//------------------------------------------------------------------------------

struct MatchRangesInternal<H, P>
where
    H: IndexHaystack,
    P: Pattern<H>,
{
    origin: H::Origin,
    inner: MatchesInternal<H, P>,
}

impl<H, P> fmt::Debug for MatchRangesInternal<H, P>
where
    H: IndexHaystack + fmt::Debug,
    P: Pattern<H>,
    P::Searcher: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MatchRangesInternal")
            .field("origin", &self.origin)
            .field("inner", &self.inner)
            .finish()
    }
}

impl<H, P> MatchRangesInternal<H, P>
where
    H: IndexHaystack,
    P: Pattern<H>,
{
    #[inline]
    fn next(&mut self) -> Option<(Range<H::Index>, H)> {
        let m = self.inner.next()?;
        let range = unsafe { m.range_from_origin(self.origin) };
        Some((range, m))
    }
}

impl<H, P> MatchRangesInternal<H, P>
where
    H: IndexHaystack,
    P: Pattern<H>,
    P::Searcher: ReverseSearcher<H>,
{
    #[inline]
    fn next_back(&mut self) -> Option<(Range<H::Index>, H)> {
        let m = self.inner.next_back()?;
        let range = unsafe { m.range_from_origin(self.origin) };
        Some((range, m))
    }
}

derive_pattern_clone! {
    clone MatchRangesInternal of IndexHaystack
    with |s| MatchRangesInternal {
        origin: s.origin,
        inner: s.inner.clone(),
    }
}

generate_pattern_iterators! {
    forward:
        struct MatchRanges;
    reverse:
        struct RMatchRanges;
    stability:
    internal:
        MatchRangesInternal of IndexHaystack yielding ((Range<H::Index>, H));
    delegate double ended;
}

pub fn match_ranges<H, P>(haystack: H, pattern: P) -> MatchRanges<H, P>
where
    H: IndexHaystack,
    P: Pattern<H>,
{
    MatchRanges(MatchRangesInternal {
        origin: haystack.origin(),
        inner: matches(haystack, pattern).0,
    })
}

pub fn rmatch_ranges<H, P>(haystack: H, pattern: P) -> RMatchRanges<H, P>
where
    H: IndexHaystack,
    P: Pattern<H>,
    P::Searcher: ReverseSearcher<H>,
{
    RMatchRanges(MatchRangesInternal {
        origin: haystack.origin(),
        inner: rmatches(haystack, pattern).0,
    })
}

pub fn find_range<H, P>(haystack: H, pattern: P) -> Option<Range<H::Index>>
where
    H: IndexHaystack,
    P: Pattern<H>,
{
    match_ranges(haystack, pattern).next().map(|s| s.0)
}

pub fn rfind_range<H, P>(haystack: H, pattern: P) -> Option<Range<H::Index>>
where
    H: IndexHaystack,
    P: Pattern<H>,
    P::Searcher: ReverseSearcher<H>,
{
    rmatch_ranges(haystack, pattern).next().map(|s| s.0)
}

//------------------------------------------------------------------------------
// Split
//------------------------------------------------------------------------------

struct SplitInternal<H, P>
where
    H: Haystack,
    P: Pattern<H>,
{
    searcher: P::Searcher,
    rest: H,
    finished: bool,
    allow_trailing_empty: bool,
}

impl<H, P> fmt::Debug for SplitInternal<H, P>
where
    H: Haystack + fmt::Debug,
    P: Pattern<H>,
    P::Searcher: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SplitNInternal")
            .field("searcher", &self.searcher)
            .field("rest", &self.rest)
            .field("finished", &self.finished)
            .finish()
    }
}

impl<H, P> SplitInternal<H, P>
where
    H: Haystack,
    P: Pattern<H>,
{
    #[inline]
    fn next(&mut self) -> Option<H> {
        if self.finished {
            return None;
        }
        let (before, found) = self.searcher.search(&mut self.rest);
        if found.is_none() {
            self.finished = true;
            if !self.allow_trailing_empty && before.is_empty() {
                return None;
            }
        }
        Some(before)
    }
}

impl<H, P> SplitInternal<H, P>
where
    H: Haystack,
    P: Pattern<H>,
    P::Searcher: ReverseSearcher<H>,
{
    #[inline]
    fn next_back(&mut self) -> Option<H> {
        if self.finished {
            return None;
        }
        let (found, after) = self.searcher.rsearch(&mut self.rest);
        if found.is_none() {
            self.finished = true;
        }
        if !self.allow_trailing_empty {
            self.allow_trailing_empty = true;
            if after.is_empty() {
                return self.next_back();
            }
        }
        Some(after)
    }
}

derive_pattern_clone! {
    clone SplitInternal of Haystack
    with |s| SplitInternal {
        searcher: s.searcher.clone(),
        rest: s.rest.clone(),
        finished: s.finished,
        allow_trailing_empty: s.allow_trailing_empty,
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

pub fn split<H, P>(haystack: H, pattern: P) -> Split<H, P>
where
    H: Haystack,
    P: Pattern<H>,
{
    Split(SplitInternal {
        searcher: pattern.into_searcher(),
        rest: haystack,
        finished: false,
        allow_trailing_empty: true,
    })
}

pub fn rsplit<H, P>(haystack: H, pattern: P) -> RSplit<H, P>
where
    H: Haystack,
    P: Pattern<H>,
    P::Searcher: ReverseSearcher<H>,
{
    RSplit(SplitInternal {
        searcher: pattern.into_searcher(),
        rest: haystack,
        finished: false,
        allow_trailing_empty: true,
    })
}

pub fn split_terminator<H, P>(haystack: H, pattern: P) -> SplitTerminator<H, P>
where
    H: Haystack,
    P: Pattern<H>,
{
    SplitTerminator(SplitInternal {
        searcher: pattern.into_searcher(),
        rest: haystack,
        finished: false,
        allow_trailing_empty: false,
    })
}

pub fn rsplit_terminator<H, P>(haystack: H, pattern: P) -> RSplitTerminator<H, P>
where
    H: Haystack,
    P: Pattern<H>,
    P::Searcher: ReverseSearcher<H>,
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

struct SplitNInternal<H, P>
where
    H: Haystack,
    P: Pattern<H>,
{
    searcher: P::Searcher,
    rest: H,
    n: usize,
}

impl<H, P> fmt::Debug for SplitNInternal<H, P>
where
    H: Haystack + fmt::Debug,
    P: Pattern<H>,
    P::Searcher: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SplitNInternal")
            .field("searcher", &self.searcher)
            .field("rest", &self.rest)
            .field("n", &self.n)
            .finish()
    }
}

impl<H, P> SplitNInternal<H, P>
where
    H: Haystack,
    P: Pattern<H>,
{
    #[inline]
    fn next(&mut self) -> Option<H> {
        match self.n {
            0 => {
                None
            }
            1 => {
                self.n = 0;
                Some(self.rest.collapse_to_end())
            }
            n => {
                let (before, found) = self.searcher.search(&mut self.rest);
                if found.is_none() {
                    self.n = 0;
                } else {
                    self.n = n - 1;
                }
                Some(before)
            }
        }
    }
}

impl<H, P> SplitNInternal<H, P>
where
    H: Haystack,
    P: Pattern<H>,
    P::Searcher: ReverseSearcher<H>,
{
    #[inline]
    fn next_back(&mut self) -> Option<H> {
        match self.n {
            0 => {
                None
            }
            1 => {
                self.n = 0;
                Some(self.rest.collapse_to_start())
            }
            n => {
                let (found, after) = self.searcher.rsearch(&mut self.rest);
                if found.is_none() {
                    self.n = 0;
                } else {
                    self.n = n - 1;
                }
                Some(after)
            }
        }
    }
}

derive_pattern_clone! {
    clone SplitNInternal of Haystack
    with |s| SplitNInternal {
        searcher: s.searcher.clone(),
        rest: s.rest.clone(),
        n: s.n,
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

pub fn splitn<H, P>(haystack: H, n: usize, pattern: P) -> SplitN<H, P>
where
    H: Haystack,
    P: Pattern<H>,
{
    SplitN(SplitNInternal {
        searcher: pattern.into_searcher(),
        rest: haystack,
        n,
    })
}

pub fn rsplitn<H, P>(haystack: H, n: usize, pattern: P) -> RSplitN<H, P>
where
    H: Haystack,
    P: Pattern<H>,
    P::Searcher: ReverseSearcher<H>,
{
    RSplitN(SplitNInternal {
        searcher: pattern.into_searcher(),
        rest: haystack,
        n,
    })
}



//------------------------------------------------------------------------------
// Split
//------------------------------------------------------------------------------

/*

struct SplitInternal<H, P>
where
    H: Haystack,
    P: Pattern<H>,
{
    searcher: P::Searcher,
    finished: bool,
    allow_trailing_empty: bool,
}

impl<H, P> fmt::Debug for SplitInternal<H, P>
where
    H: Haystack,
    P: Pattern<H>,
    P::Searcher: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SplitInternal")
            .field("searcher", &self.searcher)
            .field("finished", &self.finished)
            .finish()
    }
}

impl<H, P> SplitInternal<H, P>
where
    H: Haystack,
    P: Pattern<H>,
{
    #[inline]
    fn next(&mut self) -> Option<H> {
        if self.finished {
            return None;
        }
        let (before, found) = self.searcher.search();
        if found.is_none() {
            self.finished = true;
            if !self.allow_trailing_empty && before.is_empty() {
                return None;
            }
        }
        Some(before)
    }

    #[inline]
    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        F: FnMut(B, H) -> R,
        R: Try<Ok = B>,
    {
        let finished = &mut self.finished;
        if *finished {
            return Try::from_ok(init);
        }
        let allow_trailing_empty = self.allow_trailing_empty;
        self.searcher.try_fold(init, move |prev, before, found| {
            if found.is_none() {
                *finished = true;
                if !allow_trailing_empty && before.is_empty() {
                    return Try::from_ok(prev);
                }
            }
            f(prev, before)
        })
    }
}

impl<H, P> SplitInternal<H, P>
where
    H: Haystack,
    P: Pattern<H>,
    P::Searcher: ReverseSearcher<H>,
{
    #[inline]
    fn next_back(&mut self) -> Option<H> {
        if self.finished {
            return None;
        }
        let (mut found, mut after) = self.searcher.rsearch();
        if !self.allow_trailing_empty && after.is_empty() {
            self.allow_trailing_empty = true;
            let (new_found, new_after) = self.searcher.rsearch();
            found = new_found;
            after = new_after;
        }
        self.finished = found.is_none();
        Some(after)
    }

    #[inline]
    fn try_rfold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        F: FnMut(B, H) -> R,
        R: Try<Ok = B>,
    {
        let finished = &mut self.finished;
        if *finished {
            return Try::from_ok(init);
        }
        let allow_trailing_empty = &mut self.allow_trailing_empty;
        self.searcher.try_rfold(init, move |prev, found, after| {
            if !*allow_trailing_empty && after.is_empty() {
                *allow_trailing_empty = true;
                return Try::from_ok(prev);
            }
            if found.is_none() {
                *finished = true;
            }
            f(prev, after)
        })
    }
}

derive_pattern_clone! {
    clone SplitInternal of Haystack
    with |s| SplitInternal {
        searcher: s.searcher.clone(),
        finished: s.finished,
        allow_trailing_empty: s.allow_trailing_empty,
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



//------------------------------------------------------------------------------
// Find / Find range
//------------------------------------------------------------------------------




//------------------------------------------------------------------------------
// Matches
//------------------------------------------------------------------------------

//------------------------------------------------------------------------------
// Splits
//------------------------------------------------------------------------------

pub fn split<H, P>(haystack: H, pattern: P) -> Split<H, P>
where
    H: Haystack,
    P: Pattern<H>,
{
    Split(SplitInternal {
        searcher: pattern.into_searcher(haystack),
        finished: false,
        allow_trailing_empty: true,
    })
}

pub fn rsplit<H, P>(haystack: H, pattern: P) -> RSplit<H, P>
where
    H: Haystack,
    P: Pattern<H>,
    P::Searcher: ReverseSearcher<H>,
{
    RSplit(SplitInternal {
        searcher: pattern.into_searcher(haystack),
        finished: false,
        allow_trailing_empty: true,
    })
}

pub fn split_terminator<H, P>(haystack: H, pattern: P) -> SplitTerminator<H, P>
where
    H: Haystack,
    P: Pattern<H>,
{
    SplitTerminator(SplitInternal {
        searcher: pattern.into_searcher(haystack),
        finished: false,
        allow_trailing_empty: false,
    })
}

pub fn rsplit_terminator<H, P>(haystack: H, pattern: P) -> RSplitTerminator<H, P>
where
    H: Haystack,
    P: Pattern<H>,
    P::Searcher: ReverseSearcher<H>,
{
    RSplitTerminator(SplitInternal {
        searcher: pattern.into_searcher(haystack),
        finished: false,
        allow_trailing_empty: false,
    })
}

*/