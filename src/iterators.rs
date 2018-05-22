// Copyright 2018 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use super::{Pattern, Haystack, Searcher, ReverseSearcher, DoubleEndedSearcher};
use std::fmt;
use std::ops::Range;
use std::iter::FusedIterator;

//------------------------------------------------------------------------------
// Macro definitions
//------------------------------------------------------------------------------

/// Iterators involved in pattern operations

/// This macro generates a Clone impl for generic pattern API
/// wrapper types of the form X<H, P>
macro_rules! derive_pattern_clone {
    (clone $t:ident with |$s:ident| $e:expr) => {
        impl<H: Haystack, P: Pattern<H>> Clone for $t<H, P>
            where P::Searcher: Clone
        {
            fn clone(&self) -> Self {
                let $s = self;
                $e
            }
        }
    }
}


/// This macro generates two public iterator structs
/// wrapping a private internal one that makes use of the `Pattern` API.
///
/// For all patterns `P: Pattern<H>` the following items will be
/// generated (generics omitted):
///
/// ```text
/// struct $forward_iterator($internal_iterator);
/// struct $reverse_iterator($internal_iterator);
///
/// impl Iterator for $forward_iterator
/// { /* internal ends up calling Searcher::next_match() */ }
///
/// impl DoubleEndedIterator for $forward_iterator
///       where P::Searcher: DoubleEndedSearcher
/// { /* internal ends up calling Searcher::next_match_back() */ }
///
/// impl Iterator for $reverse_iterator
///       where P::Searcher: ReverseSearcher
/// { /* internal ends up calling Searcher::next_match_back() */ }
///
/// impl DoubleEndedIterator for $reverse_iterator
///       where P::Searcher: DoubleEndedSearcher
/// { /* internal ends up calling Searcher::next_match() */ }
/// ```
///
/// The internal one is defined outside the macro, and has almost the same
/// semantic as a DoubleEndedIterator by delegating to `pattern::Searcher` and
/// `pattern::ReverseSearcher` for both forward and reverse iteration.
///
/// "Almost", because a `Searcher` and a `ReverseSearcher` for a given
/// `Pattern` might not return the same elements, so actually implementing
/// `DoubleEndedIterator` for it would be incorrect.
/// (See the docs in `str::pattern` for more details)
///
/// However, the internal struct still represents a single ended iterator from
/// either end, and depending on pattern is also a valid double ended iterator,
/// so the two wrapper structs implement `Iterator`
/// and `DoubleEndedIterator` depending on the concrete pattern type, leading
/// to the complex impls seen above.
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
        pub struct $forward_iterator<H: Haystack, P: Pattern<H>>(
            pub(super) $internal_iterator<H, P>,
        );

        $(#[$common_stability_attribute])*
        impl<H: Haystack, P: Pattern<H>> fmt::Debug for $forward_iterator<H, P>
        where
            P::Searcher: fmt::Debug,
            H::StartCursor: fmt::Debug,
            H::EndCursor: fmt::Debug,
        {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.debug_tuple(stringify!($forward_iterator))
                    .field(&self.0)
                    .finish()
            }
        }

        $(#[$common_stability_attribute])*
        impl<H: Haystack, P: Pattern<H>> Iterator for $forward_iterator<H, P> {
            type Item = $iterty;

            #[inline]
            fn next(&mut self) -> Option<$iterty> {
                self.0.next()
            }
        }

        $(#[$common_stability_attribute])*
        impl<H: Haystack, P: Pattern<H>> Clone for $forward_iterator<H, P>
            where P::Searcher: Clone
        {
            fn clone(&self) -> Self {
                $forward_iterator(self.0.clone())
            }
        }

        $(#[$reverse_iterator_attribute])*
        $(#[$common_stability_attribute])*
        pub struct $reverse_iterator<H: Haystack, P: Pattern<H>>(
            pub(super) $internal_iterator<H, P>,
        );

        $(#[$common_stability_attribute])*
        impl<H: Haystack, P: Pattern<H>> fmt::Debug for $reverse_iterator<H, P>
        where
            P::Searcher: fmt::Debug,
            H::StartCursor: fmt::Debug,
            H::EndCursor: fmt::Debug,
        {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.debug_tuple(stringify!($reverse_iterator))
                    .field(&self.0)
                    .finish()
            }
        }

        $(#[$common_stability_attribute])*
        impl<H: Haystack, P: Pattern<H>> Iterator for $reverse_iterator<H, P>
            where P::Searcher: ReverseSearcher<H>
        {
            type Item = $iterty;

            #[inline]
            fn next(&mut self) -> Option<$iterty> {
                self.0.next_back()
            }
        }

        $(#[$common_stability_attribute])*
        impl<H: Haystack, P: Pattern<H>> Clone for $reverse_iterator<H, P>
            where P::Searcher: Clone
        {
            fn clone(&self) -> Self {
                $reverse_iterator(self.0.clone())
            }
        }

        impl<H: Haystack, P: Pattern<H>> FusedIterator for $forward_iterator<H, P> {}

        impl<H: Haystack, P: Pattern<H>> FusedIterator for $reverse_iterator<H, P>
            where P::Searcher: ReverseSearcher<H> {}

        generate_pattern_iterators!($($t)* with $(#[$common_stability_attribute])*,
                                                $forward_iterator,
                                                $reverse_iterator, $iterty);
    };
    {
        double ended; with $(#[$common_stability_attribute:meta])*,
                           $forward_iterator:ident,
                           $reverse_iterator:ident, $iterty:ty
    } => {
        $(#[$common_stability_attribute])*
        impl<H: Haystack, P: Pattern<H>> DoubleEndedIterator for $forward_iterator<H, P>
            where P::Searcher: DoubleEndedSearcher<H>
        {
            #[inline]
            fn next_back(&mut self) -> Option<$iterty> {
                self.0.next_back()
            }
        }

        $(#[$common_stability_attribute])*
        impl<H: Haystack, P: Pattern<H>> DoubleEndedIterator for $reverse_iterator<H, P>
            where P::Searcher: DoubleEndedSearcher<H>
        {
            #[inline]
            fn next_back(&mut self) -> Option<$iterty> {
                self.0.next()
            }
        }
    };
    {
        single ended; with $(#[$common_stability_attribute:meta])*,
                           $forward_iterator:ident,
                           $reverse_iterator:ident, $iterty:ty
    } => {}
}

fn is_empty<H: Haystack>(haystack: &H) -> bool {
    let start = haystack.cursor_at_front();
    let end = haystack.cursor_at_back();
    let start = unsafe { haystack.start_to_end_cursor(start) };
    start >= end
}

//------------------------------------------------------------------------------
// Split
//------------------------------------------------------------------------------

derive_pattern_clone!{
    clone SplitInternal
    with |s| SplitInternal { matcher: s.matcher.clone(), ..*s }
}

pub struct SplitInternal<H: Haystack, P: Pattern<H>> {
    pub start: H::StartCursor,
    pub end: H::EndCursor,
    pub matcher: P::Searcher,
    pub allow_trailing_empty: bool,
    pub finished: bool,
}

impl<H: Haystack, P: Pattern<H>> fmt::Debug for SplitInternal<H, P>
where
    P::Searcher: fmt::Debug,
    H::StartCursor: fmt::Debug,
    H::EndCursor: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SplitInternal")
            .field("start", &self.start)
            .field("end", &self.end)
            .field("matcher", &self.matcher)
            .field("allow_trailing_empty", &self.allow_trailing_empty)
            .field("finished", &self.finished)
            .finish()
    }
}

impl<H: Haystack, P: Pattern<H>> SplitInternal<H, P> {
    #[inline]
    fn get_end(&mut self) -> Option<H> {
        if self.finished {
            return None;
        }
        let haystack = self.matcher.haystack();
        unsafe {
            if !self.allow_trailing_empty && haystack.start_to_end_cursor(self.start) >= self.end {
                return None;
            }

            self.finished = true;
            let string = haystack.range_to_self(self.start, self.end);
            Some(string)
        }
    }

    #[inline]
    fn next(&mut self) -> Option<H> {
        if self.finished { return None }

        let haystack = self.matcher.haystack();
        match self.matcher.next_match() {
            Some((a, b)) => unsafe {
                let a = haystack.start_to_end_cursor(a);
                let b = haystack.end_to_start_cursor(b);
                let elt = haystack.range_to_self(self.start, a);
                self.start = b;
                Some(elt)
            },
            None => self.get_end(),
        }
    }

    #[inline]
    fn next_back(&mut self) -> Option<H>
        where P::Searcher: ReverseSearcher<H>
    {
        if self.finished { return None }

        if !self.allow_trailing_empty {
            self.allow_trailing_empty = true;
            if let Some(elt) = self.next_back() {
                if !is_empty(&elt) {
                    return Some(elt);
                }
            }
            if self.finished {
                return None;
            }
        }

        let haystack = self.matcher.haystack();
        match self.matcher.next_match_back() {
            Some((a, b)) => unsafe {
                let a = haystack.start_to_end_cursor(a);
                let b = haystack.end_to_start_cursor(b);
                let elt = haystack.range_to_self(b, self.end);
                self.end = a;
                Some(elt)
            },
            None => unsafe {
                self.finished = true;
                Some(haystack.range_to_self(self.start, self.end))
            },
        }
    }
}

generate_pattern_iterators! {
    forward:
        /// Created with the method [`split`].
        ///
        /// [`split`]: ../../std/pattern/trait.Haystack.html#tymethod.split
        struct Split;
    reverse:
        /// Created with the method [`rsplit`].
        ///
        /// [`rsplit`]: ../../std/pattern/trait.Haystack.html#tymethod.rsplit
        struct RSplit;
    stability:
    internal:
        SplitInternal yielding (H);
    delegate double ended;
}

generate_pattern_iterators! {
    forward:
        /// Created with the method [`split_terminator`].
        ///
        /// [`split_terminator`]: ../../std/pattern/trait.Haystack.html#tymethod.split_terminator
        struct SplitTerminator;
    reverse:
        /// Created with the method [`rsplit_terminator`].
        ///
        /// [`rsplit_terminator`]: ../../std/pattern/trait.Haystack.html#tymethod.rsplit_terminator
        struct RSplitTerminator;
    stability:
    internal:
        SplitInternal yielding (H);
    delegate double ended;
}

//------------------------------------------------------------------------------
// SplitN
//------------------------------------------------------------------------------

derive_pattern_clone!{
    clone SplitNInternal
    with |s| SplitNInternal { iter: s.iter.clone(), ..*s }
}

pub struct SplitNInternal<H: Haystack, P: Pattern<H>> {
    pub iter: SplitInternal<H, P>,
    /// The number of splits remaining
    pub count: usize,
}

impl<H: Haystack, P: Pattern<H>> fmt::Debug for SplitNInternal<H, P>
where
    P::Searcher: fmt::Debug,
    H::StartCursor: fmt::Debug,
    H::EndCursor: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SplitNInternal")
            .field("iter", &self.iter)
            .field("count", &self.count)
            .finish()
    }
}

impl<H: Haystack, P: Pattern<H>> SplitNInternal<H, P> {
    #[inline]
    fn next(&mut self) -> Option<H> {
        match self.count {
            0 => None,
            1 => { self.count = 0; self.iter.get_end() }
            _ => { self.count -= 1; self.iter.next() }
        }
    }

    #[inline]
    fn next_back(&mut self) -> Option<H>
        where P::Searcher: ReverseSearcher<H>
    {
        match self.count {
            0 => None,
            1 => { self.count = 0; self.iter.get_end() }
            _ => { self.count -= 1; self.iter.next_back() }
        }
    }
}

generate_pattern_iterators! {
    forward:
        /// Created with the method [`splitn`].
        ///
        /// [`splitn`]: ../../std/primitive.str.html#method.splitn
        struct SplitN;
    reverse:
        /// Created with the method [`rsplitn`].
        ///
        /// [`rsplitn`]: ../../std/primitive.str.html#method.rsplitn
        struct RSplitN;
    stability:
    internal:
        SplitNInternal yielding (H);
    delegate single ended;
}

//------------------------------------------------------------------------------
// Matches
//------------------------------------------------------------------------------

derive_pattern_clone!{
    clone MatchesInternal
    with |s| MatchesInternal(s.0.clone())
}

pub struct MatchesInternal<H: Haystack, P: Pattern<H>>(pub P::Searcher);

impl<H: Haystack, P: Pattern<H>> fmt::Debug for MatchesInternal<H, P>
where
    P::Searcher: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("MatchesInternal")
            .field(&self.0)
            .finish()
    }
}

impl<H: Haystack, P: Pattern<H>> MatchesInternal<H, P> {
    #[inline]
    fn next(&mut self) -> Option<H> {
        self.0.next_match().map(|(a, b)| unsafe {
            self.0.haystack().range_to_self(a, b)
        })
    }

    #[inline]
    fn next_back(&mut self) -> Option<H>
        where P::Searcher: ReverseSearcher<H>
    {
        self.0.next_match_back().map(|(a, b)| unsafe {
            self.0.haystack().range_to_self(a, b)
        })
    }
}

generate_pattern_iterators! {
    forward:
        /// Created with the method [`matches`].
        ///
        /// [`matches`]: ../../std/primitive.str.html#method.matches
        struct Matches;
    reverse:
        /// Created with the method [`rmatches`].
        ///
        /// [`rmatches`]: ../../std/primitive.str.html#method.rmatches
        struct RMatches;
    stability:
    internal:
        MatchesInternal yielding (H);
    delegate double ended;
}

//------------------------------------------------------------------------------
// MatchIndices
//------------------------------------------------------------------------------

derive_pattern_clone!{
    clone MatchIndicesInternal
    with |s| MatchIndicesInternal(s.0.clone())
}

pub struct MatchIndicesInternal<H: Haystack, P: Pattern<H>>(pub P::Searcher);

impl<H: Haystack, P: Pattern<H>> fmt::Debug for MatchIndicesInternal<H, P>
where
    P::Searcher: fmt::Debug
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("MatchIndicesInternal")
            .field(&self.0)
            .finish()
    }
}

impl<H: Haystack, P: Pattern<H>> MatchIndicesInternal<H, P> {
    #[inline]
    fn next(&mut self) -> Option<(usize, H)> {
        self.0.next_match().map(|(start, end)| unsafe {
            let haystack = self.0.haystack();
            (haystack.start_cursor_to_offset(start), haystack.range_to_self(start, end))
        })
    }

    #[inline]
    fn next_back(&mut self) -> Option<(usize, H)>
        where P::Searcher: ReverseSearcher<H>
    {
        self.0.next_match_back().map(|(start, end)| unsafe {
            let haystack = self.0.haystack();
            (haystack.start_cursor_to_offset(start), haystack.range_to_self(start, end))
        })
    }
}

generate_pattern_iterators! {
    forward:
        /// Created with the method [`match_indices`].
        ///
        /// [`match_indices`]: ../../std/primitive.str.html#method.match_indices
        struct MatchIndices;
    reverse:
        /// Created with the method [`rmatch_indices`].
        ///
        /// [`rmatch_indices`]: ../../std/primitive.str.html#method.rmatch_indices
        struct RMatchIndices;
    stability:
    internal:
        MatchIndicesInternal yielding ((usize, H));
    delegate double ended;
}

//------------------------------------------------------------------------------
// MatchRanges
//------------------------------------------------------------------------------

derive_pattern_clone!{
    clone MatchRangesInternal
    with |s| MatchRangesInternal(s.0.clone())
}

pub struct MatchRangesInternal<H: Haystack, P: Pattern<H>>(pub P::Searcher);

impl<H: Haystack, P: Pattern<H>> fmt::Debug for MatchRangesInternal<H, P>
where
    P::Searcher: fmt::Debug
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("MatchRangesInternal")
            .field(&self.0)
            .finish()
    }
}

