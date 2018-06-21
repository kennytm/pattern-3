use pattern::{Pattern, Searcher, ReverseSearcher, DoubleEndedSearcher};
use memchr::{memchr, memrchr};
use std::ops::Range;

#[derive(Debug, Clone)]
pub struct CharSearcher {
    // safety invariant: `utf8_size` must be less than 5
    utf8_size: usize,

    /// A utf8 encoded copy of the `needle`
    utf8_encoded: [u8; 4],
}

impl CharSearcher {
    #[inline]
    fn as_bytes(&self) -> &[u8] {
        unsafe {
            self.utf8_encoded.get_unchecked(..self.utf8_size)
        }
    }

    #[inline]
    fn last_byte(&self) -> u8 {
        unsafe {
            *self.utf8_encoded.get_unchecked(self.utf8_size - 1)
        }
    }
}

unsafe impl Searcher for CharSearcher {
    type Hay = str;

    fn search(&mut self, h: &str) -> Option<Range<usize>> {
        let mut finger = 0;
        let bytes = h.as_bytes();
        unsafe {
            loop {
                let index = memchr(self.last_byte(), bytes.get_unchecked(finger..))?;
                finger += index + 1;
                if finger >= self.utf8_size {
                    let found = bytes.get_unchecked((finger - self.utf8_size)..finger);
                    if found == self.as_bytes() {
                        // let start = h.as_ptr().add(finger - self.utf8_size);
                        // let end = h.as_ptr().add(finger);
                        // return Some(start..end)
                        return Some((finger - self.utf8_size)..finger);
                    }
                }
            }
        }
    }
}

unsafe impl ReverseSearcher for CharSearcher {
    fn rsearch(&mut self, h: &str) -> Option<Range<usize>> {
        let mut bytes = h.as_bytes();
        unsafe {
            loop {
                let index = memrchr(self.last_byte(), bytes)? + 1;
                if index >= self.utf8_size {
                    let found = bytes.get_unchecked((index - self.utf8_size)..index);
                    if found == self.as_bytes() {
                        // let start = h.as_ptr().add(index - self.utf8_size);
                        // let end = h.as_ptr().add(index);
                        // return Some(start..end)
                        return Some((index - self.utf8_size)..index);
                    }
                }
                bytes = bytes.get_unchecked(..(index - 1));
            }
        }
    }
}

unsafe impl DoubleEndedSearcher for CharSearcher {}

impl Pattern<str> for char {
    type Searcher = CharSearcher;

    #[inline]
    fn into_searcher(self) -> Self::Searcher {
        let mut utf8_encoded = [0u8; 4];
        let utf8_size = self.encode_utf8(&mut utf8_encoded).len();
        CharSearcher {
            utf8_size,
            utf8_encoded,
        }
    }

    #[inline]
    fn is_prefix_of(self, haystack: &str) -> bool {
        let mut utf8_encoded = [0u8; 4];
        let encoded = self.encode_utf8(&mut utf8_encoded);
        haystack.as_bytes().get(..encoded.len()) == Some(encoded.as_bytes())
    }

    #[inline]
    fn trim_start(&mut self, haystack: &str) -> usize {
        (|c: char| c == *self).trim_start(haystack)
    }

    #[inline]
    fn is_suffix_of(self, haystack: &str) -> bool {
        let mut utf8_encoded = [0u8; 4];
        let encoded = self.encode_utf8(&mut utf8_encoded);
        if encoded.len() > haystack.len() {
            return false;
        }
        let suffix = unsafe { haystack.get_unchecked((haystack.len() - encoded.len())..) };
        suffix == encoded
    }

    #[inline]
    fn trim_end(&mut self, haystack: &str) -> usize {
        (|c: char| c == *self).trim_end(haystack)
    }
}
