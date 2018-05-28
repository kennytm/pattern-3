use pattern::{Pattern, ReversePattern};
use cursor::Cursor;
use span::Span;
use std::iter::{FusedIterator, Rev};
use std::mem::size_of;

unsafe impl<'h, F, T> Pattern<'h, [T]> for F
where
    T: 'h,
    F: FnMut(&T) -> bool,
{
    type Searcher = ElemSearcher<'h, T, F>;

    #[inline]
    fn into_searcher(self, span: Span<'h, [T]>) -> Self::Searcher {
        ElemSearcher {
            remaining: span,
            predicate: self,
        }
    }

    #[inline]
    fn is_prefix_of(mut self, span: Span<'_, [T]>) -> bool {
        if let Some(c) = span.as_slice().first() {
            self(c)
        } else {
            false
        }
    }

    #[inline]
    fn trim_start(&mut self, span: &mut Span<'_, [T]>) {
        let cloned = unsafe { span.clone() };
        if let Some(first_unmatch) = (|c| !self(c)).into_searcher(cloned).next() {
            span.remove_start(first_unmatch.start());
        } else {
            span.collapse_to_end();
        }
    }
}

unsafe impl<'h, F, T> ReversePattern<'h, [T]> for F
where
    T: 'h,
    F: FnMut(&T) -> bool,
{
    type ReverseSearcher = Rev<Self::Searcher>;

    #[inline]
    fn into_reverse_searcher(self, span: Span<'h, [T]>) -> Self::ReverseSearcher {
        self.into_searcher(span).rev()
    }

    #[inline]
    fn is_suffix_of(mut self, span: Span<'_, [T]>) -> bool {
        if let Some(c) = span.as_slice().last() {
            self(c)
        } else {
            false
        }
    }

    #[inline]
    fn trim_end(&mut self, span: &mut Span<'_, [T]>) {
        let cloned = unsafe { span.clone() };
        if let Some(last_unmatch) = (|c| !self(c)).into_searcher(cloned).next_back() {
            span.remove_end(last_unmatch.end());
        } else {
            span.collapse_to_start();
        }
    }
}

pub struct ElemSearcher<'h, T, F> {
    remaining: Span<'h, [T]>,
    predicate: F,
}

impl<'h, T, F> ElemSearcher<'h, T, F>
where
    T: 'h,
    F: FnMut(&T) -> bool,
{
    #[inline]
    fn find_next(&mut self) -> Option<(*const T, *const T)> {
        let mut it = self.remaining.as_slice().iter();
        if size_of::<T>() != 0 {
            let start = it.find(|c| (self.predicate)(*c))? as *const T;
            let end = unsafe { start.add(1) };
            Some((start, end))
        } else {
            let base = self.remaining.start().raw() as usize;
            let start = base + it.position(|c| (self.predicate)(c))?;
            let end = start + 1;
            Some((start as *const T, end as *const T))
        }
    }

    #[inline]
    fn find_next_back(&mut self) -> Option<(*const T, *const T)> {
        let mut it = self.remaining.as_slice().iter();
        if size_of::<T>() != 0 {
            let start = it.rfind(|c| (self.predicate)(*c))? as *const T;
            let end = unsafe { start.add(1) };
            Some((start, end))
        } else {
            let base = self.remaining.start().raw() as usize;
            let start = base + it.rposition(|c| (self.predicate)(c))?;
            let end = start + 1;
            Some((start as *const T, end as *const T))
        }
    }
}

impl<'h, T, F> Iterator for ElemSearcher<'h, T, F>
where
    T: 'h,
    F: FnMut(&T) -> bool,
{
    type Item = Span<'h, [T]>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((start, end)) = self.find_next() {
            unsafe {
                Some(self.remaining.take_from_start_unchecked(start, end))
            }
        } else {
            self.remaining.collapse_to_end();
            None
        }
    }
}

impl<'h, T, F> FusedIterator for ElemSearcher<'h, T, F>
where
    T: 'h,
    F: FnMut(&T) -> bool,
{}

impl<'h, T, F> DoubleEndedIterator for ElemSearcher<'h, T, F>
where
    T: 'h,
    F: FnMut(&T) -> bool,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some((start, end)) = self.find_next_back() {
            unsafe {
                Some(self.remaining.take_from_end_unchecked(start, end))
            }
        } else {
            self.remaining.collapse_to_start();
            None
        }
    }
}
