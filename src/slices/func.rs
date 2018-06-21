use pattern::{Pattern, Searcher, ReverseSearcher, DoubleEndedSearcher};
use std::marker::PhantomData;
use std::ops::Range;

pub struct ElemSearcher<T, F> {
    predicate: F,
    _marker: PhantomData<T>,
}

impl<T, F> Pattern<[T]> for F
where
    F: FnMut(&T) -> bool,
{
    type Searcher = ElemSearcher<T, F>;

    #[inline]
    fn into_searcher(self) -> Self::Searcher {
        ElemSearcher {
            predicate: self,
            _marker: PhantomData,
        }
    }

    #[inline]
    fn is_prefix_of(mut self, haystack: &[T]) -> bool {
        if let Some(x) = haystack.first() {
            self(x)
        } else {
            false
        }
    }

    #[inline]
    fn trim_start(&mut self, haystack: &[T]) -> usize {
        let len = haystack.len();
        let mut it = haystack.iter();
        if it.find(|x| !self(x)).is_some() {
            len - it.as_slice().len() - 1
        } else {
            len
        }
    }

    #[inline]
    fn is_suffix_of(mut self, haystack: &[T]) -> bool {
        if let Some(x) = haystack.last() {
            self(x)
        } else {
            false
        }
    }

    #[inline]
    fn trim_end(&mut self, haystack: &[T]) -> usize {
        haystack.iter().rposition(|x| !self(x)).map_or(0, |p| p + 1)
    }
}

unsafe impl<T, F> Searcher for ElemSearcher<T, F>
where
    F: FnMut(&T) -> bool,
{
    type Hay = [T];

    #[inline]
    fn search(&mut self, rest: &[T]) -> Option<Range<usize>> {
        let pos = rest.iter().position(&mut self.predicate)?;
        Some(pos..(pos + 1))
    }
}

unsafe impl<T, F> ReverseSearcher for ElemSearcher<T, F>
where
    F: FnMut(&T) -> bool,
{
    #[inline]
    fn rsearch(&mut self, rest: &[T]) -> Option<Range<usize>> {
        let pos = rest.iter().rposition(&mut self.predicate)?;
        Some(pos..(pos + 1))
    }
}

unsafe impl<T, F> DoubleEndedSearcher for ElemSearcher<T, F>
where
    F: FnMut(&T) -> bool,
{}