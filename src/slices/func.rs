use pattern::*;
use haystack::SharedSpan;
use std::marker::PhantomData;
use std::ops::Range;

pub struct ElemSearcher<T, F> {
    predicate: F,
    _marker: PhantomData<T>,
}

macro_rules! impl_pattern {
    (<[$($gen:tt)*]> $ty:ty) => {
        impl<$($gen)*> Pattern<$ty> for F
        where
            F: FnMut(&T) -> bool,
        {
            type Searcher = ElemSearcher<T, F>;
            type Checker = ElemSearcher<T, F>;

            #[inline]
            fn into_searcher(self) -> Self::Searcher {
                ElemSearcher {
                    predicate: self,
                    _marker: PhantomData,
                }
            }

            #[inline]
            fn into_checker(self) -> Self::Checker {
                ElemSearcher {
                    predicate: self,
                    _marker: PhantomData,
                }
            }
        }
    }
}

impl_pattern!(<['h, T, F]> &'h [T]);
impl_pattern!(<['h, T, F]> &'h mut [T]);
impl_pattern!(<[T, F]> Vec<T>);

unsafe impl<T, F> Searcher for ElemSearcher<T, F>
where
    F: FnMut(&T) -> bool,
{
    type Hay = [T];

    #[inline]
    fn search(&mut self, span: SharedSpan<'_, [T]>) -> Option<Range<usize>> {
        let (rest, range) = span.into_parts();
        let start = range.start;
        let pos = rest[range].iter().position(&mut self.predicate)?;
        Some((pos + start)..(pos + start + 1))
    }
}

unsafe impl<T, F> Checker for ElemSearcher<T, F>
where
    F: FnMut(&T) -> bool,
{
    type Hay = [T];

    #[inline]
    fn is_prefix_of(mut self, hay: &[T]) -> bool {
        if let Some(x) = hay.first() {
            (self.predicate)(x)
        } else {
            false
        }
    }

    #[inline]
    fn trim_start(&mut self, hay: &[T]) -> usize {
        let len = hay.len();
        let mut it = hay.iter();
        if it.find(|x| !(self.predicate)(x)).is_some() {
            len - it.as_slice().len() - 1
        } else {
            len
        }
    }
}

unsafe impl<T, F> ReverseSearcher for ElemSearcher<T, F>
where
    F: FnMut(&T) -> bool,
{
    #[inline]
    fn rsearch(&mut self, span: SharedSpan<'_, [T]>) -> Option<Range<usize>> {
        let (rest, range) = span.into_parts();
        let start = range.start;
        let pos = rest[range].iter().rposition(&mut self.predicate)?;
        Some((pos + start)..(pos + start + 1))
    }
}

unsafe impl<T, F> ReverseChecker for ElemSearcher<T, F>
where
    F: FnMut(&T) -> bool,
{
    #[inline]
    fn is_suffix_of(mut self, hay: &[T]) -> bool {
        if let Some(x) = hay.last() {
            (self.predicate)(x)
        } else {
            false
        }
    }

    #[inline]
    fn trim_end(&mut self, hay: &[T]) -> usize {
        hay.iter().rposition(|x| !(self.predicate)(x)).map_or(0, |p| p + 1)
    }
}

unsafe impl<T, F> DoubleEndedSearcher for ElemSearcher<T, F>
where
    F: FnMut(&T) -> bool,
{}

unsafe impl<T, F> DoubleEndedChecker for ElemSearcher<T, F>
where
    F: FnMut(&T) -> bool,
{}
