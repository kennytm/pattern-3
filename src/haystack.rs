//! [Haystack] traits.

use std::ops::Range;
use std::fmt::Debug;
use std::borrow::Borrow;
use std::mem;

pub trait Hay {
    type Index: Copy + Debug + Eq;

    fn empty<'a>() -> &'a Self;

    fn add_len(&self, index: Self::Index) -> Self::Index;

    fn start_index(&self) -> Self::Index;

    fn end_index(&self) -> Self::Index;

    fn validate_range(&self, range: Range<Self::Index>);

    unsafe fn slice_unchecked(&self, range: Range<Self::Index>) -> &Self;
}

pub trait Haystack: Borrow<<Self as Haystack>::Hay> + Sized {
    type Hay: Hay + ?Sized;
    type Span: Span<Haystack = Self> + From<Self> /*+ Into<Self>*/ = UniqueSpan<Self>;

    fn empty() -> Self;

    unsafe fn split_around(self, range: Range<<Self::Hay as Hay>::Index>) -> [Self; 3];

    unsafe fn slice_unchecked(self, range: Range<<Self::Hay as Hay>::Index>) -> Self;
}

#[derive(Debug)]
pub struct SharedSpan<'a, Y: Hay + ?Sized + 'a> {
    haystack: &'a Y,
    range: Range<Y::Index>,
}

#[derive(Debug, Clone)]
pub struct UniqueSpan<H: Haystack> {
    haystack: H,
    offset: <H::Hay as Hay>::Index,
}

impl<'a, Y: Hay + ?Sized + 'a> Clone for SharedSpan<'a, Y> {
    fn clone(&self) -> Self {
        Self {
            haystack: self.haystack,
            range: self.range.clone(),
        }
    }
}

// mod sealed {
//     pub trait Sealed {}
// }

// impl<'a, Y: Hay + ?Sized + 'a> sealed::Sealed for SharedSpan<'a, Y> {}
// impl<H: Haystack> sealed::Sealed for UniqueSpan<H> {}

impl<'a, Y: Hay + ?Sized + 'a> From<&'a Y> for SharedSpan<'a, Y> {
    #[inline]
    fn from(haystack: &'a Y) -> Self {
        let range = haystack.start_index()..haystack.end_index();
        Self { haystack, range }
    }
}

impl<H: Haystack> From<H> for UniqueSpan<H> {
    #[inline]
    fn from(haystack: H) -> Self {
        let offset = haystack.borrow().start_index();
        Self { haystack, offset }
    }
}

pub trait Span: From<<Self as Span>::Haystack> {
    type Haystack: Haystack;

    fn original_range(&self) -> Range<<<Self::Haystack as Haystack>::Hay as Hay>::Index>;

    fn borrow(&self) -> SharedSpan<'_, <Self::Haystack as Haystack>::Hay>;

    fn is_empty(&self) -> bool;

    #[inline]
    fn take(&mut self) -> Self {
        mem::replace(self, Self::Haystack::empty().into())
    }

    // FIXME: This should be changed to an `impl From<Self> for Self::Haystack`.
    fn into(self) -> Self::Haystack;

    fn split_around(self, subrange: Range<<<Self::Haystack as Haystack>::Hay as Hay>::Index>) -> [Self; 3];
}

impl<'a, Y: Hay + ?Sized + 'a> SharedSpan<'a, Y> {
    pub fn into_parts(self) -> (&'a Y, Range<Y::Index>) {
        (self.haystack, self.range)
    }
}

