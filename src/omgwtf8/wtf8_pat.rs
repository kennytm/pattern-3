use pattern::*;
use haystack::Span;
use std::ops::Range;
use slices::slice::{TwoWaySearcher, SliceSearcher, SliceChecker};
#[cfg(test)]
use ext::{match_ranges, rmatch_ranges, starts_with, ends_with};

use super::wtf8::{HighSurrogate, LowSurrogate, ThreeByteSeq, Wtf8};

// The TwoWaySearcher will not match unpaired surrogate at boundary, so no need
// to convert logical range to physical range.

unsafe impl<'p> Searcher<Wtf8> for TwoWaySearcher<'p, u8> {
    #[inline]
    fn search(&mut self, span: Span<&Wtf8>) -> Option<Range<usize>> {
        let (hay, range) = span.into_parts();
        self.next(hay.as_inner(), range)
    }
}

unsafe impl<'p> ReverseSearcher<Wtf8> for TwoWaySearcher<'p, u8> {
    #[inline]
    fn rsearch(&mut self, span: Span<&Wtf8>) -> Option<Range<usize>> {
        let (hay, range) = span.into_parts();
        self.next_back(hay.as_inner(), range)
    }
}

fn span_as_inner(span: Span<&Wtf8>) -> Span<&[u8]> {
    let (hay, range) = span.into_parts();
    unsafe { Span::from_parts(hay.as_inner(), range) }
}

unsafe impl<'p> Checker<Wtf8> for SliceChecker<'p, u8> {
    #[inline]
    fn check(&mut self, span: Span<&Wtf8>) -> Option<usize> {
        self.check(span_as_inner(span))
    }
    #[inline]
    fn trim_start(&mut self, haystack: &Wtf8) -> usize {
        self.trim_start(haystack.as_inner())
    }
}

unsafe impl<'p> ReverseChecker<Wtf8> for SliceChecker<'p, u8> {
    #[inline]
    fn rcheck(&mut self, span: Span<&Wtf8>) -> Option<usize> {
        self.rcheck(span_as_inner(span))
    }
    #[inline]
    fn trim_end(&mut self, haystack: &Wtf8) -> usize {
        self.trim_end(haystack.as_inner())
    }
}

#[derive(Debug)]
enum SurrogateType {
    Split,
    Canonical,
    Empty,
}

fn extend_subrange(
    range: Range<usize>,
    mut subrange: Range<usize>,
    low_type: SurrogateType,
    high_type: SurrogateType,
) -> Range<usize> {
    subrange.start -= match low_type {
        SurrogateType::Empty => 0,
        SurrogateType::Split if subrange.start != range.start + 3 => 2,
        _ => 3,
    };
    subrange.end += match high_type {
        SurrogateType::Empty => 0,
        SurrogateType::Split if subrange.end + 3 != range.end => 2,
        _ => 3,
    };
    subrange
}

#[derive(Debug, Clone)]
pub struct LowSurrogateSearcher {
    canonical: u16,
}

impl LowSurrogateSearcher {
    #[inline]
    fn new(ls: LowSurrogate) -> Self {
        Self {
            canonical: ls.value() & 0xcfff,
        }
    }

