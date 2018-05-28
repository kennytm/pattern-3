use pattern::{Pattern, ReversePattern};
use cursor::Cursor;
use span::Span;

use std::iter::{Rev, FusedIterator};

unsafe impl<'h, F> Pattern<'h, str> for F
where
    F: FnMut(char) -> bool,
{
    type Searcher = MultiCharSearcher<'h, F>;

    #[inline]
    fn into_searcher(self, span: Span<'h, str>) -> Self::Searcher {
        MultiCharSearcher {
            predicate: self,
            remaining: span,
        }
    }

    #[inline]
    fn is_prefix_of(mut self, span: Span<'_, str>) -> bool {
        if let Some(ch) = span.as_str().chars().next() {
            self(ch)
        } else {
            false
        }
    }

    #[inline]
    fn trim_start(&mut self, span: &mut Span<'_, str>) {
        let mut chars = span.as_str().chars();
        let unconsume_amount = chars
            .find_map(|c| if !self(c) { Some(c.len_utf8()) } else { None })
            .unwrap_or(0);
        let start = unsafe { Cursor::new_unchecked(chars.as_str().as_ptr().sub(unconsume_amount)) };
        span.remove_start(start);
    }

}

unsafe impl<'h, F> ReversePattern<'h, str> for F
where
    F: FnMut(char) -> bool,
{
    type ReverseSearcher = Rev<Self::Searcher>;

    #[inline]
    fn into_reverse_searcher(self, span: Span<'h, str>) -> Self::ReverseSearcher {
        self.into_searcher(span).rev()
    }

    #[inline]
    fn is_suffix_of(mut self, span: Span<'_, str>) -> bool {
        if let Some(ch) = span.as_str().chars().next_back() {
            self(ch)
        } else {
            false
        }
    }

    // #[inline]
    // fn trim_end(&mut self, span: &mut Span<'h, str>) {
    //     let cloned = unsafe { span.clone() };
    //     if let Some(last_unmatch) = (|c| !self(c)).into_searcher(cloned).next_back() {
    //         span.remove_end(last_unmatch.end());
    //     } else {
    //         span.collapse_to_start();
    //     }
    // }


    #[inline]
    fn trim_end(&mut self, span: &mut Span<'_, str>) {
        // `find.map_or` is faster in trim_end, while
        // `find.unwrap_or` is faster in trim_start. Don't ask me why.
        let mut chars = span.as_str().chars();
        let unconsume_amount = chars
            .by_ref()
            .rev()
            .find(|c| !self(*c))
            .map_or(0, |c| c.len_utf8());
        let string = chars.as_str();
        let len = string.len() + unconsume_amount;
        let end = unsafe { Cursor::new_unchecked(string.as_ptr().add(len)) };
        span.remove_end(end);
    }
}

unsafe impl<'h, 'p> Pattern<'h, str> for &'p [char] {
    type Searcher = <MultiCharEq<'p> as Pattern<'h, str>>::Searcher;

    #[inline]
    fn into_searcher(self, span: Span<'h, str>) -> Self::Searcher {
        MultiCharEq(self).into_searcher(span)
    }

    #[inline]
    fn is_prefix_of(self, span: Span<'_, str>) -> bool {
        MultiCharEq(self).is_prefix_of(span)
    }

    #[inline]
    fn trim_start(&mut self, span: &mut Span<'_, str>) {
        MultiCharEq(*self).trim_start(span)
    }
}

unsafe impl<'h, 'p> ReversePattern<'h, str> for &'p [char] {
    type ReverseSearcher = Rev<Self::Searcher>;

    #[inline]
    fn into_reverse_searcher(self, span: Span<'h, str>) -> Self::ReverseSearcher {
        self.into_searcher(span).rev()
    }

    #[inline]
    fn is_suffix_of(self, span: Span<'_, str>) -> bool {
        MultiCharEq(self).is_suffix_of(span)
    }

    #[inline]
    fn trim_end(&mut self, span: &mut Span<'_, str>) {
        MultiCharEq(self).trim_end(span)
    }
}





pub struct MultiCharEq<'p>(&'p [char]);

impl_unboxed_functions! {
    [<'p>]
    MultiCharEq<'p> = |&self, c: char| -> bool {
        self.0.iter().any(|ch| *ch == c)
    }
}

pub struct MultiCharSearcher<'h, F>
where
    F: FnMut(char) -> bool,
{
    predicate: F,
    remaining: Span<'h, str>,
}

impl<'h, F> Iterator for MultiCharSearcher<'h, F>
where
    F: FnMut(char) -> bool,
{
    type Item = Span<'h, str>;

    #[inline]
    fn next(&mut self) -> Option<Span<'h, str>> {
        let mut chars = self.remaining.as_str().chars();
        if let Some(found) = chars.find(|c| (self.predicate)(*c)) {
            let end = chars.as_str().as_ptr();
            unsafe {
                let start = end.sub(found.len_utf8());
                Some(self.remaining.take_from_start_unchecked(start, end))
            }
        } else {
            self.remaining.collapse_to_end();
            None
        }
    }
}

impl<'h, F> FusedIterator for MultiCharSearcher<'h, F>
where
    F: FnMut(char) -> bool,
{}

impl<'h, F> DoubleEndedIterator for MultiCharSearcher<'h, F>
where
    F: FnMut(char) -> bool,
{
    #[inline]
    fn next_back(&mut self) -> Option<Span<'h, str>> {
        let mut chars = self.remaining.as_str().chars();
        if let Some(found) = chars.rfind(|c| (self.predicate)(*c)) {
            let string = chars.as_str();
            let len = string.len();
            unsafe {
                let start = string.as_ptr().add(len);
                let end = start.add(found.len_utf8());
                Some(self.remaining.take_from_end_unchecked(start, end))
            }
        } else {
            self.remaining.collapse_to_start();
            None
        }
    }
}
