use pattern::*;
use haystack::Span;
use memchr::{memchr, memrchr};
use std::ops::Range;

#[derive(Debug, Clone)]
pub struct CharSearcher {
    // safety invariant: `utf8_size` must be less than 5
    utf8_size: usize,

    /// A utf8 encoded copy of the `needle`
    utf8_encoded: [u8; 4],
}

#[derive(Debug, Clone)]
pub struct CharChecker(char);

impl CharSearcher {
    #[inline]
    fn as_bytes(&self) -> &[u8] {
        &self.utf8_encoded[..self.utf8_size]
    }

    #[inline]
    fn last_byte(&self) -> u8 {
        self.utf8_encoded[self.utf8_size - 1]
    }

    #[inline]
    fn new(c: char) -> Self {
        let mut utf8_encoded = [0u8; 4];
        let utf8_size = c.encode_utf8(&mut utf8_encoded).len();
        CharSearcher {
            utf8_size,
            utf8_encoded,
        }
    }
}

unsafe impl Searcher<str> for CharSearcher {
    #[inline]
    fn search(&mut self, span: Span<&str>) -> Option<Range<usize>> {
        let (hay, range) = span.into_parts();
        let mut finger = range.start;
        let bytes = hay.as_bytes();
        loop {
            let index = memchr(self.last_byte(), &bytes[finger..range.end])?;
            finger += index + 1;
            if finger >= self.utf8_size {
                let found = &bytes[(finger - self.utf8_size)..finger];
                if found == self.as_bytes() {
                    return Some((finger - self.utf8_size)..finger);
                }
            }
        }
    }
}

unsafe impl Consumer<str> for CharChecker {
    #[inline]
    fn consume(&mut self, span: Span<&str>) -> Option<usize> {
        let (hay, range) = span.into_parts();
        let mut utf8_encoded = [0u8; 4];
        let encoded = self.0.encode_utf8(&mut utf8_encoded);
        let check_end = range.start + encoded.len();
        if range.end < check_end {
            return None;
        }
        if unsafe { hay.as_bytes().get_unchecked(range.start..check_end) } == encoded.as_bytes() {
            Some(check_end)
        } else {
            None
        }
    }

    #[inline]
    fn trim_start(&mut self, hay: &str) -> usize {
        let mut checker = Pattern::<&str>::into_consumer(|c: char| c == self.0);
        checker.trim_start(hay)
    }
}

unsafe impl ReverseSearcher<str> for CharSearcher {
    #[inline]
    fn rsearch(&mut self, span: Span<&str>) -> Option<Range<usize>> {
        let (hay, range) = span.into_parts();
        let start = range.start;
        let mut bytes = hay[range].as_bytes();
        loop {
            let index = memrchr(self.last_byte(), bytes)? + 1;
            if index >= self.utf8_size {
                let found = &bytes[(index - self.utf8_size)..index];
                if found == self.as_bytes() {
                    let index = index + start;
                    return Some((index - self.utf8_size)..index);
                }
            }
            bytes = &bytes[..(index - 1)];
        }
    }
}

unsafe impl ReverseConsumer<str> for CharChecker {
    #[inline]
    fn rconsume(&mut self, span: Span<&str>) -> Option<usize> {
        // let (hay, range) = span.into_parts();
        // let mut utf8_encoded = [0u8; 4];
        // let encoded_len = self.0.encode_utf8(&mut utf8_encoded).len();
        // let index = range.end.wrapping_sub(encoded_len);
        // if hay.as_bytes().get(range.start..index) == Some(&utf8_encoded[..encoded_len]) {
        //     Some(index)
        // } else {
        //     None
        // }
        let (hay, range) = span.into_parts();
        let mut utf8_encoded = [0u8; 4];
        let encoded = self.0.encode_utf8(&mut utf8_encoded);
        if range.start + encoded.len() > range.end {
            return None;
        }
        let check_start = range.end - encoded.len();
        if unsafe { hay.as_bytes().get_unchecked(check_start..range.end) } == encoded.as_bytes() {
            Some(check_start)
        } else {
            None
        }
    }

    #[inline]
    fn trim_end(&mut self, haystack: &str) -> usize {
        let mut checker = Pattern::<&str>::into_consumer(|c: char| c == self.0);
        checker.trim_end(haystack)
    }
}

unsafe impl DoubleEndedSearcher<str> for CharSearcher {}
unsafe impl DoubleEndedConsumer<str> for CharChecker {}

macro_rules! impl_pattern {
    ($ty:ty) => {
        impl<'h> Pattern<$ty> for char {
            type Searcher = CharSearcher;
            type Consumer = CharChecker;

            #[inline]
            fn into_searcher(self) -> Self::Searcher {
                CharSearcher::new(self)
            }

            #[inline]
            fn into_consumer(self) -> Self::Consumer {
                CharChecker(self)
            }
        }
    }
}

impl_pattern!(&'h str);
impl_pattern!(&'h mut str);
