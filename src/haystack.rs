//! [Haystack] traits.

use cursor::RawCursor;

/// Haystacks are searchable linear data structures like strings and slices.
///
/// A haystack can be modeled as a list of *units*. In a slice `[T]` the unit is
/// an element `T` and in a string `str` the unit is a byte `u8`.
///
/// All haystacks support *slicing*, extracting a smaller haystack out of an
/// existing haystack, e.g. in the string `"hello_world"`, we could slice from
/// byte 4 to byte 8 to get a substring `"o_wo"`.
///
/// ```text
/// 0   1   2   3   4   5   6   7   8   9  10  11
/// +---+---+---+---+---+---+---+---+---+---+---+
/// | h | e | l | l | o | _ | w | o | r | l | d |    &string
/// +---+---+---+---+---+---+---+---+---+---+---+
///
///                 |               |
///                 v               v
///
///                 +---+---+---+---+
///                 | o | _ | w | o |    &string[4..8]
///                 +---+---+---+---+
/// ```
pub trait Haystack {
    /// The type of a non-validated cursor used to address units in the
    /// haystack.
    ///
    /// See the documentation of the [`RawCursor`] trait for details.
    type Cursor: RawCursor<Self>;

    /// Auxiliary data which helps converting a cursor to an index, and a span
    /// to a real haystack.
    ///
    /// This is usually a pointer, which points to the start of the haystack.
    type Origin: Copy;

    /// Obtains the origin of this haystack.
    fn as_origin_raw(&self) -> Self::Origin;

    /// Obtains the start and end cursors of this haystack.
    fn as_span_raw(&self) -> (Self::Cursor, Self::Cursor);

    /// Recovers a haystack from a span.
    unsafe fn from_span_raw<'h>(
        origin: Self::Origin,
        start: Self::Cursor,
        end: Self::Cursor,
    ) -> &'h Self;
}

/// Haystacks where the content can be mutable.
pub trait HaystackMut: Haystack {
    /// Recovers a mutable haystack from a span.
    unsafe fn from_span_raw_mut<'h>(
        origin: Self::Origin,
        start: Self::Cursor,
        end: Self::Cursor,
    ) -> &'h mut Self;
}
