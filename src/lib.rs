#![feature(
    ptr_offset_from,
    range_is_empty,
    fn_traits,
    unboxed_closures,
    linkage,
    specialization,
    try_trait,
    fundamental,
    test,
    align_offset,
    slice_internals,
)]
#![allow(warnings)]

extern crate memchr;

// extern crate memchr;

// use std::ops::Range;
// use std::borrow::Borrow;
// use std::marker::PhantomData;
// use std::cmp::Ordering;
// use std::ptr;
// use std::slice;
// use std::str;
// use std::iter::DoubleEndedIterator;
// use std::ops::Try;
// use std::mem;

mod memchr_ptr;

// pub mod traits;
// pub mod span;
// pub mod slices;
// pub mod strings;
// pub mod ext;
// pub mod not;

// pub use traits::{Pattern, Haystack, HaystackMut, Searcher, Rev};
// pub use span::Span;

// mod iterators;
// mod string;

// pub use self::iterators::{
//     Split, RSplit, SplitN, RSplitN, SplitTerminator, RSplitTerminator,
//     Searcher, RMatches, MatchIndices, RMatchIndices, MatchRanges, RMatchRanges,
//     ReplaceWith,
// };


fn bench_data() -> Vec<u8> { std::iter::repeat(b'z').take(10000).collect() }

#[bench]
fn iterator_memchr(b: &mut test::Bencher) {
    let haystack = bench_data();
    let needle = b'a';
    b.iter(|| {
        assert!(haystack.iter().position(|&b| b == needle).is_none());
    });
    b.bytes = haystack.len() as u64;
}

#[bench]
fn optimized_memchr(b: &mut test::Bencher) {
    let haystack = bench_data();
    let needle = b'a';
    b.iter(|| {
        assert!(memchr::memchr(needle, &haystack).is_none());
    });
    b.bytes = haystack.len() as u64;
}


#[bench]
fn optimized_core_memchr(b: &mut test::Bencher) {
    let haystack = bench_data();
    let needle = b'a';
    b.iter(|| {
        assert!(core::slice::memchr::memchr(needle, &haystack).is_none());
    });
    b.bytes = haystack.len() as u64;
}


#[bench]
fn optimized_memchr_ptr(b: &mut test::Bencher) {
    let haystack = bench_data();
    let needle = b'a';
    let haystack_end = unsafe { haystack.as_ptr().add(haystack.len()) };
    b.iter(|| {
        assert!(memchr_ptr::memchr_ptr(needle, &haystack) == std::ptr::null());
    });
    b.bytes = haystack.len() as u64;
}

#[bench]
fn optimized_memchr_libc(b: &mut test::Bencher) {
    let haystack = bench_data();
    let needle = b'a';
    b.iter(|| unsafe {
        assert!(libc::memchr(haystack.as_ptr() as *const libc::c_void, needle as i32, haystack.len()) == std::ptr::null_mut());
    });
    b.bytes = haystack.len() as u64;
}






// impl<'a, T: Eq + 'a> ReversePattern<'a, [T]> for T {
//     fn is_suffix_of(self, haystack: &'a [T]) -> bool {
//         haystack.last() == Some(&self)
//     }
// }

// impl<'r, 't> Pattern<'t, str> for &'r Regex {
//     type Searcher = RegexSearcher<'r, 't>;

//     fn into_searcher(self, haystack: &'t str) -> Self::Searcher {
//         RegexSearcher {
//             matches: self.find_iter(haystack),
//             remaining: CursorRange::full(haystack),
//         }
//     }

//     fn is_prefix_of(self, haystack: &'t str) -> bool {
//         self.find(haystack).map(|m| m.start()) == Some(0)
//     }

//     fn is_contained_in(self, haystack: &'t str) -> bool {
//         self.is_match(haystack)
//     }
// }

// struct RegexSearcher<'r, 't> {
//     matches: regex::Matches<'r, 't>,
//     remaining: CursorRange<'t, str>,
// }

// impl<'r, 't> Iterator for RegexSearcher<'r, 't> {
//     type Item = CursorRange<'t, str>;

