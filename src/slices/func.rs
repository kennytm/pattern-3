use pattern::*;
use haystack::Span;
use std::ops::Range;

pub struct ElemSearcher<F> {
    predicate: F,
}

macro_rules! impl_pattern {
    (<[$($gen:tt)*]> $ty:ty) => {
        impl<$($gen)*> Pattern<$ty> for F
        where
            F: FnMut(&T) -> bool,
        {
            type Searcher = ElemSearcher<F>;

            #[inline]
            fn into_searcher(self) -> Self::Searcher {
                ElemSearcher {
                    predicate: self,
                }
            }
        }
    }
}

impl_pattern!(<['h, T, F]> &'h [T]);
impl_pattern!(<['h, T, F]> &'h mut [T]);
#[cfg(feature = "std")]
impl_pattern!(<[T, F]> Vec<T>);

unsafe impl<T, F> Searcher<[T]> for ElemSearcher<F>
where
    F: FnMut(&T) -> bool,
{
    #[inline]
    fn search(&mut self, span: Span<&[T]>) -> Option<Range<usize>> {
        let (rest, range) = span.into_parts();
        let start = range.start;
        let pos = rest[range].iter().position(&mut self.predicate)?;
        Some((pos + start)..(pos + start + 1))
    }

    #[inline]
    fn consume(&mut self, span: Span<&[T]>) -> Option<usize> {
        let (hay, range) = span.into_parts();
        if range.end == range.start {
            return None;
        }
        let x = unsafe { hay.get_unchecked(range.start) };
        if (self.predicate)(x) {
            Some(range.start + 1)
        } else {
            None
        }
    }

    #[inline]
    fn trim_start(&mut self, hay: &[T]) -> usize {
        let mut it = hay.iter();
        let len = hay.len();
        if it.find(|x| !(self.predicate)(x)).is_some() {
            len - it.as_slice().len() - 1
        } else {
            len
        }
    }
}

unsafe impl<T, F> ReverseSearcher<[T]> for ElemSearcher<F>
where
    F: FnMut(&T) -> bool,
{
    #[inline]
    fn rsearch(&mut self, span: Span<&[T]>) -> Option<Range<usize>> {
        let (rest, range) = span.into_parts();
        let start = range.start;
        let pos = rest[range].iter().rposition(&mut self.predicate)?;
        Some((pos + start)..(pos + start + 1))
    }

    #[inline]
    fn rconsume(&mut self, span: Span<&[T]>) -> Option<usize> {
        let (hay, range) = span.into_parts();
        if range.start == range.end {
            return None;
        }
        let last = range.end - 1;
        let x = unsafe { hay.get_unchecked(last) };
        if (self.predicate)(x) {
            Some(last)
        } else {
            None
        }
    }

    #[inline]
    fn trim_end(&mut self, hay: &[T]) -> usize {
        hay.iter().rposition(|x| !(self.predicate)(x)).map_or(0, |p| p + 1)
    }
}

unsafe impl<T, F> DoubleEndedSearcher<[T]> for ElemSearcher<F>
where
    F: FnMut(&T) -> bool,
{}
