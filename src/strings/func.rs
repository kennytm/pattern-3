use pattern::{Pattern, Searcher, ReverseSearcher, DoubleEndedSearcher};
use std::ops::Range;

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

unsafe impl<F> Searcher for MultiCharSearcher<F>
where
    F: FnMut(char) -> bool,
{
    type Hay = str;

    #[inline]
    fn search(&mut self, h: &str) -> Option<Range<usize>> {
        let mut chars = h.chars();
        let c = chars.find(|c| (self.predicate)(*c))?;
        let end = chars.as_str().as_ptr();
        let end = unsafe { end.offset_from(h.as_ptr()) as usize };
        // let start = unsafe { end.sub(c.len_utf8()) };
        Some((end - c.len_utf8())..end)
    }
}

unsafe impl<F> ReverseSearcher for MultiCharSearcher<F>
where
    F: FnMut(char) -> bool,
{
    #[inline]
    fn rsearch(&mut self, h: &str) -> Option<Range<usize>> {
        let mut chars = h.chars();
        let c = chars.rfind(|c| (self.predicate)(*c))?;
        let string = chars.as_str();
        let start = unsafe { string.as_ptr().offset_from(h.as_ptr()) as usize };
        let start = start + string.len();
        Some(start..(start + c.len_utf8()))

        // unsafe {
        //     let start = string.as_ptr().add(string.len());
        //     let end = start.add(c.len_utf8());
        //     Some(start..end)
        // }
    }
}

unsafe impl<F> DoubleEndedSearcher for MultiCharSearcher<F>
where
    F: FnMut(char) -> bool,
{}

impl<F> Pattern<str> for F
where
    F: FnMut(char) -> bool,
{
    type Searcher = MultiCharSearcher<F>;

    #[inline]
    fn into_searcher(self) -> Self::Searcher {
        MultiCharSearcher { predicate: self }
    }

    #[inline]
    fn is_prefix_of(mut self, rest: &str) -> bool {
        if let Some(c) = rest.chars().next() {
            self(c)
        } else {
            false
        }
    }

    #[inline]
    fn is_suffix_of(mut self, rest: &str) -> bool {
        if let Some(c) = rest.chars().next_back() {
            self(c)
        } else {
            false
        }
    }

    #[inline]
    fn trim_start(&mut self, rest: &str) -> usize {
        let mut chars = rest.chars();
        let unconsume_amount = chars
            .find_map(|c| if !self(c) { Some(c.len_utf8()) } else { None })
            .unwrap_or(0);
        let consumed = unsafe { chars.as_str().as_ptr().offset_from(rest.as_ptr()) as usize };
        consumed - unconsume_amount
        // unsafe { chars.as_str().as_ptr().sub(unconsume_amount) }
    }

    #[inline]
    fn trim_end(&mut self, rest: &str) -> usize {
        // `find.map_or` is faster in trim_end in the microbenchmark, while
        // `find.unwrap_or` is faster in trim_start. Don't ask me why.
        let mut chars = rest.chars();
        let unconsume_amount = chars
            .by_ref()
            .rev() // btw, `rev().find()` is faster than `rfind()`
            .find(|c| !self(*c))
            .map_or(0, |c| c.len_utf8());
        chars.as_str().len() + unconsume_amount
    }
}

impl<'p> Pattern<str> for &'p [char] {
    type Searcher = MultiCharSearcher<MultiCharEq<'p>>;

    #[inline]
    fn into_searcher(self) -> Self::Searcher {
        MultiCharSearcher { predicate: MultiCharEq(self) }
    }

    #[inline]
    fn is_prefix_of(self, rest: &str) -> bool {
        MultiCharEq(self).is_prefix_of(rest)
    }

    #[inline]
    fn is_suffix_of(self, rest: &str) -> bool {
        MultiCharEq(self).is_suffix_of(rest)
    }

    #[inline]
    fn trim_start(&mut self, rest: &str) -> usize {
        MultiCharEq(*self).trim_start(rest)
    }

    #[inline]
    fn trim_end(&mut self, rest: &str) -> usize {
        MultiCharEq(*self).trim_end(rest)
    }
}
