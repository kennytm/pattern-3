extern crate pattern_3;

use pattern_3::ext::*;

use std::f64::NAN;

#[test]
fn test_simple() {
    let haystack = &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 2.0, 3.0, 2.0, 4.0, 8.0][..];
    let needle = &[2.0, 3.0][..];

    assert_eq!(match_indices(haystack, needle).collect::<Vec<_>>(), vec![
        (1, needle),
        (8, needle),
    ]);
}

#[test]
fn test_empty() {
    let haystack = &[1.0, 2.0, 3.0][..];
    let needle = &[][..];

    assert_eq!(match_indices(haystack, needle).collect::<Vec<_>>(), vec![
        (0, needle),
        (1, needle),
        (2, needle),
        (3, needle),
    ]);
}

#[test]
fn test_nan_haystack() {
    let haystack = &[1.0, 2.0, NAN, 1.0, 2.0, NAN, 1.0, NAN, NAN, NAN, 2.0, 1.0, 2.0][..];
    let needle = &[1.0, 2.0][..];

    assert_eq!(match_indices(haystack, needle).collect::<Vec<_>>(), vec![
        (0, needle),
        (3, needle),
        (11, needle),
    ]);
}

#[test]
fn test_nan_needle() {
    let haystack = &[1.0, 2.0, NAN, 1.0, 2.0, NAN, 1.0, NAN, NAN, NAN, 2.0, 1.0, 2.0][..];
    let needle = &[2.0, NAN][..];

    // NaN should match nothing because NaN != NaN.
    assert_eq!(match_indices(haystack, needle).collect::<Vec<_>>(), vec![
    ]);
}

#[test]
fn test_negative_zero() {
    let haystack = &[-0.0, 0.0, 0.0, -0.0, 0.0][..];
    let needle = &[0.0, -0.0][..];

    assert_eq!(match_indices(haystack, needle).collect::<Vec<_>>(), vec![
        (0, needle),
        (2, needle),
    ]);
}

fn main() {}
