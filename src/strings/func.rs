use pattern::*;
use haystack::Span;
use std::ops::Range;

#[derive(Copy, Clone, Debug)]
pub struct MultiCharEq<'p>(&'p [char]);

impl<'p> FnOnce<(char,)> for MultiCharEq<'p> {
    type Output = bool;
    #[inline]
    extern "rust-call" fn call_once(self, args: (char,)) -> bool {
        self.call(args)
    }
}

impl<'p> FnMut<(char,)> for MultiCharEq<'p> {
    #[inline]
    extern "rust-call" fn call_mut(&mut self, args: (char,)) -> bool {
        self.call(args)
    }
}

impl<'p> Fn<(char,)> for MultiCharEq<'p> {
    #[inline]
    extern "rust-call" fn call(&self, (c,): (char,)) -> bool {
        self.0.iter().any(|ch| *ch == c)
    }
}

pub struct MultiCharSearcher<F> {
    predicate: F,
}

unsafe impl<F: FnMut(char) -> bool> Searcher<str> for MultiCharSearcher<F> {
    #[inline]
    fn search(&mut self, span: Span<&str>) -> Option<Range<usize>> {
        let (hay, range) = span.into_parts();
        let st = range.start;
        let h = &hay[range];
        let mut chars = h.chars();
        let c = chars.find(|c| (self.predicate)(*c))?;
        let end = chars.as_str().as_ptr();
        let end = unsafe { end.offset_from(h.as_ptr()) as usize } + st;
        Some((end - c.len_utf8())..end)
    }
}

unsafe impl<F: FnMut(char) -> bool> Consumer<str> for MultiCharSearcher<F> {
    #[inline]
    fn consume(&mut self, hay: Span<&str>) -> Option<usize> {
        let (hay, range) = hay.into_parts();
        let start = range.start;
        if start == range.end {
            return None;
        }
        let c = unsafe { hay.get_unchecked(start..) }.chars().next().unwrap();
        if (self.predicate)(c) {
            Some(start + c.len_utf8())
        } else {
            None
        }
    }

    #[inline]
    fn trim_start(&mut self, hay: &str) -> usize {
        let mut chars = hay.chars();
        let unconsume_amount = chars
            .find_map(|c| if !(self.predicate)(c) { Some(c.len_utf8()) } else { None })
            .unwrap_or(0);
        let consumed = unsafe { chars.as_str().as_ptr().offset_from(hay.as_ptr()) as usize };
        consumed - unconsume_amount
    }
}

unsafe impl<F: FnMut(char) -> bool> ReverseSearcher<str> for MultiCharSearcher<F> {
    #[inline]
    fn rsearch(&mut self, span: Span<&str>) -> Option<Range<usize>> {
        let (hay, range) = span.into_parts();
        let st = range.start;
        let h = &hay[range];
        let mut chars = h.chars();
        let c = chars.rfind(|c| (self.predicate)(*c))?;
        let start = chars.as_str().len() + st;
        Some(start..(start + c.len_utf8()))
    }
}

unsafe impl<F: FnMut(char) -> bool> ReverseConsumer<str> for MultiCharSearcher<F> {
    #[inline]
    fn rconsume(&mut self, hay: Span<&str>) -> Option<usize> {
        let (hay, range) = hay.into_parts();
        let end = range.end;
        if range.start == end {
            return None;
        }
        let c = unsafe { hay.get_unchecked(..end) }.chars().next_back().unwrap();
        if (self.predicate)(c) {
            Some(end - c.len_utf8())
        } else {
            None
        }
    }

    #[inline]
    fn trim_end(&mut self, hay: &str) -> usize {
        // `find.map_or` is faster in trim_end in the microbenchmark, while
        // `find.unwrap_or` is faster in trim_start. Don't ask me why.
        let mut chars = hay.chars();
        let unconsume_amount = chars
            .by_ref()
            .rev() // btw, `rev().find()` is faster than `rfind()`
            .find(|c| !(self.predicate)(*c))
            .map_or(0, |c| c.len_utf8());
        chars.as_str().len() + unconsume_amount
    }
}

unsafe impl<F: FnMut(char) -> bool> DoubleEndedSearcher<str> for MultiCharSearcher<F> {}
unsafe impl<F: FnMut(char) -> bool> DoubleEndedConsumer<str> for MultiCharSearcher<F> {}

macro_rules! impl_pattern {
    ($ty:ty) => {
        impl<'h, F: FnMut(char) -> bool> Pattern<$ty> for F {
            type Searcher = MultiCharSearcher<F>;
            type Consumer = MultiCharSearcher<F>;

            #[inline]
            fn into_searcher(self) -> Self::Searcher {
                MultiCharSearcher { predicate: self }
            }

            #[inline]
            fn into_consumer(self) -> Self::Consumer {
                MultiCharSearcher { predicate: self }
            }
        }

        impl<'h, 'p> Pattern<$ty> for &'p [char] {
            type Searcher = MultiCharSearcher<MultiCharEq<'p>>;
            type Consumer = MultiCharSearcher<MultiCharEq<'p>>;

            #[inline]
            fn into_searcher(self) -> Self::Searcher {
                MultiCharSearcher { predicate: MultiCharEq(self) }
            }

            #[inline]
            fn into_consumer(self) -> Self::Consumer {
                MultiCharSearcher { predicate: MultiCharEq(self) }
            }
        }
    }
}

impl_pattern!(&'h str);
impl_pattern!(&'h mut str);
