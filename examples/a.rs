#![crate_type = "dylib"]

extern crate pattern_3;

// use pattern_3::{Pattern, Haystack, Searcher};
use pattern_3::ext::HaystackExt;

#[inline(never)]
pub fn sw1(a: &[u64], b: u64) -> &[u64] {
    HaystackExt::trim_matches(a, |c| c == &b)
}


fn main() {
    const E: &[u64] = &[];
    assert_eq!(&[2,3,4], sw1(&[1,2,3,4], 1));
    assert_eq!(&[4,3,2], sw1(&[4,3,2,1,1], 1));
    assert_eq!(&[1,5,2,8], sw1(&[1,5,2,8], 4));
    assert_eq!(E, sw1(&[6,6,6,6], 6));
    assert_eq!(E, sw1(E, 23));
    assert_eq!(E, sw1(&[24], 24));
    assert_eq!(&[5], sw1(&[5], 25));
    assert_eq!(&[75,6,77], sw1(&[6,6,75,6,77,6,6,6], 6));
}
