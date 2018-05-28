use traits::{Haystack, Pattern, ReversePattern};
use span::Span;

use super::StrCursor;

use memchr;
use std::iter::{Rev, FusedIterator};
use std::slice;

unsafe impl<'h> Pattern<'h, str> for char {
    type Searcher = CharSearcher<'h>;

    #[inline]
    fn into_searcher(self, haystack: &'h str) -> Self::Searcher {
        let mut utf8_encoded = [0u8; 4];
        let utf8_size = self.encode_utf8(&mut utf8_encoded).len();
        CharSearcher {
            remaining: haystack.to_span(),
            needle: self,
            utf8_size,
            utf8_encoded,
        }
    }

    #[inline]
    fn is_prefix_of(self, haystack: &'h str) -> bool {
        (|c: char| c == self).is_prefix_of(haystack)
    }

    #[inline]
    fn trim_start(&mut self, span: Span<'h, str>) -> Span<'h, str> {
        (|c: char| c == *self).trim_start(span)
    }
}

unsafe impl<'h> ReversePattern<'h, str> for char {
    type ReverseSearcher = Rev<Self::Searcher>;

    #[inline]
    fn into_reverse_searcher(self, haystack: &'h str) -> Self::ReverseSearcher {
        self.into_searcher(haystack).rev()
    }

    #[inline]
    fn is_suffix_of(self, haystack: &'h str) -> bool {
        (|c: char| c == self).is_suffix_of(haystack)
    }

    #[inline]
    fn trim_end(&mut self, span: Span<'h, str>) -> Span<'h, str> {
        (|c: char| c == *self).trim_end(span)
    }
}



#[derive(Debug)]
pub struct CharSearcher<'h> {
    // safety invariant: the boundary of the span must be a valid utf8 character
    // boundary. This invariant can be broken *within* `next` and `next_back`,
    // however they must exit with fingers on valid code point boundaries.
    remaining: Span<'h, str>,

    /// The character being searched for
    needle: char,

    // safety invariant: `utf8_size` must be less than 5
    utf8_size: usize,
    /// A utf8 encoded copy of the `needle`
    utf8_encoded: [u8; 4],
}

#[inline]
unsafe fn memchr_by_ptr<'h>(start: StrCursor<'h>, end: StrCursor<'h>, byte: u8) -> Option<StrCursor<'h>> {
    let len = end.offset_from(start);
    let sl = slice::from_raw_parts(start.ptr, len);
    let index = memchr::memchr(byte, sl)?;
    Some(start.add(index))
}

#[inline]
unsafe fn memrchr_by_ptr<'h>(start: StrCursor<'h>, end: StrCursor<'h>, byte: u8) -> Option<StrCursor<'h>> {
    let len = end.offset_from(start);
    let sl = slice::from_raw_parts(start.ptr, len);
    let index = memchr::memrchr(byte, sl)?;
    Some(start.add(index))
}

impl<'h> Iterator for CharSearcher<'h> {
    type Item = Span<'h, str>;

    #[inline]
    fn next(&mut self) -> Option<Span<'h, str>> {
        let start_limit = self.remaining.start();
        let end_limit = self.remaining.end();
        let last_byte = unsafe { *self.utf8_encoded.get_unchecked(self.utf8_size - 1) };
        let mut cur = start_limit;

        unsafe {
            while let Some(index) = memchr_by_ptr(cur, end_limit, last_byte) {
                cur = index.add(1);
                if cur.offset_from(start_limit) >= self.utf8_size {
                    let found_char = cur.sub(self.utf8_size);
                    let slice = slice::from_raw_parts(found_char.ptr, self.utf8_size);
                    if slice == &self.utf8_encoded[..self.utf8_size] {
                        let (_, middle, right) = self.remaining.split_twice(found_char, cur);
                        self.remaining = right;
                        return Some(middle);
                    }
                }
            }
        }

        self.remaining.collapse_to_end();
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let start = self.remaining.start();
        let end = self.remaining.end();
        let len = end.offset_from(start);
        (0, Some(len / self.utf8_size))
    }
}

impl<'h> FusedIterator for CharSearcher<'h> {}

impl<'h> DoubleEndedIterator for CharSearcher<'h> {
    #[inline]
    fn next_back(&mut self) -> Option<Span<'h, str>> {
        let start_limit = self.remaining.start();
        let end_limit = self.remaining.end();
        let last_byte = unsafe { *self.utf8_encoded.get_unchecked(self.utf8_size - 1) };
        let mut cur = end_limit;

        unsafe {
            while let Some(index) = memrchr_by_ptr(start_limit, cur, last_byte) {
                let next = index.add(1);
                if next.offset_from(start_limit) >= self.utf8_size {
                    let found_char = next.sub(self.utf8_size);
                    let slice = slice::from_raw_parts(found_char.ptr, self.utf8_size);
                    if slice == &self.utf8_encoded[..self.utf8_size] {
                        let (left, middle, _) = self.remaining.split_twice(found_char, next);
                        self.remaining = left;
                        return Some(middle);
                    }
                }
                cur = index;
            }
        }

        self.remaining.collapse_to_start();
        None
    }
}
