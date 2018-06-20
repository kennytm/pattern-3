//! [Haystack] traits.

use std::ops::Range;
use std::fmt::Debug;
use std::borrow::Borrow;

pub trait Hay {
    type StartCursor;
    type EndCursor;
}

pub trait Haystack: Borrow<<Self as Haystack>::Hay> + Default {
    type Hay: Hay + ?Sized;

    unsafe fn split_around_unchecked(
        self,
        start: <Self::Hay as Hay>::StartCursor,
        end: <Self::Hay as Hay>::EndCursor,
    ) -> (Self, Self, Self);

    unsafe fn trim_start_unchecked(self, start: <Self::Hay as Hay>::StartCursor) -> Self;
    unsafe fn trim_end_unchecked(self, end: <Self::Hay as Hay>::EndCursor) -> Self;
}

pub trait IndexHaystack: Haystack {
    type Index;
    type Origin: Copy + Debug;

    /// Obtains the origin of this haystack.
    fn origin(&self) -> Self::Origin;

    unsafe fn range_from_origin(&self, origin: Self::Origin) -> Range<Self::Index>;
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
