use traits::{Pattern, Haystack, Rev, Searcher};
use span::Span;
use std::ops::Range;

pub trait HaystackExt: Haystack {
    fn starts_with<'a, P>(&'a self, pattern: P) -> bool
    where
        P: Pattern<'a, Self>,
    {
        pattern.into_searcher(self).next_immediate().is_some()
    }

    fn ends_with<'a, P>(&'a self, pattern: P) -> bool
    where
        P: Pattern<'a, Self>,
        Rev<P::Searcher>: Searcher<'a, Self>,
    {
        Rev::new(pattern.into_searcher(self)).next_immediate().is_some()
    }

    fn contains<'a, P>(&'a self, pattern: P) -> bool
    where
        P: Pattern<'a, Self>,
    {
        pattern.into_searcher(self).next().is_some()
    }

    fn find<'a, P>(&'a self, pattern: P) -> Option<usize>
    where
        P: Pattern<'a, Self>,
    {
        pattern.into_searcher(self).next().map(|span| span.start_offset())
    }

    fn rfind<'a, P>(&'a self, pattern: P) -> Option<usize>
    where
        P: Pattern<'a, Self>,
        Rev<P::Searcher>: Searcher<'a, Self>,
    {
        Rev::new(pattern.into_searcher(self)).next().map(|span| span.start_offset())
    }

    fn find_range<'a, P>(&'a self, pattern: P) -> Option<Range<usize>>
    where
        P: Pattern<'a, Self>,
    {
        pattern.into_searcher(self).next().map(|span| span.offset_range())
    }

    fn rfind_range<'a, P>(&'a self, pattern: P) -> Option<Range<usize>>
    where
        P: Pattern<'a, Self>,
        Rev<P::Searcher>: Searcher<'a, Self>,
    {
        Rev::new(pattern.into_searcher(self)).next().map(|span| span.offset_range())
    }

    fn trim_left_matches<'a, P>(&'a self, pattern: P) -> &'a Self
    where
        P: Pattern<'a, Self>,
    {
        let full_span = Span::full(self);
        let mut searcher = pattern.into_searcher(self);
        let mut last_span = None;
        while let span @ Some(_) = searcher.next_immediate() {
            last_span = span;
        }
        if let Some(m) = last_span {
            full_span.split(&m).1.to_haystack()
        } else {
            self
        }
    }

    fn trim_right_matches<'a, P>(&'a self, pattern: P) -> &'a Self
    where
        P: Pattern<'a, Self>,
        Rev<P::Searcher>: Searcher<'a, Self>,
    {
        let full_span = Span::full(self);
        let mut searcher = Rev::new(pattern.into_searcher(self));
        let mut last_span = None;
        while let span @ Some(_) = searcher.next_immediate() {
            last_span = span;
        }
        if let Some(m) = last_span {
            full_span.split(&m).0.to_haystack()
        } else {
            self
        }
    }

    fn trim_matches<'a, P>(&'a self, pattern: P) -> &'a Self
    where
        P: Pattern<'a, Self>,
        P::Searcher: DoubleEndedIterator,
        Rev<P::Searcher>: Searcher<'a, Self>,
    {
        let full_span = Span::full(self);
        let mut searcher = pattern.into_searcher(self);
        let mut left_span = None;
        while let span @ Some(_) = searcher.next_immediate() {
            left_span = span;
        }
        let mut searcher = Rev::new(searcher);
        let mut right_span = None;
        while let span @ Some(_) = searcher.next_immediate() {
            right_span = span;
        }

        match (left_span, right_span) {
            (None, None) => return self,
            (Some(l), None) => full_span.split(&l).1,
            (None, Some(r)) => full_span.split(&r).0,
            (Some(l), Some(r)) => l.gap_between(&r),
        }.to_haystack()
    }
}

impl<H: Haystack + ?Sized> HaystackExt for H {}
