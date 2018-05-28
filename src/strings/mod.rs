use haystack::{Haystack, HaystackMut};
use cursor::RawCursor;
use span::Span;
use std::{str, slice, ptr};

impl Haystack for str {
    type Cursor = *const u8;
    type Origin = *const u8;

    fn as_origin_raw(&self) -> Self::Origin {
        self.as_ptr()
    }

    fn as_span_raw(&self) -> (Self::Cursor, Self::Cursor) {
        let start = self.as_ptr();
        let end = unsafe { start.add(self.len()) };
        (start, end)
    }

    unsafe fn from_span_raw<'h>(
        _: Self::Origin,
        start: Self::Cursor,
        end: Self::Cursor,
    ) -> &'h Self {
        let len = end.offset_from(start) as usize;
        let slice = slice::from_raw_parts(start, len);
        str::from_utf8_unchecked(slice)
    }
}

impl HaystackMut for str {
    unsafe fn from_span_raw_mut<'h>(
        _: Self::Origin,
        start: Self::Cursor,
        end: Self::Cursor,
    ) -> &'h mut Self {
        let len = end.offset_from(start) as usize;
        let slice = slice::from_raw_parts_mut(start as *mut u8, len);
        str::from_utf8_unchecked_mut(slice)
    }
}

impl RawCursor<str> for *const u8 {
    unsafe fn to_index(self, origin: *const u8) -> usize {
        self.offset_from(origin) as usize
    }
}

impl<'h> Span<'h, str> {
    fn as_str(&self) -> &str {
        unsafe {
            let start = self.start().raw();
            let end = self.end().raw();
            <str>::from_span_raw(ptr::null(), start, end)
        }
    }
}

// mod char;
mod func;
