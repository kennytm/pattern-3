#![feature(
    specialization,
    // used in two places:
    //  1. to support using the same two-way algorithm for non-Ord slices
    //  2. to distinguish SpanBehavior between SharedHaystack and (unique) Haystack.

    unboxed_closures,
    fn_traits,
    // used to treat `&[char]` as if a `FnMut(char)->bool`.

    arbitrary_self_types,
    // just for convenience in Wtf8 impl, not required by Pattern API

    ptr_offset_from,
    iterator_find_map,
    int_to_from_bytes,
    // some useful library features
)]
#![cfg_attr(test, allow(warnings))]

#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(not(feature = "std"))]
extern crate core as std;

extern crate memchr;

pub mod haystack;
pub mod pattern;
mod slices;
mod strings;
mod omgwtf8;
pub mod ext;

pub use haystack::{Hay, Haystack, Span};
pub use pattern::{Pattern, Searcher, ReverseSearcher, DoubleEndedSearcher, Checker, ReverseChecker, DoubleEndedChecker};
pub use omgwtf8::Wtf8;
