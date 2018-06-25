use haystack::{Hay, Haystack};
use std::ops::Range;

impl<T> Hay for [T] {
    type Index = usize;

    #[inline]
    fn empty<'a>() -> &'a Self {
        &[]
    }

    #[inline]
    fn add_len(&self, index: usize) -> usize {
        index + self.len()
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
    fn validate_range(&self, range: Range<usize>) {
        debug_assert!(range.start <= range.end);
        debug_assert!(range.end <= self.len());
    }
}

impl<'h, T: 'h> Haystack for &'h mut [T] {
    type Hay = [T];

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
}

impl<T> Haystack for Vec<T> {
    type Hay = [T];

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
}

mod func;
pub(crate) mod slice;