//     fn next(&mut self) -> Option<Self::Item> {
//         let m = self.matches.next()?;
//         unsafe {
//             let start_ptr = self.remaining.haystack.as_ptr().add(m.start());
//             let end_ptr = self.remaining.haystack.as_ptr().add(m.end());
//             let (_, result, remaining) = self.remaining.split_with_cursors(start_ptr, end_ptr);
//             self.remaining = remaining;
//             Some(result)
//         }
//     }
// }

// unsafe impl<'r, 't> Searcher<'t, str> for RegexSearcher<'r, 't> {
//     fn remaining(&self) -> CursorRange<'t, str> {
//         self.remaining
//     }
// }





// pub struct ElemMatches<T>(T);


// pub fn split<'a, H, P>(haystack: &'a H, pattern: P) -> impl Iterator<Item=&'a H>
// where
//     H: Haystack + ?Sized + 'a,
//     P: Pattern<'a, H>,
// {
//     Split::<'a, H, P> {
//         range: CursorRange::full(haystack),
//         matcher: pattern.into_searcher(haystack),
//         allow_trailing_empty: true,
//         finished: false,
//     }.map(|cr| cr.into_slice())
// }

// struct Split<'a, H, P>
// where
//     H: Haystack + ?Sized + 'a,
//     P: Pattern<'a, H>
// {
//     range: CursorRange<'a, H>,
//     matcher: P::Searcher,
//     allow_trailing_empty: bool,
//     finished: bool,
// }

// impl<'a, H, P> Split<'a, H, P>
// where
//     H: Haystack + ?Sized + 'a,
//     P: Pattern<'a, H>,
// {
//     fn get_end(&mut self) -> Option<CursorRange<'a, H>> {
//         if self.finished {
//             return None;
//         }
//         if !self.allow_trailing_empty && self.range.is_empty() {
//             return None;
//         }
//         self.finished = true;
//         Some(self.range)
//     }
// }

// impl<'a, H, P> Iterator for Split<'a, H, P>
// where
//     H: Haystack + ?Sized + 'a,
//     P: Pattern<'a, H>,
// {
//     type Item = CursorRange<'a, H>;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.finished {
//             return None;
//         }
//         match self.matcher.next() {
//             Some(cr) => {
//                 let (elt, rest) = self.range.split(cr);
//                 self.range = rest;
//                 Some(elt)
//             }
//             None => self.get_end(),
//         }
//     }
// }

// #[test]
// fn test_split() {
//     assert_eq!(
//         split(&[1, 2, 3, 4, 5, 6, 7][..], 4).collect::<Vec<_>>(),
//         vec![&[1, 2, 3][..], &[5, 6, 7][..]],
//     );

//     assert_eq!(
//         split(&[1, 2, 3, 3, 2, 5, 5, 5, 2][..], 2).collect::<Vec<_>>(),
//         vec![&[1][..], &[3, 3][..], &[5, 5, 5][..], &[][..]],
//     );

//     assert_eq!(
//         split(&[1, 1, 1][..], 1).collect::<Vec<_>>(),
//         vec![&[][..], &[][..], &[][..], &[][..]],
//     );
// }

// #[test]
// fn test_split_not() {
//     assert_eq!(
//         split(&[4, 4, 4, 5, 4, 4, 3, 4][..], Not(4)).collect::<Vec<_>>(),
//         vec![&[4, 4, 4][..], &[4, 4][..], &[4][..]],
//     );
// }

