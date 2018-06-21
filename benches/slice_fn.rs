#![feature(test)]

extern crate pattern_3;
extern crate test;

use pattern_3::ext;
use test::{black_box, Bencher};

// ~70000 ns/iter
#[bench]
fn bench_long_with_matches_next(b: &mut Bencher) {
    let sl = [1u32, 2, 9, 1, 3, 7, 5].iter().cloned().cycle().take(50000).collect::<Vec<_>>();
    b.iter(|| {
        let searcher = ext::matches(&*sl, |c: &u32| *c == 1);
        for span in searcher {
            black_box(span);
        }
    });
}

// ~130000 ns/iter
#[bench]
fn bench_long_with_filter_next(b: &mut Bencher) {
    let sl = [1u32, 2, 9, 1, 3, 7, 5].iter().cloned().cycle().take(50000).collect::<Vec<_>>();
    b.iter(|| {
        let searcher = sl.iter().filter(|c: &&u32| **c == 1);
        for span in searcher {
            black_box(span);
        }
    });
}

// ~70000 ns/iter [*slower*]
#[bench]
fn bench_long_with_matches_for_each(b: &mut Bencher) {
    let sl = [1u32, 2, 9, 1, 3, 7, 5].iter().cloned().cycle().take(50000).collect::<Vec<_>>();
    b.iter(|| {
        let searcher = ext::matches(&*sl, |c: &u32| *c == 1);
        searcher.for_each(|span| { black_box(span); });
    });
}

// ~40000 ns/iter
#[bench]
fn bench_long_with_filter_for_each(b: &mut Bencher) {
    let sl = [1u32, 2, 9, 1, 3, 7, 5].iter().cloned().cycle().take(50000).collect::<Vec<_>>();
    b.iter(|| {
        let searcher = sl.iter().filter(|c: &&u32| **c == 1);
        searcher.for_each(|span| { black_box(span); });
    });
}

// ~100 ns/iter
#[bench]
fn bench_short(b: &mut Bencher) {
    let sl = [1u32, 2, 9, 1, 3, 7, 5];
    b.iter(|| {
        let searcher = ext::match_indices(&sl[..], |c: &u32| *c == 1);
        assert_eq!(
            searcher.map(|(index, _)| index).collect::<Vec<_>>(),
            vec![0, 3]
        );
    });
}

// ~18000 ns/iter
#[bench]
fn bench_trim_start(b: &mut Bencher) {
    let sl = (1..50000).collect::<Vec<_>>();
    b.iter(|| {
        let res = black_box(ext::trim_start(&*sl, |c: &u32| *c < 80000));
        assert!(res.is_empty());
    });
}

// ~18000 ns/iter
#[bench]
fn bench_trim_end(b: &mut Bencher) {
    let sl = (1..50000).collect::<Vec<_>>();
    b.iter(|| {
        let res = black_box(ext::trim_end(&*sl, |c: &u32| *c < 80000));
        assert!(res.is_empty());
    });
}

// ~9000 ns/iter
#[bench]
fn bench_trim_start_half(b: &mut Bencher) {
    let sl = (1..50000).collect::<Vec<_>>();
    b.iter(|| {
        black_box(ext::trim_start(&*sl, |c: &u32| *c < 25000));
    });
}

// ~9000 ns/iter
#[bench]
fn bench_trim_end_half(b: &mut Bencher) {
    let sl = (1..50000).collect::<Vec<_>>();
    b.iter(|| {
        black_box(ext::trim_end(&*sl, |c: &u32| *c > 25000));
    });
}

fn main() {}