impl<H: Haystack, P: Pattern<H>> MatchRangesInternal<H, P> {
    #[inline]
    fn next(&mut self) -> Option<(Range<usize>, H)> {
        self.0.next_match().map(|(start, end)| unsafe {
            let haystack = self.0.haystack();
            let a = haystack.start_cursor_to_offset(start);
            let b = haystack.end_cursor_to_offset(end);
            (a..b, haystack.range_to_self(start, end))
        })
    }

    #[inline]
    fn next_back(&mut self) -> Option<(Range<usize>, H)>
        where P::Searcher: ReverseSearcher<H>
    {
        self.0.next_match_back().map(|(start, end)| unsafe {
            let haystack = self.0.haystack();
            let a = haystack.start_cursor_to_offset(start);
            let b = haystack.end_cursor_to_offset(end);
            (a..b, haystack.range_to_self(start, end))
        })
    }
}

generate_pattern_iterators! {
    forward:
        /// Created with the method [`match_indices`].
        ///
        /// [`match_indices`]: ../../std/primitive.str.html#method.match_indices
        struct MatchRanges;
    reverse:
        /// Created with the method [`rmatch_indices`].
        ///
        /// [`rmatch_indices`]: ../../std/primitive.str.html#method.rmatch_indices
        struct RMatchRanges;
    stability:
    internal:
        MatchRangesInternal yielding ((Range<usize>, H));
    delegate double ended;
}

