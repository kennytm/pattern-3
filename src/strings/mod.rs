use haystack::{Hay, Haystack};
use std::ops::Range;

impl Hay for str {
    type Index = usize;

    #[inline]
    fn empty<'a>() -> &'a Self {
        ""
    }

    #[inline]
    fn add_len(&self, index: usize) -> usize {
        self.len() + index
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
    fn validate_range(&self, range: Range<usize>) {
        debug_assert!(range.start <= range.end);
        debug_assert!(range.end <= self.len());
        debug_assert!(self.is_char_boundary(range.start));
        debug_assert!(self.is_char_boundary(range.end));
    }

    #[inline]
    unsafe fn slice_unchecked(&self, range: Range<usize>) -> &Self {
        self.get_unchecked(range)
    }
}

impl<'h> Haystack for &'h mut str {
    type Hay = str;

    #[inline]
    fn empty() -> &'h mut str {
        Self::default()
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

mod char;
mod func;
mod str;
