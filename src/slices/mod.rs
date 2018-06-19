use haystack::{Haystack, IndexHaystack};
use std::ops::{Deref, Range};
use std::slice::{from_raw_parts, from_raw_parts_mut};
use std::mem::replace;

impl<'h, T: 'h> Haystack for &'h [T] {
    #[inline]
    fn is_empty(&self) -> bool {
        (**self).is_empty()
    }

    #[inline]
    fn collapse_to_end(&mut self) -> Self {
        let (left, right) = self.split_at(self.len());
        *self = right;
        left
    }

    #[inline]
    fn collapse_to_start(&mut self) -> Self {
        let (left, right) = self.split_at(0);
        *self = left;
        right
    }

    // #[inline]
    // fn empty_at_start(&self) -> Self {
    //     unsafe {
    //         self.get_unchecked(..0)
    //     }
    // }

    // #[inline]
    // fn empty_at_end(&self) -> Self {
    //     unsafe {
    //         self.get_unchecked(self.len()..)
    //     }
    // }

    // #[inline]
    // fn consume_first(&mut self) -> Option<Self> {
    //     if self.is_empty() {
    //         None
    //     } else {
    //         let (left, right) = self.split_at(1);
    //         *self = right;
    //         Some(left)
    //     }
    // }

    // #[inline]
    // fn consume_last(&mut self) -> Option<Self> {
    //     if self.is_empty() {
    //         None
    //     } else {
    //         let (left, right) = self.split_at(self.len() - 1);
    //         *self = left;
    //         Some(right)
    //     }
    // }
}

impl<'h, T: 'h> IndexHaystack for &'h [T] {
    type Origin = *const T;
    type Index = usize;

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
    #[inline]
    fn is_empty(&self) -> bool {
        (**self).is_empty()
    }

    #[inline]
    fn collapse_to_end(&mut self) -> Self {
        let s = replace(self, Default::default());
        let len = s.len();
        let (left, right) = s.split_at_mut(len);
        *self = right;
        left
    }

    #[inline]
    fn collapse_to_start(&mut self) -> Self {
        let s = replace(self, Default::default());
        let (left, right) = s.split_at_mut(0);
        *self = left;
        right
    }

    // #[inline]
    // fn empty_at_start(&self) -> Self {
    //     unsafe {
    //         from_raw_parts_mut(self.as_mut_ptr(), 0)
    //     }
    // }

    // #[inline]
    // fn empty_at_end(&self) -> Self {
    //     unsafe {
    //         from_raw_parts_mut(self.as_mut_ptr().add(self.len()), 0)
    //     }
    // }

    // #[inline]
    // fn consume_first(&mut self) -> Option<Self> {
    //     if self.is_empty() {
    //         None
    //     } else {
    //         let s = replace(self, Default::default());
    //         let (left, right) = s.split_at_mut(1);
    //         *self = right;
    //         Some(left)
    //     }
    // }

    // #[inline]
    // fn consume_last(&mut self) -> Option<Self> {
    //     if self.is_empty() {
    //         None
    //     } else {
    //         let s = replace(self, Default::default());
    //         let len = s.len();
    //         let (left, right) = s.split_at_mut(len - 1);
    //         *self = left;
    //         Some(right)
    //     }
    // }
}

impl<'h, T: 'h> IndexHaystack for &'h mut [T] {
    type Origin = *const T;
    type Index = usize;

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
    #[inline]
    fn is_empty(&self) -> bool {
        (**self).is_empty()
    }

    #[inline]
    fn collapse_to_end(&mut self) -> Self {
        replace(self, Vec::new())
    }

    #[inline]
    fn collapse_to_start(&mut self) -> Self {
        replace(self, Vec::new())
    }

    // #[inline]
    // fn empty_at_start(&self) -> Self {
    //     Vec::new()
    // }

    // #[inline]
    // fn empty_at_end(&self) -> Self {
    //     Vec::new()
    // }

    // #[inline]
    // fn consume_first(&mut self) -> Option<Self> {
    //     if self.is_empty() {
    //         None
    //     } else {
    //         let first = self.remove(0);
    //         Some(vec![first])
    //     }
    // }

    // #[inline]
    // fn consume_last(&mut self) -> Option<Self> {
    //     self.pop().map(|v| vec![v])
    // }
}



trait SliceLike: Deref<Target = [<Self as SliceLike>::Item]> + Sized {
    type Item;

    unsafe fn split_around_unchecked(self, range: Range<usize>) -> (Self, Self, Self);

    unsafe fn slice_from_unchecked(self, index: usize) -> Self;

    unsafe fn slice_to_unchecked(self, index: usize) -> Self;
}

impl<'h, T: 'h> SliceLike for &'h [T] {
    type Item = T;

    #[inline]
    unsafe fn split_around_unchecked(self, range: Range<usize>) -> (Self, Self, Self) {
        let start = self.as_ptr();
        let c1 = start.add(range.start);
        let c2 = start.add(range.end);
        (
            from_raw_parts(start, range.start),
            from_raw_parts(c1, range.end - range.start),
            from_raw_parts(c2, self.len() - range.end),
        )
    }

    #[inline]
    unsafe fn slice_from_unchecked(self, index: usize) -> Self {
        self.get_unchecked(index..)
    }

    #[inline]
    unsafe fn slice_to_unchecked(self, index: usize) -> Self {
        self.get_unchecked(..index)
    }
}

impl<'h, T: 'h> SliceLike for &'h mut [T] {
    type Item = T;

    #[inline]
    unsafe fn split_around_unchecked(self, range: Range<usize>) -> (Self, Self, Self) {
        let start = self.as_mut_ptr();
        let c1 = start.add(range.start);
        let c2 = start.add(range.end);
        (
            from_raw_parts_mut(start, range.start),
            from_raw_parts_mut(c1, range.end - range.start),
            from_raw_parts_mut(c2, self.len() - range.end),
        )
    }

    #[inline]
    unsafe fn slice_from_unchecked(self, index: usize) -> Self {
        self.get_unchecked_mut(index..)
    }

    #[inline]
    unsafe fn slice_to_unchecked(self, index: usize) -> Self {
        self.get_unchecked_mut(..index)
    }
}

impl<T> SliceLike for Vec<T> {
    type Item = T;

    #[inline]
    unsafe fn split_around_unchecked(mut self, range: Range<usize>) -> (Self, Self, Self) {
        let right = self.split_off(range.end);
        let middle = self.split_off(range.start);
        (self, middle, right)
    }

    #[inline]
    unsafe fn slice_from_unchecked(mut self, index: usize) -> Self {
        self.drain(..index).for_each(drop);
        self
    }

    #[inline]
    unsafe fn slice_to_unchecked(mut self, index: usize) -> Self {
        self.truncate(index);
        self
    }
}

mod func;
// mod slice;
