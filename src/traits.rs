use span::Span;
use std::ops::{Range, Try};

pub trait Haystack {
    /// The item type of the haystack, meaning `&Self` can be treated as the
    /// same as the slice `&[Item]` in terms of memory access.
    type Item;

    /// Returns the two pointers that spans the whole haystack.
    fn ptr_range(&self) -> Range<*const Self::Item>;

    /// Obtains a haystack from the start and end pointers.
    unsafe fn from_ptr_range<'a>(range: Range<*const Self::Item>) -> &'a Self;

    ///
    unsafe fn start_ptr_to_offset(
        haystack_range: Range<*const Self::Item>,
        cur: *const Self::Item,
    ) -> usize;

    unsafe fn end_ptr_to_offset(
        haystack_range: Range<*const Self::Item>,
        cur: *const Self::Item,
    ) -> usize;

    unsafe fn start_to_end_ptr(
        haystack_range: Range<*const Self::Item>,
        cur: *const Self::Item,
    ) -> *const Self::Item;

    unsafe fn end_to_start_ptr(
        haystack_range: Range<*const Self::Item>,
        cur: *const Self::Item,
    ) -> *const Self::Item;
}

pub trait HaystackMut: Haystack {
    unsafe fn from_ptr_range_mut<'a>(range: Range<*mut Self::Item>) -> &'a mut Self;
}

pub unsafe trait Searcher<'a, H>: Iterator<Item = Span<'a, H>>
where
    H: Haystack + ?Sized + 'a,
{
    /// Returns the span of the next match if it appears immediately after the
    /// previous search result.
    ///
    /// If there is no matches immediately after the previous search result,
    /// this method returns `None` and the iterator should not advance.
    ///
    /// When this method returns `Some(x)`, it also means calling `.next()`
    /// instead will also return the same `Some(x)`.
    fn next_immediate(&mut self) -> Option<Span<'a, H>>;
}


pub trait Pattern<'a, H>: Sized
where
    H: Haystack + ?Sized + 'a,
{
    type Searcher: Searcher<'a, H>;

    fn into_searcher(self, haystack: &'a H) -> Self::Searcher;
}

/// Wrapper type to obtain a reversed searcher.
#[fundamental]
pub struct Rev<S> {
    pub inner: S,
}

impl<S> Rev<S> {
    pub fn new(inner: S) -> Self {
        Rev { inner }
    }
}

impl<S: DoubleEndedIterator> Iterator for Rev<S> {
    type Item = S::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    fn try_fold<B, F, R>(&mut self, init: B, f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> R,
        R: Try<Ok = B>,
    {
        self.inner.try_rfold(init, f)
    }

    fn fold<Acc, F>(self, init: Acc, f: F) -> Acc
    where
        F: FnMut(Acc, Self::Item) -> Acc,
    {
        self.inner.rfold(init, f)
    }

    #[inline]
    fn find<P>(&mut self, predicate: P) -> Option<Self::Item>
    where
        P: FnMut(&Self::Item) -> bool,
    {
        self.inner.rfind(predicate)
    }

    #[inline]
    fn rposition<P>(&mut self, predicate: P) -> Option<usize>
    where
        P: FnMut(Self::Item) -> bool,
    {
        self.inner.position(predicate)
    }
}
