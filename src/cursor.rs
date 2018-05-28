use std::fmt;
use std::cmp::Ordering;
use std::marker::PhantomData;

use haystack::{Haystack, HaystackMut};

/// Raw cursor to [Haystack] units.
///
/// In a haystack, a unit represents the smallest addressable item, but not
/// every unit can be sliced individually. For instance, a `str` must be UTF-8
/// encoded, thus a non-ASCII byte like `\xF0` cannot appear on its own, but
/// must appear together in a four-byte sequence like `\xF0\x9F\xA4\xA3`.
///
/// ```text
/// 0    1    2    3    4    5    6    7    8    9   10
/// +----+----+----+----+----+----+----+----+----+----+
/// | 31 | 32 | 33 | F0   9F   A4   A3 | 34 | 35 | 36 |    &string
/// +----+----+----+----+----+----+----+----+----+----+
///
///                |    |
///                v    v
///
///                x----x
///                | F0       &string[3..4]    // an invalid haystack
///                x----x
/// ```
///
/// These inseparable units together is called a *sequence*. In a slice `[T]`,
/// every single unit is a sequence, but in a string `str` a sequence may
/// consist of 1 to 4 units.
///
/// A *cursor* points to locations between each unit. Cursors can be thought of
/// generalized pointers. The locations within a sequence is called *interior*,
/// and the other locations are on the *sequence boundaries*. Valid cursors
/// should only point to the boundaries.
///
/// This trait shall only be implemented for valid (sequence boundary) cursors.
///
/// ```text
/// 0    1    2    3    4    5    6    7    8    9   10
/// +----+----+----+----+----+----+----+----+----+----+
/// | 31 | 32 | 33 | F0   9F   A4   A3 | 34 | 35 | 36 |
/// +----+----+----+----+----+----+----+----+----+----+
/// B    B    B    B    I    I    I    B    B    B    B
///
///                                             B = Boundary
///                                             I = Interior
/// ```
///
/// # Index
///
/// Cursors used in pattern matching are often implemented as raw pointers to
/// achieve maximum performance. However raw pointers are not meant to be
/// consumed by API users, since they are unsafe and not friendly to create.
///
/// When using strings and slices, we use integers to subslice these types e.g.
/// `&s[3..5]`. The numbers 3 and 5 are the *index* of the corresponding
/// cursors.
///
/// The index is usually the distance from the start of the haystack to the
/// current cursor.
// FIXME: Do we want to support index which is not a usize?
pub trait RawCursor<H>: Copy + Ord + fmt::Debug
where
    H: Haystack + ?Sized,
{
    /// Converts the cursor to an index.
    unsafe fn to_index(self, origin: H::Origin) -> usize;
}

/// Trait for safe origin wrappers.
///
/// A raw haystack origin has no associated lifetime, and thus all APIs
/// involving these raw values are unsafe. This safe wrapper ensures the origin
/// is tied to the origina haystack in terms of lifetime, and thus we can have a
/// memory-safe API.
pub trait Origin<'h, H>: Copy
where
    H: Haystack + ?Sized,
{
    /// Obtains the raw origin.
    fn raw(self) -> H::Origin;
}

