use needle::*;
use haystack::{Haystack, Span};
use std::ops::Range;
use slices::slice::{TwoWaySearcher, SliceSearcher, NaiveSearcher};
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

unsafe impl<'p> Consumer<Wtf8> for NaiveSearcher<'p, u8> {
    #[inline]
    fn consume(&mut self, span: Span<&Wtf8>) -> Option<usize> {
        self.consume(span_as_inner(span))
    }

    #[inline]
    fn trim_start(&mut self, hay: &Wtf8) -> usize {
        self.trim_start(hay.as_inner())
    }
}

unsafe impl<'p> ReverseConsumer<Wtf8> for NaiveSearcher<'p, u8> {
    #[inline]
    fn rconsume(&mut self, span: Span<&Wtf8>) -> Option<usize> {
        self.rconsume(span_as_inner(span))
    }

    #[inline]
    fn trim_end(&mut self, hay: &Wtf8) -> usize {
        self.trim_end(hay.as_inner())
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

#[derive(Debug, Clone)]
pub struct Wtf8Searcher<S> {
    low: Option<LowSurrogateSearcher>,
    middle: S,
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

unsafe impl<'p> Searcher<Wtf8> for Wtf8Searcher<SliceSearcher<'p, u8>> {
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

unsafe impl<'p> Consumer<Wtf8> for Wtf8Searcher<NaiveSearcher<'p, u8>> {
    #[inline]
    fn consume(&mut self, span: Span<&Wtf8>) -> Option<usize> {
        let (hay, range) = span.into_parts();
        let bytes = hay[range.clone()].as_inner();
        let low_len = if self.low.is_some() { 3 } else { 0 };
        let high_len = if self.high.is_some() { 3 } else { 0 };
        let middle = self.middle.needle();
        let mut match_len = low_len + middle.len() + high_len;
        if bytes.len() < match_len {
            return None;
        }
        let middle_start = low_len;
        let middle_end = match_len - high_len;
        if &bytes[middle_start..middle_end] != middle {
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

unsafe impl<'p> ReverseSearcher<Wtf8> for Wtf8Searcher<SliceSearcher<'p, u8>> {
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
    }
}

unsafe impl<'p> ReverseConsumer<Wtf8> for Wtf8Searcher<NaiveSearcher<'p, u8>> {
    #[inline]
    fn rconsume(&mut self, span: Span<&Wtf8>) -> Option<usize> {
        let (hay, range) = span.into_parts();
        let bytes = hay[range.clone()].as_inner();
        let low_len = if self.low.is_some() { 3 } else { 0 };
        let high_len = if self.high.is_some() { 3 } else { 0 };
        let middle = self.middle.needle();
        let mut match_len = low_len + middle.len() + high_len;
        if bytes.len() < match_len {
            return None;
        }
        let middle_end = bytes.len() - high_len;
        let middle_start = middle_end - middle.len();
        if &bytes[middle_start..middle_end] != middle {
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

impl<'p, H: Haystack<Target = Wtf8>> Needle<H> for &'p Wtf8 {
    type Searcher = Wtf8Searcher<SliceSearcher<'p, u8>>;
    type Consumer = Wtf8Searcher<NaiveSearcher<'p, u8>>;

    fn into_searcher(self) -> Self::Searcher {
        let (low, middle, high) = self.canonicalize();
        Wtf8Searcher {
            low: low.map(LowSurrogateSearcher::new),
            middle: SliceSearcher::new(middle),
            high: high.map(HighSurrogateSearcher::new),
        }
    }

    fn into_consumer(self) -> Self::Consumer {
        let (low, middle, high) = self.canonicalize();
        Wtf8Searcher {
            low: low.map(LowSurrogateSearcher::new),
            middle: NaiveSearcher::new(middle),
            high: high.map(HighSurrogateSearcher::new),
        }
    }
}

// FIXME cannot impl `Needle<(_: Haystack<Target = Wtf8>)>` due to RFC 1672 being postponed.
// (need to wait for chalk)
impl<'h, 'p> Needle<&'h Wtf8> for &'p str {
    type Searcher = SliceSearcher<'p, u8>;
    type Consumer = NaiveSearcher<'p, u8>;

    fn into_searcher(self) -> Self::Searcher {
        SliceSearcher::new(self.as_bytes())
    }

    fn into_consumer(self) -> Self::Consumer {
        NaiveSearcher::new(self.as_bytes())
    }
}
