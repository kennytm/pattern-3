use traits::{Haystack, HaystackMut};
use std::{fmt, ptr, slice};
use std::ops::Range;

pub struct Span<H>
where
    H: Haystack + ?Sized,
{
    // unfortunately Range isn't Copy :(
    haystack_start: *const H::Item,
    haystack_end: *const H::Item,
    start: *const H::Item,
    end: *const H::Item,
}

impl<H: Haystack + ?Sized> Clone for Span<H> {
    fn clone(&self) -> Self { *self }
}
impl<H: Haystack + ?Sized> Copy for Span<H> {
}

impl<H: Haystack + ?Sized> PartialEq for Span<H> {
    fn eq(&self, other: &Self) -> bool {
        self.haystack_start == other.haystack_start &&
            self.haystack_end == other.haystack_end &&
            self.start == other.start &&
            self.end == other.end
    }
}
impl<H: Haystack + ?Sized> Eq for Span<H> {}

impl<H: Haystack + ?Sized> fmt::Debug for Span<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Span")
            .field("haystack", &(self.haystack_start..self.haystack_end))
            .field("range", &(self.start..self.end))
            .finish()
    }
}

impl<H: Haystack + ?Sized> Span<H> {
    #[inline]
    pub fn full(haystack: &H) -> Self {
        let range = haystack.cursor_range();
        Span {
            haystack_start: range.start,
            haystack_end: range.end,
            start: range.start,
            end: range.end,
        }
    }

    #[inline]
    pub unsafe fn with_range_unchecked(&self, range: Range<*const H::Item>) -> Self {
        Span {
            haystack_start: self.haystack_start,
            haystack_end: self.haystack_end,
            start: range.start,
            end: range.end,
        }
    }

    pub fn with_range(&self, range: Range<*const H::Item>) -> Option<Self> {
        if self.haystack_start <= range.start && range.start <= range.end && range.end <= self.haystack_end {
            Some(unsafe { self.with_range_unchecked(range) })
        } else {
            None
        }
    }

    pub fn to_offset(&self) -> usize {
        unsafe {
            H::start_cursor_to_offset(self.haystack_start..self.haystack_end, self.start)
        }
    }

    pub fn to_offset_range(&self) -> Range<usize> {
        unsafe {
            let haystack = self.haystack_start..self.haystack_end;
            let start = H::start_cursor_to_offset(haystack.clone(), self.start);
            let end = H::end_cursor_to_offset(haystack, self.end);
            start..end
        }
    }

    pub fn range(&self) -> Range<*const H::Item> {
        self.start..self.end
    }

    pub fn start(&self) -> *const H::Item {
        self.start
    }

    pub fn end(&self) -> *const H::Item {
        self.end
    }

    pub unsafe fn split_off_from_start_unchecked(&mut self, count: usize) -> Span<H> {
        let haystack = self.haystack_start..self.haystack_end;
        let old_start = self.start;
        self.start = self.start.add(count);
        let old_end = H::start_to_end_cursor(haystack, self.start);
        self.with_range_unchecked(old_start..old_end)
    }

    pub unsafe fn inc_start(&mut self, count: usize) {
        self.start = self.start.add(count);
    }

    pub unsafe fn dec_end(&mut self, count: usize) {
        self.end = self.end.sub(count);
    }

    pub fn to_haystack(&self) -> &'a H {
        unsafe {
            H::from_cursor_range(self.start..self.end)
        }
    }

    pub fn to_slice(&self) -> &'a [H::Item] {
        unsafe {
            let len = self.end.offset_from(self.start) as usize;
            slice::from_raw_parts(self.start, len)
        }
    }

    pub fn is_empty(&self) -> bool {
        unsafe {
            self.haystack.start_to_end_cursor(self.start) == self.end
        }
    }

    /// `(a..b).split(c..d) == (a..c, d..b)`
    pub fn split(&self, other: &Span<'a, H>) -> (Span<'a, H>, Span<'a, H>) {
        debug_assert!(ptr::eq(self.haystack, other.haystack));
        debug_assert!(self.start <= other.start);
        debug_assert!(other.end <= self.end);

        unsafe {
            let left_end = self.haystack.start_to_end_cursor(other.start);
            let right_start = self.haystack.end_to_start_cursor(other.end);
            (
                self.with_cursor_range(self.start..left_end),
                self.with_cursor_range(right_start..self.end),
            )
        }
    }

    fn is_followed_by(&self, other: &Span<'a, H>) -> bool {
        debug_assert!(ptr::eq(self.haystack, other.haystack));
        unsafe {
            self.end == other.haystack.start_to_end_cursor(other.start)
        }
    }

    pub fn collapse_to_end(&mut self) {
        self.start = unsafe { self.haystack.end_to_start_cursor(self.start) };
    }

    fn collapsed_to_start(mut self) -> Self {
        unsafe {
            self.end = self.haystack.start_to_end_cursor(self.start);
        }
        self
    }

    fn collapsed_to_end(mut self) -> Self {
        unsafe {
            self.start = self.haystack.end_to_start_cursor(self.end);
        }
        self
    }

    pub fn gap_between(&self, other: &Span<'a, H>) -> Span<'a, H> {
        debug_assert!(ptr::eq(self.haystack, other.haystack));
        unsafe {
            let start = self.haystack.end_to_start_cursor(self.end);
            let end = other.haystack.start_to_end_cursor(other.start);
            debug_assert!(start <= other.start);
            self.with_cursor_range(start..end)
        }
    }

    /// If `self` and `other` has the same start cursor, mutate `self` to start
    /// from `other.end` instead, and return true. Otherwise, do nothing and
    /// return false.
    fn trim_start(&mut self, other: &Span<'a, H>) -> bool {
        debug_assert!(ptr::eq(self.haystack, other.haystack));
        if self.start != other.start {
            return false;
        }
        self.start = unsafe { other.haystack.end_to_start_cursor(other.end) };
        true
    }

    /// If `self` and `other` has the same end cursor, mutate `self` to end with
    /// `other.start` instead, and return true. Otherwise, do nothing and return
    /// false.
    fn trim_end(&mut self, other: &Span<'a, H>) -> bool {
        debug_assert!(ptr::eq(self.haystack, other.haystack));
        if self.end != other.end {
            return false;
        }
        self.end = unsafe { other.haystack.start_to_end_cursor(other.start) };
        true
    }
}

impl<'a, H: HaystackMut + ?Sized + 'a> Span<'a, H> {
    /// # Safety
    ///
    /// The original haystack must be already mutable, and the original haystack
    /// must no longer be accessible after this call.
    pub unsafe fn into_slice_mut(self) -> &'a mut H {
        let start = self.start as *mut H::Item;
        let end = self.end as *mut H::Item;
        H::from_cursor_range_mut(start..end)
    }
}
