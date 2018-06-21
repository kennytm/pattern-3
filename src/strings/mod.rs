use haystack::{Hay, Haystack, IndexHaystack};
use std::slice::{from_raw_parts, from_raw_parts_mut};
use std::str::{from_utf8_unchecked, from_utf8_unchecked_mut};
use std::ops::Range;

impl Hay for str {
    type Index = usize;

    fn is_empty(&self) -> bool {
        <str>::is_empty(self)
    }
}

impl<'h> Haystack for &'h str {
    type Hay = str;

    #[inline]
    unsafe fn split_around_unchecked(self, range: Range<usize>) -> (Self, Self, Self) {
        // let st = self.as_ptr();
        // let start_len = range.start.offset_from(st) as usize;
        // let middle_len = range.end.offset_from(range.start) as usize;
        // let end_len = self.len() - start_len - middle_len;
        // (
        //     from_utf8_unchecked(from_raw_parts(st, start_len)),
        //     from_utf8_unchecked(from_raw_parts(range.start, middle_len)),
        //     from_utf8_unchecked(from_raw_parts(range.end, end_len)),
        // )
        let st = self.as_ptr();
        let c1 = st.add(range.start);
        let c2 = st.add(range.end);
        (
            from_utf8_unchecked(from_raw_parts(st, range.start)),
            from_utf8_unchecked(from_raw_parts(c1, range.end - range.start)),
            from_utf8_unchecked(from_raw_parts(c2, self.len() - range.end)),
        )
    }

    #[inline]
    unsafe fn trim_start_unchecked(self, start: usize) -> Self {
        // let len = self.len() - (ptr.offset_from(self.as_ptr()) as usize);
        // from_utf8_unchecked(from_raw_parts(ptr, len))
        self.get_unchecked(start..)
    }

    #[inline]
    unsafe fn trim_end_unchecked(self, end: usize) -> Self {
        // let len = ptr.offset_from(self.as_ptr()) as usize;
        // from_utf8_unchecked(from_raw_parts(self.as_ptr(), len))
        self.get_unchecked(..end)
    }
}

impl<'h> IndexHaystack for &'h str {
    type Origin = *const u8;

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
    type Hay = str;

    #[inline]
    unsafe fn split_around_unchecked(self, range: Range<usize>) -> (Self, Self, Self) {
        // let st = self.as_ptr();
        // let start_len = range.start.offset_from(st) as usize;
        // let middle_len = range.end.offset_from(range.start) as usize;
        // let end_len = self.len() - start_len - middle_len;
        // (
        //     from_utf8_unchecked_mut(from_raw_parts_mut(st as *mut u8, start_len)),
        //     from_utf8_unchecked_mut(from_raw_parts_mut(range.start as *mut u8, middle_len)),
        //     from_utf8_unchecked_mut(from_raw_parts_mut(range.end as *mut u8, end_len)),
        // )
        let st = self.as_ptr();
        let c1 = st.add(range.start);
        let c2 = st.add(range.end);
        (
            from_utf8_unchecked_mut(from_raw_parts_mut(st as *mut u8, range.start)),
            from_utf8_unchecked_mut(from_raw_parts_mut(c1 as *mut u8, range.end - range.start)),
            from_utf8_unchecked_mut(from_raw_parts_mut(c2 as *mut u8, self.len() - range.end)),
        )
    }

    #[inline]
    unsafe fn trim_start_unchecked(self, start: usize) -> Self {
        // let len = self.len() - (ptr.offset_from(self.as_ptr()) as usize);
        // from_utf8_unchecked_mut(from_raw_parts_mut(ptr as *mut u8, len))
        self.get_unchecked_mut(start..)
    }

    #[inline]
    unsafe fn trim_end_unchecked(self, end: usize) -> Self {
        // let len = ptr.offset_from(self.as_ptr()) as usize;
        // from_utf8_unchecked_mut(from_raw_parts_mut(self.as_ptr() as *mut u8, len))
        self.get_unchecked_mut(..end)
    }
}

impl<'h> IndexHaystack for &'h mut str {
    type Origin = *const u8;

    fn origin(&self) -> Self::Origin {
        self.as_ptr()
    }

    unsafe fn range_from_origin(&self, origin: Self::Origin) -> Range<usize> {
        let start = self.as_ptr().offset_from(origin) as usize;
        let end = start + self.len();
        start..end
    }
}

mod char;
mod func;
mod str;
