use haystack::{Haystack, HaystackMut};
use cursor::{RawCursor, SharedOrigin, MutOrigin};
use std::slice;
use std::mem::{size_of, align_of};
use span::Span;

impl<T> Haystack for [T] {
    type Cursor = *const T;
    type Origin = *const T;

    #[inline]
    fn as_origin_raw(&self) -> Self::Origin {
        self.as_ptr()
    }

    #[inline]
    fn as_span_raw(&self) -> (Self::Cursor, Self::Cursor) {
        if size_of::<T>() != 0 {
            let start = self.as_ptr();
            let end = unsafe { start.add(self.len()) };
            (start, end)
        } else {
            (0 as *const T, self.len() as *const T)
        }
    }

    #[inline]
    unsafe fn from_span_raw<'h>(
        origin: Self::Origin,
        start: Self::Cursor,
        end: Self::Cursor,
    ) -> &'h Self {
        let (origin, len) = if size_of::<T>() != 0 {
            (start, end.offset_from(start) as usize)
        } else {
            (origin, (end as usize) - (start as usize))
        };
        slice::from_raw_parts(origin, len)
    }
}

impl<T> HaystackMut for [T] {
    #[inline]
    unsafe fn from_span_raw_mut<'h>(
        origin: Self::Origin,
        start: Self::Cursor,
        end: Self::Cursor,
    ) -> &'h mut Self {
        let (origin, len) = if size_of::<T>() != 0 {
            (start, end.offset_from(start) as usize)
        } else {
            (origin, (end as usize) - (start as usize))
        };
        slice::from_raw_parts_mut(origin as *mut T, len)
    }
}

impl<T> RawCursor<[T]> for *const T {
    unsafe fn to_index(self, origin: *const T) -> usize {
        if size_of::<T>() != 0 {
            self.offset_from(origin) as usize
        } else {
            self as usize
        }
    }
}

impl<'h, T> Span<'h, [T]> {
    fn as_slice(&self) -> &[T] {
        unsafe {
            let origin = align_of::<T>() as *const T;
            let start = self.start().raw();
            let end = self.end().raw();
            <[T]>::from_span_raw(origin, start, end)
        }
    }
}

mod func;
