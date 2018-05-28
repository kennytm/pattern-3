extern crate pattern_3;

use pattern_3::*;

#[test]
fn test_try_fold() {
    let slice = &[0u32, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10][..];
    let (span, origin) = Span::and_origin(slice);
    let pat = |x: &u32| *x % 2 == 0;
    let searcher = pat.into_searcher(span);
    let res = searcher
        .map(|span| span.to_index(origin))
        .skip_while(|offset| *offset < 4)
        .collect::<Vec<_>>();
    assert_eq!(&res[..], &[4, 6, 8, 10][..]);
}

fn main() {}
