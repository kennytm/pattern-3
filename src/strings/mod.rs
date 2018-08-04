use haystack::{Hay, Haystack};
use std::ops::Range;

unsafe impl Hay for str {
    type Index = usize;

    #[inline]
    fn empty<'a>() -> &'a Self {
        ""
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
        index + self.get_unchecked(index..).chars().next().unwrap().len_utf8()
    }

    #[inline]
    unsafe fn prev_index(&self, index: Self::Index) -> Self::Index {
        index - self.get_unchecked(..index).chars().next_back().unwrap().len_utf8()
    }
}

unsafe impl<'h> Haystack for &'h mut str {
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

    #[inline]
    fn restore_range(&self, range: Range<usize>, subrange: Range<usize>) -> Range<usize> {
        (subrange.start + range.start)..(subrange.end + range.start)
    }
}

mod char;
mod func;
mod str;