/*

    //--------------------------------------------------------------------------
    // Extension methods for searching a pattern from a haystack.
    //--------------------------------------------------------------------------

    /// Returns `true` if the given pattern matches a sub-slice of this
    /// haystack.
    ///
    /// Returns `false` if it does not.
    #[inline]
    fn contains<P: Pattern<Self>>(self, pat: P) -> bool {
        pat.is_contained_in(self)
    }

    /// Returns `true` if the given pattern matches a prefix of this haystack.
    ///
    /// Returns `false` if it does not.
    #[inline]
    fn starts_with<P: Pattern<Self>>(self, pat: P) -> bool {
        pat.is_prefix_of(self)
    }

    /// Returns `true` if the given pattern matches a suffix of this haystack.
    ///
    /// Returns `false` if it does not.
    #[inline]
    fn ends_with<P: Pattern<Self>>(self, pat: P) -> bool
    where
        P::Searcher: ReverseSearcher<Self>
    {
        pat.is_suffix_of(self)
    }

    /// Returns the start index of the first sub-slice of this haystack that
    /// matches the pattern.
    ///
    /// Returns [`None`] if the pattern doesn't match.
    fn find<P: Pattern<Self>>(self, pat: P) -> Option<usize> {
        let mut searcher = pat.into_searcher(self);
        let cursor = searcher.next_match()?.0;
        unsafe {
            Some(Haystack::start_cursor_to_offset(&searcher.haystack(), cursor))
        }
    }

    /// Returns the start index of the last sub-slice of this haystack that
    /// matches the pattern.
    ///
    /// Returns [`None`] if the pattern doesn't match.
    fn rfind<P: Pattern<Self>>(self, pat: P) -> Option<usize>
    where
        P::Searcher: ReverseSearcher<Self>
    {
        let mut searcher = pat.into_searcher(self);
        let cursor = searcher.next_match_back()?.0;
        unsafe {
            Some(searcher.haystack().start_cursor_to_offset(cursor))
        }
    }

    /// Returns the range of the first sub-slice of this haystack that matches
    /// the pattern.
    ///
    /// Returns [`None`] if the pattern doesn't match.
    fn find_range<P: Pattern<Self>>(self, pat: P) -> Option<Range<usize>> {
        let mut searcher = pat.into_searcher(self);
        let range = searcher.next_match()?;
        let hs = searcher.haystack();
        unsafe {
            Some(hs.start_cursor_to_offset(range.0) .. hs.end_cursor_to_offset(range.1))
        }
    }

    /// Returns the range of the last sub-slice of this haystack that matches
    /// the pattern.
    ///
    /// Returns [`None`] if the pattern doesn't match.
    fn rfind_range<P: Pattern<Self>>(self, pat: P) -> Option<Range<usize>>
    where
        P::Searcher: ReverseSearcher<Self>,
    {
        let mut searcher = pat.into_searcher(self);
        let range = searcher.next_match_back()?;
        let hs = searcher.haystack();
        unsafe {
            Some(hs.start_cursor_to_offset(range.0) .. hs.end_cursor_to_offset(range.1))
        }
    }

    /// An iterator over sub-slices of this haystack, separated by a pattern.
    #[inline]
    fn split<P: Pattern<Self>>(self, pat: P) -> Split<Self, P> {
        Split(iterators::SplitInternal {
            start: self.cursor_at_front(),
            end: self.cursor_at_back(),
            matcher: pat.into_searcher(self),
            allow_trailing_empty: true,
            finished: false,
        })
    }

    /// An iterator over sub-slices of the given haystack, separated by a
    /// pattern and yielded in reverse order.
    #[inline]
    fn rsplit<P: Pattern<Self>>(self, pat: P) -> RSplit<Self, P>
    where
        P::Searcher: ReverseSearcher<Self>,
    {
        RSplit(self.split(pat).0)
    }

    ///
    #[inline]
    fn splitn<P: Pattern<Self>>(self, count: usize, pat: P) -> SplitN<Self, P> {
        SplitN(iterators::SplitNInternal {
            iter: self.split(pat).0,
            count,
        })
    }

    ///
    #[inline]
    fn rsplitn<P: Pattern<Self>>(self, count: usize, pat: P) -> RSplitN<Self, P>
    where
        P::Searcher: ReverseSearcher<Self>,
    {
        RSplitN(self.splitn(count, pat).0)
    }

    /// An iterator over sub-slices of the given haystack, separated by pattern.
    #[inline]
    fn split_terminator<P: Pattern<Self>>(self, pat: P) -> SplitTerminator<Self, P> {
        SplitTerminator(iterators::SplitInternal {
            allow_trailing_empty: false,
            ..self.split(pat).0
        })
    }

    /// An iterator over sub-slices of `self`, separated by a pattern and
    /// yielded in reverse order.
    #[inline]
    fn rsplit_terminator<P: Pattern<Self>>(self, pat: P) -> RSplitTerminator<Self, P>
    where
        P::Searcher: ReverseSearcher<Self>,
    {
        RSplitTerminator(self.split_terminator(pat).0)
    }

    ///
    #[inline]
    fn matches<P: Pattern<Self>>(self, pat: P) -> Searcher<Self, P> {
        Searcher(iterators::MatchesInternal(pat.into_searcher(self)))
    }

    ///
    #[inline]
    fn rmatches<P: Pattern<Self>>(self, pat: P) -> RMatches<Self, P>
    where
        P::Searcher: ReverseSearcher<Self>,
    {
        RMatches(self.matches(pat).0)
    }

    ///
    #[inline]
    fn match_indices<P: Pattern<Self>>(self, pat: P) -> MatchIndices<Self, P> {
        MatchIndices(iterators::MatchIndicesInternal(pat.into_searcher(self)))
    }

    ///
    #[inline]
    fn rmatch_indices<P: Pattern<Self>>(self, pat: P) -> RMatchIndices<Self, P>
    where
        P::Searcher: ReverseSearcher<Self>,
    {
        RMatchIndices(self.match_indices(pat).0)
    }

    ///
    #[inline]
    fn match_ranges<P: Pattern<Self>>(self, pat: P) -> MatchRanges<Self, P> {
        MatchRanges(iterators::MatchRangesInternal(pat.into_searcher(self)))
    }

    ///
    #[inline]
    fn rmatch_ranges<P: Pattern<Self>>(self, pat: P) -> RMatchRanges<Self, P>
    where
        P::Searcher: ReverseSearcher<Self>,
    {
        RMatchRanges(self.match_ranges(pat).0)
    }

    ///
    #[inline]
    fn trim_matches<P: Pattern<Self>>(self, pat: P) -> Self
    where
        P::Searcher: DoubleEndedSearcher<Self>
    {
        let mut i = self.cursor_at_front();
        let mut j = unsafe { self.start_to_end_cursor(i) };
        let mut matcher = pat.into_searcher(self);
        if let Some((a, b)) = matcher.next_reject() {
            i = a;
            j = b;
            // Remember earliest known match, correct it below if last match is different
        }
        if let Some((_, b)) = matcher.next_reject_back() {
            j = b;
        }
        unsafe {
            // Searcher is known to return valid indices
            matcher.haystack().range_to_self(i, j)
        }
    }

    ///
    #[inline]
    fn trim_left_matches<P: Pattern<Self>>(self, pat: P) -> Self {
        let end = self.cursor_at_back();
        let mut i = unsafe { self.end_to_start_cursor(end) };
        let mut matcher = pat.into_searcher(self);
        if let Some((a, _)) = matcher.next_reject() {
            i = a;
        }
        unsafe {
            // Searcher is known to return valid indices
            matcher.haystack().range_to_self(i, end)
        }
    }

    ///
    #[inline]
    fn trim_right_matches<P: Pattern<Self>>(self, pat: P) -> Self
    where
        P::Searcher: ReverseSearcher<Self>,
    {
        let start = self.cursor_at_front();
        let mut j = unsafe { self.start_to_end_cursor(start) };
        let mut matcher = pat.into_searcher(self);
        if let Some((_, b)) = matcher.next_reject_back() {
            j = b;
        }
        unsafe {
            // Searcher is known to return valid indices
            matcher.haystack().range_to_self(start, j)
        }
    }

    /// Performs generic replacement.
    #[inline]
    fn replace_with<P, B, F>(self, pat: P, to: F, count: Option<usize>) -> ReplaceWith<Self, P, F>
    where
        P: Pattern<Self>,
        B: From<Self>,
        F: FnMut(Self) -> B,
    {
        ReplaceWith::new(self, pat, to, count)
    }
}

*/