impl<'a, Y: Hay + ?Sized + 'a> Span for SharedSpan<'a, Y> {
    type Haystack = &'a Y;

    #[inline]
    fn original_range(&self) -> Range<Y::Index> {
        self.range.clone()
    }

    #[inline]
    fn borrow(&self) -> SharedSpan<'_, Y> {
        self.clone()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.range.start == self.range.end
    }

    #[inline]
    fn into(self) -> Self::Haystack {
        unsafe {
            self.haystack.slice_unchecked(self.range)
        }
    }

    #[inline]
    fn split_around(self, subrange: Range<<<Self::Haystack as Haystack>::Hay as Hay>::Index>) -> [Self; 3] {
        let haystack = self.haystack;
        haystack.borrow().validate_range(subrange.clone());
        [
            Self { haystack, range: self.range.start..subrange.start },
            Self { haystack, range: subrange.clone() },
            Self { haystack, range: subrange.end..self.range.end },
        ]
    }
}

impl<H: Haystack> Span for UniqueSpan<H> {
    type Haystack = H;

    #[inline]
    fn original_range(&self) -> Range<<H::Hay as Hay>::Index> {
        let end = self.haystack.borrow().add_len(self.offset);
        self.offset..end
    }

    #[inline]
    fn borrow(&self) -> SharedSpan<'_, H::Hay> {
        self.haystack.borrow().into()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        let hay = self.haystack.borrow();
        hay.start_index() == hay.end_index()
    }

    #[inline]
    fn into(self) -> Self::Haystack {
        self.haystack
    }

    #[inline]
    fn split_around(self, subrange: Range<<<Self::Haystack as Haystack>::Hay as Hay>::Index>) -> [Self; 3] {
        self.haystack.borrow().validate_range(subrange.clone());
        let [left, middle, right] = unsafe { self.haystack.split_around(subrange.clone()) };
        let left_offset = self.offset;
        let middle_offset = left.borrow().add_len(left_offset);
        let right_offset = middle.borrow().add_len(middle_offset);
        [
            Self { haystack: left, offset: left_offset },
            Self { haystack: middle, offset: middle_offset },
            Self { haystack: right, offset: right_offset },
        ]
    }
}

impl<'a, Y: Hay + ?Sized + 'a> Haystack for &'a Y {
    type Hay = Y;
    type Span = SharedSpan<'a, Y>;

    #[inline]
    fn empty() -> Self {
        Y::empty()
    }

    #[inline]
    unsafe fn split_around(self, range: Range<Y::Index>) -> [Self; 3] {
        [
            self.slice_unchecked(self.start_index()..range.start),
            self.slice_unchecked(range.clone()),
            self.slice_unchecked(range.end..self.end_index()),
        ]
    }

    #[inline]
    unsafe fn slice_unchecked(self, range: Range<Y::Index>) -> Self {
        Y::slice_unchecked(self, range)
    }
}

// /// Haystacks are searchable linear data structures like strings and slices.
// ///
// /// A haystack can be modeled as a list of *units*. In a slice `[T]` the unit is
// /// an element `T` and in a string `str` the unit is a byte `u8`.
// ///
// /// All haystacks support *slicing*, extracting a smaller haystack out of an
// /// existing haystack, e.g. in the string `"hello_world"`, we could slice from
// /// byte 4 to byte 8 to get a substring `"o_wo"`.
// ///
// /// ```text
// /// 0   1   2   3   4   5   6   7   8   9  10  11
// /// +---+---+---+---+---+---+---+---+---+---+---+
// /// | h | e | l | l | o | _ | w | o | r | l | d |    &string
// /// +---+---+---+---+---+---+---+---+---+---+---+
// ///
// ///                 |               |
// ///                 v               v
// ///
// ///                 +---+---+---+---+
// ///                 | o | _ | w | o |    &string[4..8]
// ///                 +---+---+---+---+
// /// ```
// pub trait Haystack: Sized {
//     fn is_empty(&self) -> bool;

//     fn collapse_to_end(&mut self) -> Self;

//     fn collapse_to_start(&mut self) -> Self;

//     // fn empty_at_start(&self) -> Self;

//     // fn empty_at_end(&self) -> Self;

//     // fn consume_first(&mut self) -> Option<Self>;

//     // fn consume_last(&mut self) -> Option<Self>;
// }
