#![feature(
    ptr_offset_from,
    range_is_empty,
    fn_traits,
    unboxed_closures,
    linkage,
    specialization,
    try_trait,
    fundamental,
    test,
    align_offset,
    slice_internals,
    exact_size_is_empty,
    trusted_len,
    iterator_find_map,
    ptr_wrapping_offset_from,
    generic_associated_types,
    slice_get_slice,
    core_intrinsics,
    catch_expr,
    associated_type_defaults,
    generic_associated_types,
    slice_index_methods,
    step_trait,
    exact_chunks,
)]
#![cfg_attr(test, allow(warnings))]

extern crate memchr;

// #[macro_use]
// mod macros;

pub mod haystack;
pub mod pattern;
mod slices;
mod strings;
pub mod ext;

pub use haystack::{Hay, Haystack};
pub use pattern::{Pattern, Searcher, ReverseSearcher, DoubleEndedSearcher};
