use pattern::*;
use haystack::{Haystack, Span};
use memchr::{memchr, memrchr};
use std::ops::Range;

#[derive(Debug, Clone)]
pub struct CharSearcher {
    // safety invariant: `utf8_size` must be less than 5
    utf8_size: usize,

    /// A utf8 encoded copy of the `needle`
    utf8_encoded: [u8; 4],

    /// The character currently being searched.
    c: char,
}

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
            c,
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

unsafe impl Consumer<str> for CharSearcher {
    #[inline]
    fn consume(&mut self, span: Span<&str>) -> Option<usize> {
        let mut consumer = Pattern::<&[u8]>::into_consumer(self.as_bytes());
        consumer.consume(span.as_bytes())
    }

    #[inline]
    fn trim_start(&mut self, hay: &str) -> usize {
        let mut consumer = Pattern::<&str>::into_consumer(|c: char| c == self.c);
        consumer.trim_start(hay)
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

unsafe impl ReverseConsumer<str> for CharSearcher {
    #[inline]
    fn rconsume(&mut self, span: Span<&str>) -> Option<usize> {
        if self.utf8_size == 1 {
            let mut consumer = Pattern::<&[u8]>::into_consumer(|b: &u8| *b == self.c as u8);
            consumer.rconsume(span.as_bytes())
        } else {
            let mut consumer = Pattern::<&str>::into_consumer(|c: char| c == self.c);
            consumer.rconsume(span)
        }
    }

    #[inline]
    fn trim_end(&mut self, haystack: &str) -> usize {
        let mut consumer = Pattern::<&str>::into_consumer(|c: char| c == self.c);
        consumer.trim_end(haystack)
    }
}

unsafe impl DoubleEndedSearcher<str> for CharSearcher {}
unsafe impl DoubleEndedConsumer<str> for CharSearcher {}

impl<H: Haystack<Target = str>> Pattern<H> for char {
    type Searcher = CharSearcher;
    type Consumer = CharSearcher;

    #[inline]
    fn into_searcher(self) -> Self::Searcher {
        CharSearcher::new(self)
    }

    #[inline]
    fn into_consumer(self) -> Self::Consumer {
        CharSearcher::new(self)
    }
}