    #[inline]
    fn is_match(&self, tbs: ThreeByteSeq) -> Option<SurrogateType> {
        let tbs = tbs.value();
        if (tbs & 0xcfff) as u16 != self.canonical {
            return None;
        }
        match tbs >> 12 {
            0xedb => Some(SurrogateType::Canonical),
            0x800..=0xbff => Some(SurrogateType::Split),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HighSurrogateSearcher {
    canonical: u32,
    split: u32,
}

impl HighSurrogateSearcher {
    #[inline]
    fn new(hs: HighSurrogate) -> Self {
        // the canonical representation
        //
        //          c = 1010 jihg 10fe dcba
        //
        // rearrange
        //
        //  c & 0xf03 = 0000 jihg 0000 00ba
        //   c & 0xfc = 0000 0000 00fe dc00
        // ...|...<<2 = 0000 jihg fedc 00ba
        //  ...+0x100 = 000K JIHG fedc 00ba
        //
        // rearrange again
        //
        //  s & 0x3ff = 0000 00HG fedc 00ba
        // s & 0xfc00 = 000K JI00 0000 0000
        // ...|...<<2 = 0KJI 00HG fedc 00ba
        //  ...|0x808 = 0KJI 10HG fedc 10ba
        //
        // this will be the split representation shifted right by 4 bits.

        let c = hs.value();
        let s = ((c & 0xf03) | (c & 0x3c) << 2) + 0x100;
        let s = (s & 0x3ff) | (s & 0xfc00) << 2 | 0x808;
        Self {
            canonical: c as u32 | 0xed0000,
            split: s as u32 | 0xf0000,
        }
    }

    #[inline]
    fn is_match(&self, tbs: ThreeByteSeq) -> Option<SurrogateType> {
        let tbs = tbs.value();
        if tbs == self.canonical {
            Some(SurrogateType::Canonical)
        } else if tbs >> 4 == self.split {
            Some(SurrogateType::Split)
        } else {
            None
        }
    }
}

// pub enum Wtf8Searcher<'p> {
//     Empty(EmptySearcher),
//     Surrogates {
//         low: Option<LowSurrogateSearcher>,
//         high: Option<HighSurrogateSearcher>,
//     },
//     TwoWay {
//         low: Option<LowSurrogateSearcher>,
//         middle: TwoWaySearcher<'p, u8>,
//         high: Option<HighSurrogateSearcher>,
//     },
// }

#[derive(Debug, Clone)]
pub struct Wtf8Searcher<'p> {
    low: Option<LowSurrogateSearcher>,
    middle: SliceSearcher<'p, u8>,
    high: Option<HighSurrogateSearcher>,
}

fn compare_boundary_surrogates(
    low: &Option<LowSurrogateSearcher>,
    high: &Option<HighSurrogateSearcher>,
    bytes: &[u8],
    range: Range<usize>,
    subrange: Range<usize>,
) -> Option<(SurrogateType, SurrogateType)> {
    let low_type = if let Some(low) = low {
        if subrange.start - range.start < 3 {
            return None;
        }
        let tbs = unsafe { bytes.get_unchecked((subrange.start - 3)..subrange.start) };
        low.is_match(ThreeByteSeq::new(tbs))?
    } else {
        SurrogateType::Empty
    };

    let high_type = if let Some(high) = high {
        if range.end - subrange.end < 3 {
            return None;
        }
        let tbs = unsafe { bytes.get_unchecked(subrange.end..(subrange.end + 3)) };
        high.is_match(ThreeByteSeq::new(tbs))?
    } else {
        SurrogateType::Empty
    };

    Some((low_type, high_type))
}

unsafe impl<'p> Searcher<Wtf8> for Wtf8Searcher<'p> {
    #[inline]
    fn search(&mut self, mut span: Span<&Wtf8>) -> Option<Range<usize>> {
        let (hay, range) = span.clone().into_parts();
        while let Some(subrange) = self.middle.search(span.clone()) {
            if let Some((low_type, high_type)) = compare_boundary_surrogates(
                &self.low,
                &self.high,
                hay.as_inner(),
                range.clone(),
                subrange.clone(),
            ) {
                return Some(extend_subrange(range, subrange, low_type, high_type));
            } else {
                span = unsafe { Span::from_parts(hay, subrange.end..range.end) };
            }
        }
        None
    }
}

unsafe impl<'p> ReverseSearcher<Wtf8> for Wtf8Searcher<'p> {
    #[inline]
    fn rsearch(&mut self, mut span: Span<&Wtf8>) -> Option<Range<usize>> {
        let (hay, range) = span.clone().into_parts();
        while let Some(subrange) = self.middle.rsearch(span.clone()) {
            if let Some((low_type, high_type)) = compare_boundary_surrogates(
                &self.low,
                &self.high,
                hay.as_inner(),
                range.clone(),
                subrange.clone(),
            ) {
                return Some(extend_subrange(range, subrange, low_type, high_type));
            } else {
                span = unsafe { Span::from_parts(hay, range.start..subrange.start) };
            }
        }
        None

        // match self {
        //     Wtf8Searcher::Empty(s) => s.rsearch(span),
        //     Wtf8Searcher::Surrogates { low, high } => {
        //         let (hay, range) = span.into_parts();
        //         let bytes = hay.as_inner();
        //         let mut index = range.end;
        //         loop {
        //             match compare_boundary_surrogates(low, high, bytes, range.clone(), index..index) {
        //                 Some((low_type, high_type)) => {
        //                     return Some(extend_subrange(range, index..index, low_type, high_type));
        //                 }
        //                 None if index > range.start => {
        //                     index = unsafe { hay.prev_index(index) };
        //                 }
        //                 None => {
        //                     return None;
        //                 }
        //             }
        //         }
        //     }
        //     Wtf8Searcher::TwoWay { low, middle, high } => {
        //         let (hay, range) = span.into_parts();
        //         let bytes = hay.as_inner();
        //         let mut span = unsafe { Span::from_parts(bytes, range.clone()) };
        //         while let Some(subrange) = middle.rsearch(span.clone()) {
        //             match compare_boundary_surrogates(low, high, bytes, range.clone(), subrange.clone()) {
        //                 Some((low_type, high_type)) => {
        //                     return Some(extend_subrange(range, subrange, low_type, high_type));
        //                 }
        //                 None => {
        //                     span = unsafe { span.slice_unchecked(range.start..subrange.start) };
        //                 }
        //             }
        //         }
        //         None
        //     }
        // }
    }
}

pub struct Wtf8Checker<'p> {
    low: Option<LowSurrogateSearcher>,
    middle: &'p [u8],
    high: Option<HighSurrogateSearcher>,
}

unsafe impl<'p> Checker<Wtf8> for Wtf8Checker<'p> {
    #[inline]
    fn check(&mut self, span: Span<&Wtf8>) -> Option<usize> {
        let (hay, range) = span.into_parts();
        let bytes = hay[range.clone()].as_inner();
        let low_len = if self.low.is_some() { 3 } else { 0 };
        let high_len = if self.high.is_some() { 3 } else { 0 };
        let mut match_len = low_len + self.middle.len() + high_len;
        if bytes.len() < match_len {
            return None;
        }
        let middle_start = low_len;
        let middle_end = match_len - high_len;
        if &bytes[middle_start..middle_end] != self.middle {
            return None;
        }
        if let Some(high) = &self.high {
            if let SurrogateType::Split = high.is_match(ThreeByteSeq::new(&bytes[middle_end..]))? {
                if bytes.len() != match_len {
                    match_len -= 1;
                }
            }
        }
        if let Some(low) = &self.low {
            if let SurrogateType::Split = low.is_match(ThreeByteSeq::new(bytes))? {
                if range.start != 0 {
                    match_len -= 1;
                }
            }
        }
        Some(range.start + match_len)
    }
}

unsafe impl<'p> ReverseChecker<Wtf8> for Wtf8Checker<'p> {
    #[inline]
    fn rcheck(&mut self, span: Span<&Wtf8>) -> Option<usize> {
        let (hay, range) = span.into_parts();
        let bytes = hay[range.clone()].as_inner();
        let low_len = if self.low.is_some() { 3 } else { 0 };
        let high_len = if self.high.is_some() { 3 } else { 0 };
        let mut match_len = low_len + self.middle.len() + high_len;
        if bytes.len() < match_len {
            return None;
        }
        let middle_end = bytes.len() - high_len;
        let middle_start = middle_end - self.middle.len();
        if &bytes[middle_start..middle_end] != self.middle {
            return None;
        }
        if let Some(low) = &self.low {
            let start_index = bytes.len() - match_len;
            if let SurrogateType::Split = low.is_match(ThreeByteSeq::new(&bytes[start_index..]))? {
                if start_index != 0 {
                    match_len -= 1;
                }
            }
        }
        if let Some(high) = &self.high {
            if let SurrogateType::Split = high.is_match(ThreeByteSeq::new(&bytes[middle_end..]))? {
                if bytes.len() != range.end {
                    match_len -= 1;
                }
            }
        }
        Some(range.end - match_len)
    }
}

impl<'h, 'p> Pattern<&'h Wtf8> for &'p Wtf8 {
    type Searcher = Wtf8Searcher<'p>;
    type Checker = Wtf8Checker<'p>;

    fn into_searcher(self) -> Self::Searcher {
        let (low, middle, high) = self.canonicalize();
        Wtf8Searcher {
            low: low.map(LowSurrogateSearcher::new),
            middle: SliceSearcher::new(middle),
            high: high.map(HighSurrogateSearcher::new),
        }
    }

    fn into_checker(self) -> Self::Checker {
        let (low, middle, high) = self.canonicalize();
        Wtf8Checker {
            low: low.map(LowSurrogateSearcher::new),
            middle,
            high: high.map(HighSurrogateSearcher::new),
        }
    }
}

impl<'h, 'p> Pattern<&'h Wtf8> for &'p str {
    type Searcher = SliceSearcher<'p, u8>;
    type Checker = SliceChecker<'p, u8>;

    fn into_searcher(self) -> Self::Searcher {
        SliceSearcher::new(self.as_bytes())
    }

    fn into_checker(self) -> Self::Checker {
        SliceChecker(self.as_bytes())
    }
}
