use haystack::{Hay, Span};
use std::ops::Range;

pub mod wtf8;
pub use self::wtf8::Wtf8;

impl Hay for Wtf8 {
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

    #[inline]
    unsafe fn restore_range(&self, range: Range<usize>, subrange: Range<usize>) -> Range<usize> {
        if self.is_empty() {
            return range;
        }
        let bytes = self.as_inner();

        let mut range_start = range.start;
        if let 0x80..=0xbf = bytes.get_unchecked(0) {
            range_start -= 1;
        }

        let mut subrange_end = subrange.end;
        if subrange_end >= 3 && subrange_end != bytes.len() && *bytes.get_unchecked(subrange_end - 3) >= 0xf0 {
            subrange_end -= 1;
        }

        let restore_index = |i| {
            if i == 0 {
                range.start
            } else if i == bytes.len() {
                range.end
            } else {
                range_start + i
            }
        };

        restore_index(subrange.start)..restore_index(subrange_end)
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

#[test]
fn test_restore_range() {
    fn do_test(sliced: &Wtf8, range: Range<usize>, map: &[(usize, usize)]) {
        for (src_i, dest_i) in map {
            for (src_j, dest_j) in map {
                if src_i <= src_j {
                    assert_eq!(
                        unsafe { sliced.restore_range(range.clone(), *src_i..*src_j) },
                        *dest_i..*dest_j,
                        "sliced = {:?}, range = {:?}, src = {}..{}",
                        sliced, range, src_i, src_j
                    )
                }
            }
        }
    }

    let full = Wtf8::from_str("\u{10000}\u{10000}");

    do_test(
        full,
        0..8,
        &[(0, 0), (2, 2), (4, 4), (6, 6), (8, 8)],
    );
    do_test(
        &full[2..8],
        2..8,
        &[(0, 2), (3, 4), (5, 6), (7, 8)],
    );
    do_test(
        &full[0..6],
        0..6,
        &[(0, 0), (2, 2), (4, 4), (7, 6)],
    );
    do_test(
        &full[2..6],
        2..6,
        &[(0, 2), (3, 4), (6, 6)],
    );
    do_test(
        &full[0..2],
        0..2,
        &[(0, 0), (3, 2)],
    );
    do_test(
        &full[4..6],
        4..6,
        &[(0, 4), (3, 6)],
    );
    do_test(
        &full[4..4],
        4..4,
        &[(0, 4)],
    );
    do_test(
        &full[2..2],
        2..2,
        &[(0, 2)],
    );

    let normal = unsafe { Wtf8::from_bytes_unchecked(b"\xed\xb0\x80\xed\xa0\x80") };
    do_test(
        normal,
        0..6,
        &[(0, 0), (3, 3), (6, 6)],
    );
    do_test(
        normal,
        3..6,
        &[(0, 3), (3, 6)],
    );
}

mod wtf8_pat;
