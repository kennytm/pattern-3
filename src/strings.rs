use traits::{Haystack, HaystackMut, Searcher};
use span::Span;
use std::ops::Range;
use std::{str, slice};
use memchr;

impl Haystack for str {
    type Item = u8;

    fn start_cursor_at_front(&self) -> *const u8 {
        self.as_ptr()
    }
    fn end_cursor_at_back(&self) -> *const u8 {
        unsafe { self.as_ptr().add(self.len()) }
    }

    unsafe fn start_cursor_to_offset(&self, cur: *const u8) -> usize {
        cur.offset_from(self.as_ptr()) as usize
    }

    unsafe fn end_cursor_to_offset(&self, cur: *const u8) -> usize {
        cur.offset_from(self.as_ptr()) as usize
    }

    unsafe fn from_cursor_range<'a>(range: Range<*const u8>) -> &'a Self {
        let len = range.end.offset_from(range.start) as usize;
        let slice = slice::from_raw_parts(range.start, len);
        str::from_utf8_unchecked(slice)
    }

    unsafe fn start_to_end_cursor(&self, start_cur: *const u8) -> *const u8 {
        start_cur
    }

    unsafe fn end_to_start_cursor(&self, end_cur: *const u8) -> *const u8 {
        end_cur
    }
}

impl HaystackMut for str {
    unsafe fn from_cursor_range_mut<'a>(range: Range<*mut u8>) -> &'a mut Self {
        let len = range.end.offset_from(range.start) as usize;
        let slice = slice::from_raw_parts_mut(range.start, len);
        str::from_utf8_unchecked_mut(slice)
    }
}


pub struct CharSearcher<'a> {
    // safety invariant: the boundary of the span must be a valid utf8 character
    // boundary. This invariant can be broken *within* `next` and `next_back`,
    // however they must exit with fingers on valid code point boundaries.
    remaining: Span<'a, str>,

    /// The character being searched for
    needle: char,

    // safety invariant: `utf8_size` must be less than 5
    utf8_size: usize,
    /// A utf8 encoded copy of the `needle`
    utf8_encoded: [u8; 4],
}

impl<'a> Iterator for CharSearcher<'a> {
    type Item = Span<'a, str>;

    #[inline]
    fn next(&mut self) -> Option<Span<'a, str>> {
        loop {
            let bytes = self.remaining.to_slice();
            if bytes.is_empty() {
                return None;
            }
            let last_byte = unsafe { *self.utf8_encoded.get_unchecked(self.utf8_size - 1) };
            if let Some(index) = memchr::memchr(last_byte, bytes) {
                unsafe {
                    self.remaining.inc_start(index + 1);
                    let finger = self.remaining.start();
                    let found_char = finger.wrapping_sub(self.utf8_size);
                    if let Some(span) = self.remaining.with_cursor_range_checked(found_char..finger) {
                        if span.to_slice() == &self.utf8_encoded[..self.utf8_size] {
                            return Some(span);
                        }
                    }
                }
            } else {
                self.remaining.collapse_to_end();
                return None;
            }
        }
    }
}

unsafe impl<'a> Searcher<'a, str> for CharSearcher<'a> {
    fn next_immediate(&mut self) -> Option<Span<'a, str>> {
        let remaining = self.remaining.to_slice();
        if remaining.len() >= self.
    }
}
