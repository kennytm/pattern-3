#![feature(test)]

extern crate pattern_3;
extern crate test;

use pattern_3::Pattern;
use test::{black_box, Bencher};

#[bench]
fn bench_slice_fn_long_with_next(b: &mut Bencher) {
    let sl = [1u32, 2, 9, 1, 3, 7, 5].iter().cloned().cycle().take(50000).collect::<Vec<_>>();
    b.iter(|| {
        let searcher = (|c: &u32| *c == 1).into_searcher(&*sl);
        for span in searcher {
            black_box(span);
        }
    });
}

#[bench]
fn bench_slice_fn_long_with_for_each(b: &mut Bencher) {
    let sl = [1u32, 2, 9, 1, 3, 7, 5].iter().cloned().cycle().take(50000).collect::<Vec<_>>();
    b.iter(|| {
        let searcher = (|c: &u32| *c == 1).into_searcher(&*sl);
        searcher.for_each(|span| { black_box(span); });
    });
}

#[bench]
fn bench_slice_fn_short(b: &mut Bencher) {
    let sl = [1u32, 2, 9, 1, 3, 7, 5];
    b.iter(|| {
        let searcher = (|c: &u32| *c == 1).into_searcher(&sl);
        assert_eq!(
            searcher.map(|span| span.to_offset()).collect::<Vec<_>>(),
            vec![0, 3]
        );
    });
}

fn main() {}