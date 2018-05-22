use traits::{Haystack, HaystackMut, Pattern, Rev, Searcher};
use span::Span;
use std::ops::Range;
use std::slice;

impl<T> Haystack for [T] {
    type Item = T;

    fn start_cursor_at_front(&self) -> *const T {
        self.as_ptr()
    }
    fn end_cursor_at_back(&self) -> *const T {
        unsafe { self.as_ptr().add(self.len()) }
    }

    unsafe fn start_cursor_to_offset(&self, cur: *const T) -> usize {
        cur.offset_from(self.as_ptr()) as usize
    }

    unsafe fn end_cursor_to_offset(&self, cur: *const T) -> usize {
        cur.offset_from(self.as_ptr()) as usize
    }

    unsafe fn from_cursor_range<'a>(range: Range<*const T>) -> &'a Self {
        let len = range.end.offset_from(range.start) as usize;
        slice::from_raw_parts(range.start, len)
    }

    unsafe fn start_to_end_cursor(&self, start_cur: *const T) -> *const T {
        start_cur
    }

    unsafe fn end_to_start_cursor(&self, end_cur: *const T) -> *const T {
        end_cur
    }
}

impl<T> HaystackMut for [T] {
    unsafe fn from_cursor_range_mut<'a>(range: Range<*mut T>) -> &'a mut Self {
        let len = range.end.offset_from(range.start) as usize;
        slice::from_raw_parts_mut(range.start, len)
    }
}


impl<'h, F, T> Pattern<'h, [T]> for F
where
    T: 'h,
    F: FnMut(&'h T) -> bool,
{
    type Searcher = ElemSearcher<'h, T, F>;

    fn into_searcher(self, haystack: &'h [T]) -> Self::Searcher {
        ElemSearcher {
            remaining: Span::full(haystack),
            predicate: self,
        }
    }
}



pub struct ElemSearcher<'h, T, F>
where
    T: 'h,
    F: FnMut(&'h T) -> bool,
{
    remaining: Span<'h, [T]>,
    predicate: F,
}

impl<'h, T, F> ElemSearcher<'h, T, F>
where
    T: 'h,
    F: FnMut(&'h T) -> bool,
{
    #[inline]
    fn next_span(&mut self, forward: bool) -> Option<Span<'h, [T]>> {
        while !self.remaining.is_empty() {
            unsafe {
                let (start, end) = if forward {
                    let start = self.remaining.start_cursor();
                    self.remaining.set_start_cursor(start.add(1));
                    (start, self.remaining.start_cursor())
                } else {
                    let end = self.remaining.end_cursor();
                    self.remaining.set_end_cursor(end.sub(1));
                    (self.remaining.end_cursor(), end)
                };
                if (self.predicate)(&*start) {
                    return Some(self.remaining.with_cursor_range(start..end));
                }
            }
        }
        None
    }
}

impl<'h, T, F> Iterator for ElemSearcher<'h, T, F>
where
    T: 'h,
    F: FnMut(&'h T) -> bool,
{
    type Item = Span<'h, [T]>;
    fn next(&mut self) -> Option<Self::Item> {
        self.next_span(true)
    }
}

impl<'h, T, F> DoubleEndedIterator for ElemSearcher<'h, T, F>
where
    T: 'h,
    F: FnMut(&'h T) -> bool,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.next_span(false)
    }
}

unsafe impl<'h, T, F> Searcher<'h, [T]> for ElemSearcher<'h, T, F>
where
    T: 'h,
    F: FnMut(&'h T) -> bool,
{
    fn next_immediate(&mut self) -> Option<Span<'h, [T]>> {
        if self.remaining.is_empty() {
            return None;
        }
        unsafe {
            let start = self.remaining.start_cursor();
            if !(self.predicate)(&*start) {
                return None;
            }
            self.remaining.add_start_cursor(1);
            Some(self.remaining.with_cursor_range(start..self.remaining.start_cursor()))
        }
    }
}

unsafe impl<'h, T, F> Searcher<'h, [T]> for Rev<ElemSearcher<'h, T, F>>
where
    T: 'h,
    F: FnMut(&'h T) -> bool,
{
    fn next_immediate(&mut self) -> Option<Span<'h, [T]>> {
        let this = &mut self.inner;
        if this.remaining.is_empty() {
            return None;
        }
        unsafe {
            let end = this.remaining.end_cursor();
            let new_end = end.sub(1);
            if !(this.predicate)(&*new_end) {
                return None;
            }
            this.remaining.set_end_cursor(new_end);
            Some(this.remaining.with_cursor_range(new_end..end))
        }
    }
}