//------------------------------------------------------------------------------
// ReplaceWith
//------------------------------------------------------------------------------

enum ReplaceState<H: Haystack> {
    HasNext(H::StartCursor),
    Match(H::StartCursor, H::EndCursor),
    Finished,
}

impl<H> fmt::Debug for ReplaceState<H>
where
    H: Haystack,
    H::StartCursor: fmt::Debug,
    H::EndCursor: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ReplaceState::HasNext(ref a) => f.debug_tuple("HasNext").field(a).finish(),
            ReplaceState::Match(ref a, ref b) => f.debug_tuple("Match").field(a).field(b).finish(),
            ReplaceState::Finished => f.write_str("Finished"),
        }
    }
}

impl<H: Haystack> Clone for ReplaceState<H> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<H: Haystack> Copy for ReplaceState<H> {}


///
pub struct ReplaceWith<H: Haystack, P: Pattern<H>, F> {
    searcher: P::Searcher,
    to: F,
    count: Option<usize>,
    state: ReplaceState<H>,
}

impl<H, P, F> fmt::Debug for ReplaceWith<H, P, F>
where
    H: Haystack,
    H::StartCursor: fmt::Debug,
    H::EndCursor: fmt::Debug,
    P: Pattern<H>,
    P::Searcher: fmt::Debug,
    F: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ReplaceWith")
            .field("searcher", &self.searcher)
            .field("to", &self.to)
            .field("count", &self.count)
            .field("state", &self.state)
            .finish()
    }
}

