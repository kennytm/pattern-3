use haystack::{Hay, Span};
use std::ops::Range;

pub mod wtf8;
pub use self::wtf8::Wtf8;

unsafe impl Hay for Wtf8 {
    type Index = usize;

    #[inline]
    fn empty<'a>() -> &'a Self {
        Wtf8::from_str("")
    }

    #[inline]
    fn start_index(&self) -> usize {
        0
    }

    #[inline]
    fn end_index(&self) -> usize {
        self.len()
    }

    #[inline]
    unsafe fn slice_unchecked(&self, range: Range<usize>) -> &Self {
        &self[range]
    }

    #[inline]
    unsafe fn next_index(&self, index: usize) -> usize {
        let offset = match *self.as_inner().get_unchecked(index) {
            0x00..=0x7f => 1,
            0x80..=0xbf => if index == 0 { 3 } else { 2 },
            0xc0..=0xdf => 2,
            0xe0..=0xef => 3,
            0xf0..=0xff => if index + 3 == self.len() { 3 } else { 2 },
            _ => unreachable!(),
        };
        index + offset
    }

    #[inline]
    unsafe fn prev_index(&self, index: usize) -> usize {
        let bytes = self.as_inner();
        let mut e = index - 1;

        let mut c = *bytes.get_unchecked(e);
        if c < 0x80 {
            return e;
        }
        e -= 1;
        c = *bytes.get_unchecked(e);
        if c >= 0xc0 {
            return e;
        }
        e -= 1;
        c = *bytes.get_unchecked(e);
        if c < 0xc0 && e != 0 {
            e += 1;
        }
        e
    }
}

#[test]
fn test_wtf8_next_last_index() {
    let string = unsafe { Wtf8::from_bytes_unchecked(b"a\xc3\xa9 \xed\xa0\xbd\xf0\x9f\x92\xa9") };
    unsafe {
        for w in [0, 1, 3, 4, 7, 9, 11].windows(2) {
            let i = w[0];
            let j = w[1];
            assert_eq!(string.next_index(i), j);
            assert_eq!(string.prev_index(j), i);
        }
    }
}

impl<'h> Span<&'h Wtf8> {
    pub fn as_bytes(self) -> Span<&'h [u8]> {
        let (haystack, range) = self.into_parts();
        unsafe {
            Span::from_parts(haystack.as_inner(), range)
        }
    }
}

mod wtf8_pat;
