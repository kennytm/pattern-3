use haystack::{Haystack, HaystackMut};
use cursor::{Cursor, Origin, SharedOrigin, MutOrigin};
use std::{fmt, mem};
use std::ops::Range;

/// A span covers a continuous subsequence of a haystack, bounded by a "start"
/// and "end" cursor.
pub struct Span<'h, H>
where
    H: Haystack + ?Sized,
{
    start: Cursor<'h, H>,
    end: Cursor<'h, H>,
}

impl<'h, H> fmt::Debug for Span<'h, H>
where
    H: Haystack + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Span")
            .field(&(self.start..self.end))
            .finish()
    }
}

impl<'h, H> Span<'h, H>
where
    H: Haystack + ?Sized,
{
    /// Creates a new span from the haystack.
    #[inline]
    pub fn new(haystack: &'h H) -> Self {
        let (start, end) = haystack.as_span_raw();
        unsafe {
            Span {
                start: Cursor::new_unchecked(start),
                end: Cursor::new_unchecked(end),
            }
        }
    }

    /// Creates a new span and obtains the origin of a haystack.
    #[inline]
    pub fn and_origin(haystack: &'h H) -> (Self, SharedOrigin<'h, H>) {
        (Self::new(haystack), SharedOrigin::new(haystack))
    }

    /// Returns the index of the start cursor.
    #[inline]
    pub fn to_index<O>(&self, origin: O) -> usize
    where
        O: Origin<'h, H>,
    {
        self.start.to_index(origin)
    }

    /// Returns the index range.
    #[inline]
    pub fn to_range<O>(&self, origin: O) -> Range<usize>
    where
        O: Origin<'h, H>,
    {
        let start = self.start.to_index(origin);
        let end = self.end.to_index(origin);
        start..end
    }

    /// Returns the start cursor.
    #[inline]
    pub fn start(&self) -> Cursor<'h, H> {
        self.start
    }

    /// Returns the end cursor.
    #[inline]
    pub fn end(&self) -> Cursor<'h, H> {
        self.end
    }

    #[inline]
    fn validate_cursor(&self, cursor: Cursor<'h, H>) {
        debug_assert!(self.start <= cursor, "Cursor before start of span");
        debug_assert!(cursor <= self.end, "Cursor after end of span");
    }

    /// Splits this span into two parts, returns the starting part, and update
    /// `self` to become the ending part.
    ///
    /// If this span was `start..end`, this method will set the span to
    /// `cursor..end` and return `start..cursor`.
    ///
    /// # Safety
    ///
    /// The cursor should be valid (points to sequence boundaries) and lies
    /// between `start..end` of the current span.
    #[inline]
    pub fn remove_start(&mut self, cursor: Cursor<'h, H>) -> Self {
        self.validate_cursor(cursor);
        Span {
            start: mem::replace(&mut self.start, cursor),
            end: cursor,
        }
    }

    /// Splits this span into two parts, returns the ending part, and update
    /// `self` to become the starting part.
    ///
    /// If this span was `start..end`, this method will set the span to
    /// `start..cursor` and return `cursor..end`.
    ///
    /// # Safety
    ///
    /// The cursor should be valid (points to sequence boundaries) and lies
    /// between `start..end` of the current span.
    #[inline]
    pub fn remove_end(&mut self, cursor: Cursor<'h, H>) -> Self {
        self.validate_cursor(cursor);
        Span {
            start: cursor,
            end: mem::replace(&mut self.end, cursor),
        }
    }

    #[inline]
    fn validate_span(&self, c1: Cursor<'h, H>, c2: Cursor<'h, H>) {
        debug_assert!(self.start <= c1, "First cursor before start of span");
        debug_assert!(c1 <= c2, "Cursors not in correct order");
        debug_assert!(c2 <= self.end, "Second cursor after end of span");
    }

    /// Takes a subspan between two cursors from this span, and update `self` to
    /// the end of the subspan.
    ///
    /// If this span was `start..end`, this method will set the span to
    /// `c2..end` and return `c1..c2`.
    ///
    /// # Safety
    ///
    /// The cursors should be valid (points to sequence boundaries), lie between
    /// `start..end` of the current span, and `c1` should be positioned before
    /// `c2`.
    #[inline]
    pub fn take_from_start(&mut self, c1: Cursor<'h, H>, c2: Cursor<'h, H>) -> Self {
        self.validate_span(c1, c2);
        self.start = c2;
        Span {
            start: c1,
            end: c2,
        }
    }

    #[inline]
    pub unsafe fn take_from_start_unchecked(&mut self, c1: H::Cursor, c2: H::Cursor) -> Self {
        let c1 = Cursor::new_unchecked(c1);
        let c2 = Cursor::new_unchecked(c2);
        self.start = c2;
        Span {
            start: c1,
            end: c2,
        }
    }

    /// Takes a subspan between two cursors from this span, and update `self` to
    /// the start of the subspan.
    ///
    /// If this span was `start..end`, this method will set the span to
    /// `start..c1` and return `c1..c2`.
    ///
    /// # Safety
    ///
    /// The cursors should be valid (points to sequence boundaries), lie between
    /// `start..end` of the current span, and `c1` should be positioned before
    /// `c2`.
    #[inline]
    pub fn take_from_end(&mut self, c1: Cursor<'h, H>, c2: Cursor<'h, H>) -> Self {
        self.validate_span(c1, c2);
        self.end = c1;
        Span {
            start: c1,
            end: c2,
        }
    }

    #[inline]
    pub unsafe fn take_from_end_unchecked(&mut self, c1: H::Cursor, c2: H::Cursor) -> Self {
        let c1 = Cursor::new_unchecked(c1);
        let c2 = Cursor::new_unchecked(c2);
        self.end = c1;
        Span {
            start: c1,
            end: c2,
        }
    }

    /// Takes a subspan to the first cursor, and update `self` to start from the
    /// second cursor.
    ///
    /// If this span was `start..end`, this method will set the span to
    /// `c2..end` and return `start..c1`.
    ///
    ///  # Safety
    ///
    /// The cursors should be valid (points to sequence boundaries), lie between
    /// `start..end` of the current span, and `c1` should be positioned before
    /// `c2`.
    pub fn cut_from_start(&mut self, c1: Cursor<'h, H>, c2: Cursor<'h, H>) -> Self {
        self.validate_span(c1, c2);
        Span {
            start: mem::replace(&mut self.start, c2),
            end: c1,
        }
    }

    /// Takes a subspan from the second cursor, and update `self` to end until
    /// first cursor.
    ///
    /// If this span was `start..end`, this method will set the span to
    /// `start..c1` and return `c2..end`.
    ///
    ///  # Safety
    ///
    /// The cursors should be valid (points to sequence boundaries), lie between
    /// `start..end` of the current span, and `c1` should be positioned before
    /// `c2`.
    pub fn cut_from_end(&mut self, c1: Cursor<'h, H>, c2: Cursor<'h, H>) -> Self {
        self.validate_span(c1, c2);
        Span {
            start: c1,
            end: mem::replace(&mut self.end, c2),
        }
    }

    /// Collapses this span to an empty span pointing to the original end.
    pub fn collapse_to_end(&mut self) {
        self.start = self.end;
    }

    /// Collapses this span to an empty span pointing to the original start.
    pub fn collapse_to_start(&mut self) {
        self.end = self.start;
    }

    /// Checks whether this span is empty.
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Clones this span.
    ///
    /// # Safety
    ///
    /// This method is unsafe because a span can be used to represent a mutable
    /// haystack e.g. `&mut H`. In this case, cloning the span will allow
    /// external code to create mutable aliases via [`to_haystack_mut`].
    ///
    /// This method requires the caller to adhere one the following conditions:
    ///
    /// * The source haystack is immutable e.g. `&H`, or
    /// * The `to_haystack_mut` method is only called on `self`, and none of its
    ///     clones, or
    /// * The `to_haystack_mut` method is only called on the clone, and not
    ///     `self`.
    pub unsafe fn clone(&self) -> Self {
        Span {
            start: self.start,
            end: self.end,
        }
    }

    /// Retrieves the original haystack.
    pub fn to_haystack(self, origin: SharedOrigin<'h, H>) -> &'h H {
        unsafe {
            H::from_span_raw(origin.raw(), self.start.raw(), self.end.raw())
        }
    }
}

impl<'h, H> Span<'h, H>
where
    H: HaystackMut + ?Sized,
{
    /// Creates a new span and obtains the origin of a mutable haystack.
    #[inline]
    pub fn and_origin_mut(haystack: &'h mut H) -> (Self, MutOrigin<'h, H>) {
        let (start, end) = haystack.as_span_raw();
        unsafe {
            (
                Span {
                    start: Cursor::new_unchecked(start),
                    end: Cursor::new_unchecked(end),
                },
                MutOrigin::new(haystack),
            )
        }
    }

    /// Retrieves the original mutable haystack.
    #[inline]
    pub fn to_haystack_mut(self, origin: MutOrigin<'h, H>) -> &'h mut H {
        unsafe {
            H::from_span_raw_mut(origin.raw(), self.start.raw(), self.end.raw())
        }
    }
}
