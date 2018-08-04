use haystack::{Hay, Haystack};
use std::ops::Range;

unsafe impl<T> Hay for [T] {
    type Index = usize;

    #[inline]
    fn empty<'a>() -> &'a Self {
        &[]
    }

    #[inline]
    fn start_index(&self) -> usize {
        0
    }

    #[inline]
    fn end_index(&self) -> usize {
        self.len()
    }

    #[inline]
    unsafe fn slice_unchecked(&self, range: Range<usize>) -> &Self {
        self.get_unchecked(range)
    }

    #[inline]
    unsafe fn next_index(&self, index: Self::Index) -> Self::Index {
        index + 1
    }

    #[inline]
    unsafe fn prev_index(&self, index: Self::Index) -> Self::Index {
        index - 1
    }
}

unsafe impl<'h, T: 'h> Haystack for &'h mut [T] {
    #[inline]
    fn empty() -> Self {
        &mut []
    }

    #[inline]
    unsafe fn slice_unchecked(self, range: Range<usize>) -> Self {
        self.get_unchecked_mut(range)
    }

    #[inline]
    unsafe fn split_around(self, range: Range<usize>) -> [Self; 3] {
        let (haystack, right) = self.split_at_mut(range.end);
        let (left, middle) = haystack.split_at_mut(range.start);
        [left, middle, right]
    }

    #[inline]
    fn restore_range(&self, range: Range<usize>, subrange: Range<usize>) -> Range<usize> {
        (subrange.start + range.start)..(subrange.end + range.start)
    }
}

#[cfg(feature = "std")]
unsafe impl<T> Haystack for Vec<T> {
    #[inline]
    fn empty() -> Self {
        Vec::new()
    }

    #[inline]
    unsafe fn slice_unchecked(mut self, range: Range<usize>) -> Self {
        self.truncate(range.end);
        self.drain(..range.start);
        self
    }

    #[inline]
    unsafe fn split_around(mut self, range: Range<usize>) -> [Self; 3] {
        let right = self.split_off(range.end);
        let middle = self.split_off(range.start);
        [self, middle, right]
    }

    #[inline]
    fn restore_range(&self, range: Range<usize>, subrange: Range<usize>) -> Range<usize> {
        (subrange.start + range.start)..(subrange.end + range.start)
    }
}

mod func;
pub(crate) mod slice;