impl<H, P, F> Clone for ReplaceWith<H, P, F>
where
    H: Haystack,
    P: Pattern<H>,
    P::Searcher: Clone,
    F: Clone,
{
    fn clone(&self) -> Self {
        ReplaceWith {
            searcher: self.searcher.clone(),
            to: self.to.clone(),
            count: self.count,
            state: self.state,
        }
    }
}

impl<H: Haystack, P: Pattern<H>, F> ReplaceWith<H, P, F> {
    #[inline]
    pub(super) fn new(haystack: H, pat: P, to: F, count: Option<usize>) -> Self {
        let state = ReplaceState::HasNext(haystack.cursor_at_front());
        ReplaceWith {
            searcher: pat.into_searcher(haystack),
            to,
            count,
            state,
        }
    }

    fn next_match(&mut self) -> Option<(H::StartCursor, H::EndCursor)> {
        if let Some(ref mut i) = self.count {
            if *i == 0 {
                return None;
            } else {
                *i -= 1;
            }
        }
        self.searcher.next_match()
    }
}

impl<H, P, F, B> Iterator for ReplaceWith<H, P, F>
where
    H: Haystack,
    P: Pattern<H>,
    B: From<H>,
    F: FnMut(H) -> B,
{
    type Item = B;

    fn next(&mut self) -> Option<B> {
        let (next_state, ret_val) = match self.state {
            ReplaceState::Finished => (ReplaceState::Finished, None),
            ReplaceState::HasNext(last_end) => {
                let haystack = self.searcher.haystack();
                unsafe {
                    let (next_state, cur_start) = if let Some((a, b)) = self.next_match() {
                        (ReplaceState::Match(a, b), haystack.start_to_end_cursor(a))
                    } else {
                        (ReplaceState::Finished, haystack.cursor_at_back())
                    };
                    (next_state, Some(haystack.range_to_self(last_end, cur_start).into()))
                }
            }
            ReplaceState::Match(cur_start, cur_end) => {
                let haystack = self.searcher.haystack();
                unsafe {
                    (
                        ReplaceState::HasNext(haystack.end_to_start_cursor(cur_end)),
                        Some((self.to)(haystack.range_to_self(cur_start, cur_end))),
                    )
                }
            }
        };
        self.state = next_state;
        ret_val
    }
}

impl<H, P, F, B> FusedIterator for ReplaceWith<H, P, F>
where
    H: Haystack,
    P: Pattern<H>,
    B: From<H>,
    F: FnMut(H) -> B,
{}
