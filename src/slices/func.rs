use pattern::{Pattern, Searcher, ReverseSearcher, DoubleEndedSearcher};
use haystack::Haystack;
use super::SliceLike;
use std::mem::{replace, size_of};

pub struct ElemSearcher<F> {
    predicate: F,
}

#[inline]
fn search<H, F>(predicate: &mut F, rest: &mut H) -> (H, Option<H>)
where
    F: FnMut(&H::Item) -> bool,
    H: SliceLike + Haystack,
{
    let h = rest.collapse_to_end();
    match h.iter().position(predicate) {
        Some(i) => {
            let (before, found, after) = unsafe { h.split_around_unchecked(i..(i + 1)) };
            *rest = after;
            (before, Some(found))
        }
        None => {
            (h, None)
        }
    }
}

#[inline]
fn rsearch<H, F>(predicate: &mut F, rest: &mut H) -> (Option<H>, H)
where
    F: FnMut(&H::Item) -> bool,
    H: SliceLike + Haystack,
{
    let h = rest.collapse_to_start();
    match h.iter().rposition(predicate) {
        Some(i) => {
            let (before, found, after) = unsafe { h.split_around_unchecked(i..(i + 1)) };
            *rest = before;
            (Some(found), after)
        }
        None => {
            (None, h)
        }
    }
}

#[inline]
fn is_prefix_of<T, F>(mut predicate: F, rest: &[T]) -> bool
where
    F: FnMut(&T) -> bool,
{
    if let Some(x) = rest.first() {
        predicate(x)
    } else {
        false
    }
}

#[inline]
fn is_suffix_of<T, F>(mut predicate: F, rest: &[T]) -> bool
where
    F: FnMut(&T) -> bool,
{
    if let Some(x) = rest.last() {
        predicate(x)
    } else {
        false
    }
}

#[inline]
fn trim_start<H, F>(predicate: &mut F, rest: &mut H)
where
    F: FnMut(&H::Item) -> bool,
    H: SliceLike + Haystack,
{
    // FIXME: Implementing this using pointer arithmetic could be 3% faster.
    // Note that a pointer arithmetic solution needs to deal with ZSTs properly.
    let h = rest.collapse_to_start();
    let len = h.len();
    let position = {
        let mut it = h.iter();
        if it.find(|x| !predicate(x)).is_some() {
            len - it.as_slice().len() - 1
        } else {
            len
        }
    };
    *rest = unsafe { h.slice_from_unchecked(position) }
}

#[inline]
fn trim_end<H, F>(predicate: &mut F, rest: &mut H)
where
    F: FnMut(&H::Item) -> bool,
    H: SliceLike + Haystack,
{
    let h = rest.collapse_to_start();
    let position = h.iter().rposition(|x| !predicate(x)).map_or(0, |p| p + 1);
    *rest = unsafe { h.slice_to_unchecked(position) };
}

macro_rules! impl_pattern_for_slice_like {
    ([$($gen:tt)*] $ty:ty) => {
        impl $($gen)* Pattern<$ty> for F
        where
            F: FnMut(&T) -> bool,
        {
            type Searcher = ElemSearcher<F>;

            #[inline]
            fn into_searcher(self) -> ElemSearcher<F> {
                ElemSearcher {
                    predicate: self,
                }
            }

            #[inline]
            fn is_prefix_of(self, haystack: $ty) -> bool {
                is_prefix_of(self, &*haystack)
            }

            #[inline]
            fn trim_start(&mut self, haystack: &mut $ty) {
                trim_start(self, haystack)
            }

            #[inline]
            fn is_suffix_of(self, haystack: $ty) -> bool {
                is_suffix_of(self, &*haystack)
            }

            #[inline]
            fn trim_end(&mut self, haystack: &mut $ty) {
                trim_end(self, haystack)
            }
        }

        unsafe impl $($gen)* Searcher<$ty> for ElemSearcher<F>
        where
            F: FnMut(&T) -> bool,
        {
            #[inline]
            fn search(&mut self, haystack: &mut $ty) -> ($ty, Option<$ty>) {
                search(&mut self.predicate, haystack)
            }
        }

        unsafe impl $($gen)* ReverseSearcher<$ty> for ElemSearcher<F>
        where
            F: FnMut(&T) -> bool,
        {
            #[inline]
            fn rsearch(&mut self, haystack: &mut $ty) -> (Option<$ty>, $ty) {
                rsearch(&mut self.predicate, haystack)
            }
        }

        unsafe impl $($gen)* DoubleEndedSearcher<$ty> for ElemSearcher<F>
        where
            F: FnMut(&T) -> bool,
        {}
    }
}

impl_pattern_for_slice_like!([<'h, T: 'h, F>] &'h [T]);
impl_pattern_for_slice_like!([<'h, T: 'h, F>] &'h mut [T]);
impl_pattern_for_slice_like!([<T, F>] Vec<T>);
