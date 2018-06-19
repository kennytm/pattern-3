use haystack::{Haystack, IndexHaystack};
use std::slice::{from_raw_parts, from_raw_parts_mut};
use std::str::{from_utf8_unchecked, from_utf8_unchecked_mut};
use std::ops::{Deref, Range};
use std::mem::replace;

impl<'h> Haystack for &'h str {
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

impl<'h> IndexHaystack for &'h str {
    type Origin = *const u8;
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

impl<'h> Haystack for &'h mut str {
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
        let len = s.len();
        let (left, right) = s.split_at_mut(0);
        *self = left;
        right
    }
}

impl<'h> IndexHaystack for &'h mut str {
    type Origin = *const u8;
    type Index = usize;

    fn origin(&self) -> Self::Origin {
        self.as_ptr()
    }

    unsafe fn range_from_origin(&self, origin: Self::Origin) -> Range<usize> {
        let start = self.as_ptr().offset_from(origin) as usize;
        let end = start + self.len();
        start..end
    }
}

pub trait StrLike: Deref<Target = str> + Sized {
    unsafe fn split_around_unchecked(self, range: Range<usize>) -> (Self, Self, Self);
    unsafe fn split_around_ptr_unchecked(self, range: Range<*const u8>) -> (Self, Self, Self);
    unsafe fn slice_from_ptr_unchecked(self, ptr: *const u8) -> Self;
    unsafe fn slice_to_unchecked(self, index: usize) -> Self;
}

impl<'h> StrLike for &'h str {
    #[inline]
    unsafe fn split_around_unchecked(self, range: Range<usize>) -> (Self, Self, Self) {
        let start = self.as_ptr();
        let c1 = start.add(range.start);
        let c2 = start.add(range.end);
        (
            from_utf8_unchecked(from_raw_parts(start, range.start)),
            from_utf8_unchecked(from_raw_parts(c1, range.end - range.start)),
            from_utf8_unchecked(from_raw_parts(c2, self.len() - range.end)),
        )
    }

    #[inline]
    unsafe fn split_around_ptr_unchecked(self, range: Range<*const u8>) -> (Self, Self, Self) {
        let start = self.as_ptr();
        let start_len = range.start.offset_from(start) as usize;
        let middle_len = range.end.offset_from(range.start) as usize;
        let end_len = self.len() - start_len - middle_len;
        (
            from_utf8_unchecked(from_raw_parts(start, start_len)),
            from_utf8_unchecked(from_raw_parts(range.start, middle_len)),
            from_utf8_unchecked(from_raw_parts(range.end, end_len)),
        )
    }

    #[inline]
    unsafe fn slice_from_ptr_unchecked(self, ptr: *const u8) -> Self {
        let len = self.len() - (ptr.offset_from(self.as_ptr()) as usize);
        from_utf8_unchecked(from_raw_parts(ptr, len))
    }

    #[inline]
    unsafe fn slice_to_unchecked(self, index: usize) -> Self {
        self.get_unchecked(..index)
    }
}

impl<'h> StrLike for &'h mut str {
    #[inline]
    unsafe fn split_around_unchecked(self, range: Range<usize>) -> (Self, Self, Self) {
        let start = self.as_bytes_mut().as_mut_ptr();
        let c1 = start.add(range.start);
        let c2 = start.add(range.end);
        (
            from_utf8_unchecked_mut(from_raw_parts_mut(start, range.start)),
            from_utf8_unchecked_mut(from_raw_parts_mut(c1, range.end - range.start)),
            from_utf8_unchecked_mut(from_raw_parts_mut(c2, self.len() - range.end)),
        )
    }

    #[inline]
    unsafe fn split_around_ptr_unchecked(self, range: Range<*const u8>) -> (Self, Self, Self) {
        let start = self.as_ptr();
        let start_len = range.start.offset_from(start) as usize;
        let middle_len = range.end.offset_from(range.start) as usize;
        let end_len = self.len() - start_len - middle_len;
        (
            from_utf8_unchecked_mut(from_raw_parts_mut(start as *mut u8, start_len)),
            from_utf8_unchecked_mut(from_raw_parts_mut(range.start as *mut u8, middle_len)),
            from_utf8_unchecked_mut(from_raw_parts_mut(range.end as *mut u8, end_len)),
        )
    }

    #[inline]
    unsafe fn slice_from_ptr_unchecked(self, ptr: *const u8) -> Self {
        let len = self.len() - (ptr.offset_from(self.as_ptr()) as usize);
        from_utf8_unchecked_mut(from_raw_parts_mut(ptr as *mut u8, len))
    }

    #[inline]
    unsafe fn slice_to_unchecked(self, index: usize) -> Self {
        self.get_unchecked_mut(..index)
    }
}

mod char;
mod func;