macro_rules! declare_safe_origin {
    ($(
        $(#[$attr:meta])*
        pub struct $name:ident for ($reference:ty) with $haystack:ident;
    )+) => {$(
        $(#[$attr])*
        pub struct $name<'h, H>
        where
            H: $haystack + ?Sized + 'h,
        {
            raw: H::Origin,
            _marker: PhantomData<$reference>,
        }

        impl<'h, H> Clone for $name<'h, H>
        where
            H: $haystack + ?Sized,
        {
            fn clone(&self) -> Self {
                *self
            }
        }

        impl<'h, H> Copy for $name<'h, H>
        where
            H: $haystack + ?Sized,
        {}

        impl<'h, H> Origin<'h, H> for $name<'h, H>
        where
            H: $haystack + ?Sized,
        {
            #[inline]
            fn raw(self) -> H::Origin {
                self.raw
            }
        }

        impl<'h, H> fmt::Debug for $name<'h, H>
        where
            H: $haystack + ?Sized,
        {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str(stringify!($name))
            }
        }

        impl<'h, H> $name<'h, H>
        where
            H: $haystack + ?Sized + 'h,
        {
            pub(crate) fn new(haystack: $reference) -> Self {
                Self {
                    raw: haystack.as_origin_raw(),
                    _marker: PhantomData,
                }
            }

            pub unsafe fn new_unchecked(raw: H::Origin) -> Self {
                Self {
                    raw,
                    _marker: PhantomData,
                }
            }
        }
    )+}
}

declare_safe_origin! {
    /// A safe origin wrapper for shared haystack (`&H`).
    ///
    /// An instance of `SharedOrigin` can be created using the
    /// [`Span::and_origin`](crate::span::Span::and_origin) method.
    pub struct SharedOrigin for (&'h H) with Haystack;

    /// A safe origin wrapper for mutable haystack (`&mut H`).
    ///
    /// An instance of `MutOrigin` can be created using the
    /// [`Span::and_origin_mut`](crate::span::Span::and_origin_mut) method.
    pub struct MutOrigin for (&'h mut H) with HaystackMut;
}

/// Checked cursor to [Haystack] units.
///
/// This structure is a thin wrapper around a [RawCursor] instant which ensures
/// the cursor is valid:
///
/// 1. it can only point within haystack it was generated.
/// 2. it can only point to sequence boundaries.
pub struct Cursor<'h, H>
where
    H: Haystack + ?Sized,
{
    raw: H::Cursor,
    _marker: PhantomData<&'h ()>,
}

impl<'h, H> Clone for Cursor<'h, H>
where
    H: Haystack + ?Sized,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'h, H> fmt::Debug for Cursor<'h, H>
where
    H: Haystack + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.raw.fmt(f)
    }
}

impl<'h, H> PartialEq for Cursor<'h, H>
where
    H: Haystack + ?Sized,
{
    fn eq(&self, other: &Self) -> bool { self.raw == other.raw }
    fn ne(&self, other: &Self) -> bool { self.raw != other.raw }
}

impl<'h, H> PartialOrd for Cursor<'h, H>
where
    H: Haystack + ?Sized,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.raw.partial_cmp(&other.raw)
    }
    fn lt(&self, other: &Self) -> bool { self.raw < other.raw }
    fn le(&self, other: &Self) -> bool { self.raw <= other.raw }
    fn gt(&self, other: &Self) -> bool { self.raw > other.raw }
    fn ge(&self, other: &Self) -> bool { self.raw >= other.raw }
}

impl<'h, H> Ord for Cursor<'h, H>
where
    H: Haystack + ?Sized,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.raw.cmp(&other.raw)
    }
}

impl<'h, H> Eq for Cursor<'h, H>
where
    H: Haystack + ?Sized,
{}

impl<'h, H> Copy for Cursor<'h, H>
where
    H: Haystack + ?Sized,
{}


impl<'h, H> Cursor<'h, H>
where
    H: Haystack + ?Sized,
{
    /// Creates a new cursor from a raw cursor without checking.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the raw cursor originates from the haystack
    /// `&'h H`.
    #[inline]
    pub unsafe fn new_unchecked(raw: H::Cursor) -> Self {
        Cursor { raw, _marker: PhantomData }
    }

    /// Obtains the raw cursor.
    #[inline]
    pub fn raw(self) -> H::Cursor {
        self.raw
    }

    /// Computes the integer index of this cursor, given the haystack origin.
    pub fn to_index<O>(self, origin: O) -> usize
    where
        O: Origin<'h, H>,
    {
        unsafe {
            self.raw.to_index(origin.raw())
        }
    }
}
