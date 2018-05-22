#![feature(unboxed_closures, fn_traits, ptr_offset_from)]

extern crate pattern_3;

use pattern_3::{Pattern, Haystack, HaystackMut, Searcher, Rev, Span};

use std::ops::{Range, Deref};
use std::slice;
use std::mem::transmute;

pub struct S2<T>([T]);
pub struct S3<T>(T);

impl<T> Deref for S2<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Haystack for S2<T> {
    type Cursor = *const T;

    fn cursor_range(&self) -> Range<Self::Cursor> {
        let start = self.as_ptr();
        let end = unsafe { start.add(self.len()) };
        start..end
    }

    unsafe fn start_cursor_to_offset(&self, cur: Self::Cursor) -> usize {
        cur.offset_from(self.as_ptr()) as usize
    }

    unsafe fn end_cursor_to_offset(&self, cur: Self::Cursor) -> usize {
        cur.offset_from(self.as_ptr()) as usize
    }

    unsafe fn from_cursor_range<'a>(range: Range<Self::Cursor>) -> &'a Self {
        let len = range.end.offset_from(range.start) as usize;
        transmute(slice::from_raw_parts(range.start, len))
    }

    unsafe fn start_to_end_cursor(&self, start_cur: Self::Cursor) -> Self::Cursor {
        start_cur
    }

    unsafe fn end_to_start_cursor(&self, end_cur: Self::Cursor) -> Self::Cursor {
        end_cur
    }
}

impl<T> HaystackMut for S2<T> {
    unsafe fn from_cursor_range_mut<'a>(range: Range<Self::Cursor>) -> &'a mut Self {
        let len = range.end.offset_from(range.start) as usize;
        transmute(slice::from_raw_parts_mut(range.start as *mut T, len))
    }
}


impl<'h, 'p, T> Pattern<'h, S2<T>> for &'p S3<T>
where
    T: PartialEq + 'h + 'p,
{
    type Searcher = ElemSearcher<'h, T, ElemRefEq<'p, T>>;

    fn into_searcher(self, haystack: &'h S2<T>) -> Self::Searcher {
        ElemSearcher {
            remaining: Span::full(haystack),
            predicate: ElemRefEq { item: &self.0 },
        }
    }
}


pub struct ElemRefEq<'p, T: 'p> {
    item: &'p T,
}

impl<'h, 'p, T> FnOnce<(&'h T,)> for ElemRefEq<'p, T>
where
    T: PartialEq + 'h + 'p,
{
    type Output = bool;
    extern "rust-call" fn call_once(self, args: (&'h T,)) -> bool {
        self.item == args.0
    }
}
impl<'h, 'p, T> FnMut<(&'h T,)> for ElemRefEq<'p, T>
where
    T: PartialEq + 'h + 'p,
{
    extern "rust-call" fn call_mut(&mut self, args: (&'h T,)) -> bool {
        self.item == args.0
    }
}
impl<'h, 'p, T> Fn<(&'h T,)> for ElemRefEq<'p, T>
where
    T: PartialEq + 'h + 'p,
{
    extern "rust-call" fn call(&self, args: (&'h T,)) -> bool {
        self.item == args.0
    }
}


pub struct ElemSearcher<'h, T, F>
where
    T: 'h,
    F: FnMut(&'h T) -> bool,
{
    remaining: Span<'h, S2<T>>,
    predicate: F,
}

impl<'h, T, F> ElemSearcher<'h, T, F>
where
    T: 'h,
    F: FnMut(&'h T) -> bool,
{
    fn next_span(&mut self, forward: bool) -> Option<Span<'h, S2<T>>> {
        while !self.remaining.is_empty() {
            unsafe {
                let range = if forward {
                    let start = self.remaining.start_cursor();
                    self.remaining.set_start_cursor(start.add(1));
                    start..self.remaining.start_cursor()
                } else {
                    let end = self.remaining.end_cursor();
                    self.remaining.set_end_cursor(end.sub(1));
                    self.remaining.end_cursor()..end
                };
                if (self.predicate)(&*range.start) {
                    return Some(self.remaining.with_cursor_range(range));
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
    type Item = Span<'h, S2<T>>;
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

unsafe impl<'h, T, F> Searcher<'h, S2<T>> for ElemSearcher<'h, T, F>
where
    T: 'h,
    F: FnMut(&'h T) -> bool,
{
    fn next_immediate(&mut self) -> Option<Span<'h, S2<T>>> {
        if self.remaining.is_empty() {
            return None;
        }
        unsafe {
            let start = self.remaining.start_cursor();
            if !(self.predicate)(&*start) {
                return None;
            }
            let new_start = start.add(1);
            self.remaining.set_start_cursor(new_start);
            Some(self.remaining.with_cursor_range(start..new_start))
        }
    }
}

unsafe impl<'h, T, F> Searcher<'h, S2<T>> for Rev<ElemSearcher<'h, T, F>>
where
    T: 'h,
    F: FnMut(&'h T) -> bool,
{
    fn next_immediate(&mut self) -> Option<Span<'h, S2<T>>> {
        let this = &mut self.inner;
        if this.remaining.is_empty() {
            return None;
        }
        unsafe {
            let end = this.remaining.end_cursor();
            let new_end = end.sub(1);
            if !(this.predicate)(&*end) {
                return None;
            }
            this.remaining.set_end_cursor(new_end);
            Some(this.remaining.with_cursor_range(end..new_end))
        }
    }
}


fn main() {}