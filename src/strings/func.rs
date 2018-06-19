use pattern::{Pattern, Searcher, ReverseSearcher, DoubleEndedSearcher};
use haystack::Haystack;
use super::StrLike;

pub struct MultiCharSearcher<F> {
    predicate: F,
}

pub struct MultiCharEq<'p>(&'p [char]);

impl_unboxed_functions! {
    [<'p>]
    MultiCharEq<'p> = |&self, c: char| -> bool {
        self.0.iter().any(|ch| *ch == c)
    }
}

#[inline]
fn search<H, F>(predicate: &mut F, rest: &mut H) -> (H, Option<H>)
where
    F: FnMut(char) -> bool,
    H: StrLike + Haystack,
{
    let h = rest.collapse_to_end();

    let range = do catch {
        let mut chars = h.chars();
        let c = chars.find(|c| predicate(*c))?;
        let end = chars.as_str().as_ptr();
        let start = unsafe { end.sub(c.len_utf8()) };
        start..end
    };

    if let Some(range) = range {
        let (before, found, after) = unsafe { h.split_around_ptr_unchecked(range) };
        *rest = after;
        (before, Some(found))
    } else {
        (h, None)
    }
}

#[inline]
fn rsearch<H, F>(predicate: &mut F, rest: &mut H) -> (Option<H>, H)
where
    F: FnMut(char) -> bool,
    H: StrLike + Haystack,
{
    let h = rest.collapse_to_start();

    let range = do catch {
        let mut chars = h.chars();
        let c = chars.rfind(|c| predicate(*c))?;
        let string = chars.as_str();
        unsafe {
            let start = string.as_ptr().add(string.len());
            let end = start.add(c.len_utf8());
            start..end
        }
    };

    if let Some(range) = range {
        let (before, found, after) = unsafe { h.split_around_ptr_unchecked(range) };
        *rest = before;
        (Some(found), after)
    } else {
        (None, h)
    }
}

#[inline]
fn is_prefix_of<F>(mut predicate: F, rest: &str) -> bool
where
    F: FnMut(char) -> bool,
{
    if let Some(c) = rest.chars().next() {
        predicate(c)
    } else {
        false
    }
}

#[inline]
fn is_suffix_of<F>(mut predicate: F, rest: &str) -> bool
where
    F: FnMut(char) -> bool,
{
    if let Some(c) = rest.chars().next_back() {
        predicate(c)
    } else {
        false
    }
}

#[inline]
fn trim_start<H, F>(predicate: &mut F, rest: &mut H)
where
    F: FnMut(char) -> bool,
    H: StrLike + Haystack,
{
    let h = rest.collapse_to_start();
    let ptr = {
        let mut chars = h.chars();
        let unconsume_amount = chars
            .find_map(|c| if !predicate(c) { Some(c.len_utf8()) } else { None })
            .unwrap_or(0);
        // eprintln!("{:?} / {:?} / {:?}", &*h, chars.as_str(), unconsume_amount);
        unsafe { chars.as_str().as_ptr().sub(unconsume_amount) }
    };
    *rest = unsafe { h.slice_from_ptr_unchecked(ptr) };
}

#[inline]
fn trim_end<H, F>(predicate: &mut F, rest: &mut H)
where
    F: FnMut(char) -> bool,
    H: StrLike + Haystack,
{
    // `find.map_or` is faster in trim_end in the microbenchmark, while
    // `find.unwrap_or` is faster in trim_start. Don't ask me why.
    let h = rest.collapse_to_start();
    let index = {
        let mut chars = h.chars();
        let unconsume_amount = chars
            .by_ref()
            .rev() // btw, `rev().find()` is faster than `rfind()`
            .find(|c| !predicate(*c))
            .map_or(0, |c| c.len_utf8());
        chars.as_str().len() + unconsume_amount
    };
    *rest = unsafe { h.slice_to_unchecked(index) };
}

macro_rules! impl_pattern_for_str_like {
    ([$($gen_f:tt)*] [$($gen_p:tt)*] $ty:ty) => {
        impl $($gen_f)* Pattern<$ty> for F
        where
            F: FnMut(char) -> bool,
        {
            type Searcher = MultiCharSearcher<F>;

            #[inline]
            fn into_searcher(self) -> Self::Searcher {
                MultiCharSearcher {
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

        impl $($gen_p)* Pattern<$ty> for &'p [char] {
            type Searcher = MultiCharSearcher<MultiCharEq<'p>>;

            #[inline]
            fn into_searcher(self) -> Self::Searcher {
                MultiCharSearcher {
                    predicate: MultiCharEq(self),
                }
            }

            #[inline]
            fn is_prefix_of(self, haystack: $ty) -> bool {
                is_prefix_of(MultiCharEq(self), &*haystack)
            }

            #[inline]
            fn trim_start(&mut self, haystack: &mut $ty) {
                trim_start(&mut MultiCharEq(*self), haystack)
            }

            #[inline]
            fn is_suffix_of(self, haystack: $ty) -> bool {
                is_suffix_of(MultiCharEq(self), &*haystack)
            }

            #[inline]
            fn trim_end(&mut self, haystack: &mut $ty) {
                trim_end(&mut MultiCharEq(*self), haystack)
            }
        }

        unsafe impl $($gen_f)* Searcher<$ty> for MultiCharSearcher<F>
        where
            F: FnMut(char) -> bool,
        {
            #[inline]
            fn search(&mut self, haystack: &mut $ty) -> ($ty, Option<$ty>) {
                search(&mut self.predicate, haystack)
            }
        }

        unsafe impl $($gen_f)* ReverseSearcher<$ty> for MultiCharSearcher<F>
        where
            F: FnMut(char) -> bool,
        {
            #[inline]
            fn rsearch(&mut self, haystack: &mut $ty) -> (Option<$ty>, $ty) {
                rsearch(&mut self.predicate, haystack)
            }
        }

        unsafe impl $($gen_f)* DoubleEndedSearcher<$ty> for MultiCharSearcher<F>
        where
            F: FnMut(char) -> bool,
        {}
    }
}

impl_pattern_for_str_like!([<'h, F>] [<'h, 'p>] &'h str);
impl_pattern_for_str_like!([<'h, F>] [<'h, 'p>] &'h mut str);
