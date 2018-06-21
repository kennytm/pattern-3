use haystack::{Hay, Haystack, IndexHaystack};
use std::ops::Range;
use std::slice::{from_raw_parts, from_raw_parts_mut};

impl<T> Hay for [T] {
    type Index = usize;

    #[inline]
    fn is_empty(&self) -> bool {
        <[T]>::is_empty(self)
    }
}

impl<'h, T: 'h> Haystack for &'h [T] {
    type Hay = [T];

    #[inline]
    unsafe fn split_around_unchecked(self, range: Range<usize>) -> (Self, Self, Self) {
        let st = self.as_ptr();
        let c1 = st.add(range.start);
        let c2 = st.add(range.end);
        (
            from_raw_parts(st, range.start),
            from_raw_parts(c1, range.end - range.start),
            from_raw_parts(c2, self.len() - range.end),
        )
    }

    #[inline]
    unsafe fn trim_start_unchecked(self, start: usize) -> Self {
        self.get_unchecked(start..)
    }

    #[inline]
    unsafe fn trim_end_unchecked(self, end: usize) -> Self {
        self.get_unchecked(..end)
    }
}

impl<'h, T: 'h> IndexHaystack for &'h [T] {
    type Origin = *const T;

    #[inline]
    fn origin(&self) -> Self::Origin {
        self.as_ptr()
    }

    #[inline]
    unsafe fn range_from_origin(&self, origin: Self::Origin) -> Range<usize> {
        let start = self.as_ptr().offset_from(origin) as usize;
        let end = start + self.len();
        start..end
    }
}

impl<'h, T: 'h> Haystack for &'h mut [T] {
    type Hay = [T];

    #[inline]
    unsafe fn split_around_unchecked(self, range: Range<usize>) -> (Self, Self, Self) {
        let st = self.as_mut_ptr();
        let c1 = st.add(range.start);
        let c2 = st.add(range.end);
        (
            from_raw_parts_mut(st, range.start),
            from_raw_parts_mut(c1, range.end - range.start),
            from_raw_parts_mut(c2, self.len() - range.end),
        )
    }

    #[inline]
    unsafe fn trim_start_unchecked(self, start: usize) -> Self {
        self.get_unchecked_mut(start..)
    }

    #[inline]
    unsafe fn trim_end_unchecked(self, end: usize) -> Self {
        self.get_unchecked_mut(..end)
    }
}

impl<'h, T: 'h> IndexHaystack for &'h mut [T] {
    type Origin = *const T;

    #[inline]
    fn origin(&self) -> Self::Origin {
        self.as_ptr()
    }

    #[inline]
    unsafe fn range_from_origin(&self, origin: Self::Origin) -> Range<usize> {
        let start = self.as_ptr().offset_from(origin) as usize;
        let end = start + self.len();
        start..end
    }
}

impl<T> Haystack for Vec<T> {
    type Hay = [T];

    #[inline]
    unsafe fn split_around_unchecked(mut self, range: Range<usize>) -> (Self, Self, Self) {
        let right = self.split_off(range.end);
        let middle = self.split_off(range.start);
        (self, middle, right)
    }


    #[inline]
    unsafe fn trim_start_unchecked(mut self, start: usize) -> Self {
        self.drain(..start).for_each(drop);
        self
    }

    #[inline]
    unsafe fn trim_end_unchecked(mut self, end: usize) -> Self {
        self.truncate(end);
        self
    }
}

mod func;
pub(crate) mod slice;
