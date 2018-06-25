use pattern::*;
use haystack::SharedSpan;
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

unsafe impl Searcher for CharSearcher {
    type Hay = str;

    #[inline]
    fn search(&mut self, span: SharedSpan<'_, str>) -> Option<Range<usize>> {
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

unsafe impl Checker for CharChecker {
    type Hay = str;

    #[inline]
    fn is_prefix_of(self, haystack: &str) -> bool {
        let mut utf8_encoded = [0u8; 4];
        let encoded = self.0.encode_utf8(&mut utf8_encoded);
        haystack.as_bytes().get(..encoded.len()) == Some(encoded.as_bytes())
    }

    #[inline]
    fn trim_start(&mut self, haystack: &str) -> usize {
        let mut checker = Pattern::<&str>::into_checker(|c: char| c == self.0);
        checker.trim_start(haystack)
    }
}

unsafe impl ReverseSearcher for CharSearcher {
    #[inline]
    fn rsearch(&mut self, span: SharedSpan<'_, str>) -> Option<Range<usize>> {
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

unsafe impl ReverseChecker for CharChecker {
    #[inline]
    fn is_suffix_of(self, haystack: &str) -> bool {
        let mut utf8_encoded = [0u8; 4];
        let encoded_len = self.0.encode_utf8(&mut utf8_encoded).len();
        let suffix = haystack.as_bytes().get(haystack.len().wrapping_sub(encoded_len)..);
        suffix == Some(&utf8_encoded[..encoded_len])
    }

    #[inline]
    fn trim_end(&mut self, haystack: &str) -> usize {
        let mut checker = Pattern::<&str>::into_checker(|c: char| c == self.0);
        checker.trim_end(haystack)
    }
}

unsafe impl DoubleEndedSearcher for CharSearcher {}
unsafe impl DoubleEndedChecker for CharChecker {}

macro_rules! impl_pattern {
    ($ty:ty) => {
        impl<'h> Pattern<$ty> for char {
            type Searcher = CharSearcher;
            type Checker = CharChecker;

            #[inline]
            fn into_searcher(self) -> Self::Searcher {
                CharSearcher::new(self)
            }

            #[inline]
            fn into_checker(self) -> Self::Checker {
                CharChecker(self)
            }
        }
    }
}

impl_pattern!(&'h str);
impl_pattern!(&'h mut str);
