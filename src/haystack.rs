//! Haystacks.
//!
//! A *haystack* refers to any linear structure which can be split or sliced
//! into smaller, non-overlapping parts. Examples are strings and vectors.
//!
//! ```rust
//! let haystack: &str = "hello";       // a string slice (`&str`) is a haystack.
//! let (a, b) = haystack.split_at(4);  // it can be split into two strings.
//! let c = &a[1..3];                   // it can be sliced.
//! ```
//!
//! The minimal haystack which cannot be further sliced is called a *codeword*.
//! For instance, the codeword of a string would be a UTF-8 sequence. A haystack
//! can therefore be viewed as a consecutive list of codewords.
//!
//! The boundary between codewords can be addressed using an *index*. The
//! numbers 1, 3 and 4 in the snippet above are sample indices of a string. An
//! index is usually a `usize`.
//!
//! An arbitrary number may point outside of a haystack, or in the interior of a
//! codeword. These indices are invalid. A *valid index* of a certain haystack
//! would only point to the boundaries.

use std::ops::{Deref, Range};
use std::fmt::Debug;
use std::mem;

/// Borrowed [`Haystack`].
///
/// Every `Haystack` type can be borrowed as references to `Hay` types. This
/// allows multiple similar types to share the same implementation (e.g. the
/// haystacks `&[T]`, `&mut [T]` and `Vec<T>` all have the same corresponding
/// hay type `[T]`).
///
/// # Safety
///
/// This trait is unsafe as there are some unchecked requirements which the
/// implementor must uphold. Failing to meet these requirements would lead to
/// out-of-bound access. The safety requirements are written in each member of
/// this trait.
pub unsafe trait Hay {
    /// The index type of the haystack. Typically a `usize`.
    ///
    /// Splitting a hay must be sublinear using this index type. For instance,
    /// if we implement `Hay` for a linked list, the index should not be an
    /// integer offset (`usize`) as this would require O(n) time to chase the
    /// pointer and find the split point. Instead, for a linked list we should
    /// directly use the node pointer as the index.
    ///
    /// # Safety
    ///
    /// Valid indices of a single hay have a total order, even this type does
    /// not require an `Ord` bound — for instance, to order two linked list
    /// cursors, we need to chase the links and see if they meet; this is slow
    /// and not suitable for implementing `Ord`, but conceptually an ordering
    /// can be defined on linked list cursors.
    type Index: Copy + Debug + Eq;

    /// Creates an empty hay.
    ///
    /// # Safety
    ///
    /// An empty hay's start and end indices must be the same, e.g.
    ///
    /// ```rust
    /// extern crate pattern_3;
    /// use pattern_3::Hay;
    ///
    /// let empty = <str>::empty();
    /// assert_eq!(empty.start_index(), empty.end_index());
    /// ```
    ///
    /// This also suggests that there is exactly one valid index for an empty
    /// hay.
    ///
    /// There is no guarantee that two separate calls to `.empty()` will produce
    /// the same hay reference.
    fn empty<'a>() -> &'a Self;

    /// Obtains the index to the start of the hay.
    ///
    /// Usually this method returns `0`.
    ///
    /// # Safety
    ///
    /// Implementation must ensure that the start index of hay is the first
    /// valid index, i.e. for all valid indices `i` of `self`, we have
    /// `self.start_index() <= i`.
    fn start_index(&self) -> Self::Index;

    /// Obtains the index to the end of the hay.
    ///
    /// Usually this method returns the length of the hay.
    ///
    /// # Safety
    ///
    /// Implementation must ensure that the end index of hay is the last valid
    /// index, i.e. for all valid indices `i` of `self`, we have
    /// `i <= self.end_index()`.
    fn end_index(&self) -> Self::Index;

    /// Returns the next immediate index in this haystack.
    ///
    /// # Safety
    ///
    /// The `index` must be a valid index, and also must not equal to
    /// `self.end_index()`.
    ///
    /// Implementation must ensure that if `j = self.next_index(i)`, then `j`
    /// is also a valid index satisfying `j > i`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pattern_3::Hay;
    ///
    /// let sample = "A→😀";
    /// unsafe {
    ///     assert_eq!(sample.next_index(0), 1);
    ///     assert_eq!(sample.next_index(1), 4);
    ///     assert_eq!(sample.next_index(4), 8);
    /// }
    /// ```
    unsafe fn next_index(&self, index: Self::Index) -> Self::Index;

    /// Returns the previous immediate index in this haystack.
    ///
    /// # Safety
    ///
    /// The `index` must be a valid index, and also must not equal to
    /// `self.start_index()`.
    ///
    /// Implementation must ensure that if `j = self.prev_index(i)`, then `j`
    /// is also a valid index satisfying `j < i`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pattern_3::Hay;
    ///
    /// let sample = "A→😀";
    /// unsafe {
    ///     assert_eq!(sample.prev_index(8), 4);
    ///     assert_eq!(sample.prev_index(4), 1);
    ///     assert_eq!(sample.prev_index(1), 0);
    /// }
    /// ```
    unsafe fn prev_index(&self, index: Self::Index) -> Self::Index;

    /// Obtains a child hay by slicing `self`.
    ///
    /// # Safety
    ///
    /// The two ends of the range must be valid indices. The start of the range
    /// must be before the end of the range (`range.start <= range.end`).
    unsafe fn slice_unchecked(&self, range: Range<Self::Index>) -> &Self;
}

/// Linear splittable structure.
///
/// A `Haystack` is implemented for reference and collection types such as
/// `&[T]`, `&mut [T]` and `Vec<T>`. Every haystack can be borrowed as an
/// underlying representation called a [`Hay`]. Multiple haystacks may share the
/// same hay type, and thus share the same implementation of pattern search
/// algorithms.
///
/// # Safety
///
/// This trait is unsafe as there are some unchecked requirements which the
/// implementor must uphold. Failing to meet these requirements would lead to
/// out-of-bound access. The safety requirements are written in each member of
/// this trait.
pub unsafe trait Haystack: Deref + Sized where Self::Target: Hay {
    /// Creates an empty haystack.
    fn empty() -> Self;

    /// Splits the haystack into 3 slices around the given range.
    ///
    /// This method splits `self` into 3 non-overlapping parts:
    ///
    /// 1. Before the range (`self[..range.start]`),
    /// 2. Inside the range (`self[range]`), and
    /// 3. After the range (`self[range.end..]`)
    ///
    /// The returned array contains these 3 parts in order.
    ///
    /// # Safety
    ///
    /// Caller should ensure that the starts and end indices of `range` are
    /// valid indices for the haystack `self` with `range.start <= range.end`.
    ///
    /// If the haystack is a mutable reference (`&mut A`), implementation must
    /// ensure that the 3 returned haystack are truly non-overlapping in memory.
    /// This is required to uphold the "Aliasing XOR Mutability" guarantee. If a
    /// haystack cannot be physically split into non-overlapping parts (e.g. in
    /// `OsStr`), then `&mut A` should not implement `Haystack` either.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pattern_3::Haystack;
    ///
    /// let haystack = &mut [0, 1, 2, 3, 4, 5, 6];
    /// let [left, middle, right] = unsafe { haystack.split_around(2..6) };
    /// assert_eq!(left, &mut [0, 1]);
    /// assert_eq!(middle, &mut [2, 3, 4, 5]);
    /// assert_eq!(right, &mut [6]);
    /// ```
    unsafe fn split_around(self, range: Range<<Self::Target as Hay>::Index>) -> [Self; 3];

    /// Subslices this haystack.
    ///
    /// # Safety
    ///
    /// The starts and end indices of `range` must be valid indices for the
    /// haystack `self` with `range.start <= range.end`.
    unsafe fn slice_unchecked(self, range: Range<<Self::Target as Hay>::Index>) -> Self {
        let [_, middle, _] = self.split_around(range);
        middle
    }

    /// Transforms the range from relative to self's parent to the original
    /// haystack it was sliced from.
    ///
    /// Typically this method can be simply implemented as
    ///
    /// ```text
    /// (original.start + parent.start)..(original.start + parent.end)
    /// ```
    ///
    /// If this haystack is a [`SharedHaystack`], this method would never be
    /// called.
    ///
    /// # Safety
    ///
    /// The `parent` range should be a valid range relative to a hay *a*, which
    /// was used to slice out *self*: `self == &a[parent]`.
    ///
    /// Similarly, the `original` range should be a valid range relative to
    /// another hay *b* used to slice out *a*: `a == &b[original]`.
    ///
    /// The distance of `parent` must be consistent with the length of `self`.
    ///
    /// This method should return a range which satisfies:
    ///
    /// ```text
    /// self == &b[parent][original] == &b[range]
    /// ```
    ///
    /// Slicing can be destructive and *invalidates* some indices, in particular
    /// for owned type with a pointer-like index, e.g. linked list. In this
    /// case, one should derive an entirely new index range from `self`, e.g.
    /// returning `self.start_index()..self.end_index()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pattern_3::Haystack;
    ///
    /// let hay = b"This is a sample haystack";
    /// let this = hay[2..23][3..19].to_vec();
    /// assert_eq!(&*this, &hay[this.restore_range(2..23, 3..19)]);
    /// ```
    fn restore_range(
        &self,
        original: Range<<Self::Target as Hay>::Index>,
        parent: Range<<Self::Target as Hay>::Index>,
    ) -> Range<<Self::Target as Hay>::Index>;
}

/// A haystack which can be shared and cheaply cloned (e.g. `&H`, `Rc<H>`).
///
/// If a haystack implements this marker trait, during internal operations the
/// original haystack will be retained in full and cloned, rather than being
/// sliced and splitted. Being a shared haystack allows searcher to see the
/// entire haystack, including the consumed portion.
pub trait SharedHaystack: Haystack + Clone
where Self::Target: Hay // FIXME: RFC 2089 or 2289
{}

/// The borrowing behavior differs between a (unique) haystack and shared
/// haystack. We use *specialization* to distinguish between these behavior:
///
/// * When using `split_around()` and `slice_unchecked()` with a unique
///     haystack, the original haystack will be splitted or sliced accordingly
///     to maintain unique ownership.
/// * When using these functions with a shared haystack, the original haystack
///     will be cloned in full as that could provide more context into
///     searchers.
///
/// This trait will never be public.
trait SpanBehavior: Haystack
where Self::Target: Hay // FIXME: RFC 2089 or 2289
{
    fn take(&mut self) -> Self;

    fn from_span(span: Span<Self>) -> Self;

    unsafe fn split_around_for_span(self, subrange: Range<<Self::Target as Hay>::Index>) -> [Self; 3];

    unsafe fn slice_unchecked_for_span(self, subrange: Range<<Self::Target as Hay>::Index>) -> Self;

    fn borrow_range(
        &self,
        range: Range<<Self::Target as Hay>::Index>,
    ) -> Range<<Self::Target as Hay>::Index>;

    fn do_restore_range(
        &self,
        range: Range<<Self::Target as Hay>::Index>,
        subrange: Range<<Self::Target as Hay>::Index>,
    ) -> Range<<Self::Target as Hay>::Index>;
}

impl<H: Haystack> SpanBehavior for H
where H::Target: Hay // FIXME: RFC 2089 or 2289
{
    #[inline]
    default fn take(&mut self) -> Self {
        mem::replace(self, Self::empty())
    }

    #[inline]
    default fn from_span(span: Span<Self>) -> Self {
        span.haystack
    }

    #[inline]
    default fn borrow_range(&self, _: Range<<Self::Target as Hay>::Index>) -> Range<<Self::Target as Hay>::Index> {
        self.start_index()..self.end_index()
    }

    #[inline]
    default fn do_restore_range(
        &self,
        range: Range<<Self::Target as Hay>::Index>,
        subrange: Range<<Self::Target as Hay>::Index>,
    ) -> Range<<Self::Target as Hay>::Index> {
        self.restore_range(range, subrange)
    }

    #[inline]
    default unsafe fn split_around_for_span(self, subrange: Range<<Self::Target as Hay>::Index>) -> [Self; 3] {
        self.split_around(subrange)
    }

    #[inline]
    default unsafe fn slice_unchecked_for_span(self, subrange: Range<<Self::Target as Hay>::Index>) -> Self {
        self.slice_unchecked(subrange)
    }
}

impl<H: SharedHaystack> SpanBehavior for H
where H::Target: Hay // FIXME: RFC 2089 or 2289
{
    #[inline]
    fn take(&mut self) -> Self {
        self.clone()
    }

    #[inline]
    fn from_span(span: Span<Self>) -> Self {
        unsafe {
            span.haystack.slice_unchecked(span.range)
        }
    }

    #[inline]
    fn borrow_range(&self, range: Range<<Self::Target as Hay>::Index>) -> Range<<Self::Target as Hay>::Index> {
        range
    }

    #[inline]
    fn do_restore_range(
        &self,
        _: Range<<Self::Target as Hay>::Index>,
        subrange: Range<<Self::Target as Hay>::Index>,
    ) -> Range<<Self::Target as Hay>::Index> {
        subrange
    }

    #[inline]
    unsafe fn split_around_for_span(self, _: Range<<Self::Target as Hay>::Index>) -> [Self; 3] {
        [self.clone(), self.clone(), self]
    }

    #[inline]
    unsafe fn slice_unchecked_for_span(self, _: Range<<Self::Target as Hay>::Index>) -> Self {
        self
    }
}



/// A span is a haystack coupled with the original range where the haystack is found.
#[derive(Debug, Clone)]
pub struct Span<H: Haystack>
where H::Target: Hay // FIXME: RFC 2089 or 2289
{
    haystack: H,
    range: Range<<<H as Deref>::Target as Hay>::Index>,
    //^ The `<H as Trait>` is to trick `#[derive]` not to generate
    //  the where bound for `H::Hay`.
}

/// Creates a span which covers the entire haystack.
impl<H: Haystack> From<H> for Span<H>
where H::Target: Hay // FIXME: RFC 2089 or 2289
{
    #[inline]
    fn from(haystack: H) -> Self {
        let range = haystack.start_index()..haystack.end_index();
        Self { haystack, range }
    }
}

impl<H: SharedHaystack> Span<H>
where H::Target: Hay // FIXME: RFC 2089 or 2289
{
    /// Decomposes this span into the original haystack, and the range it focuses on.
    #[inline]
    pub fn into_parts(self) -> (H, Range<<H::Target as Hay>::Index>) {
        (self.haystack, self.range)
    }

    /// Creates a span from a haystack, and a range it should focus on.
    ///
    /// # Safety
    ///
    /// The `range` must be a valid range relative to `haystack`.
    #[inline]
    pub unsafe fn from_parts(haystack: H, range: Range<<H::Target as Hay>::Index>) -> Self {
        Self { haystack, range }
    }
}

impl<'h> Span<&'h str> {
    /// Reinterprets the string span as a byte-array span.
    #[inline]
    pub fn as_bytes(self) -> Span<&'h [u8]> {
        Span {
            haystack: self.haystack.as_bytes(),
            range: self.range,
        }
    }
}

impl<H: Haystack> Span<H>
where H::Target: Hay // FIXME: RFC 2089 or 2289
{
    /// The range of the span, relative to the ultimate original haystack it was sliced from.
    #[inline]
    pub fn original_range(&self) -> Range<<H::Target as Hay>::Index> {
        self.range.clone()
    }

    /// Borrows a shared span.
    #[inline]
    pub fn borrow(&self) -> Span<&H::Target> {
        Span {
            haystack: &*self.haystack,
            range: self.haystack.borrow_range(self.range.clone()),
        }
    }

    /// Checks whether this span is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.range.start == self.range.end
    }

    /// Returns this span by value, and replaces the original span by an empty
    /// span.
    #[inline]
    pub fn take(&mut self) -> Self {
        let haystack = self.haystack.take();
        let range = self.range.clone();
        self.range.end = self.range.start;
        Span { haystack, range }
    }

    // FIXME: This should be changed to an `impl From<Span<H>> for H`.
    /// Slices the original haystack to the focused range.
    #[inline]
    pub fn into(self) -> H {
        H::from_span(self)
    }

    /// Splits this span into 3 spans around the given range.
    ///
    /// # Safety
    ///
    /// `subrange` must be a valid range relative to `self.borrow()`. A safe
    /// usage is like:
    ///
    /// ```rust
    /// # use pattern_3::{Span, Pattern, Searcher};
    /// # let span = Span::from("foo");
    /// # let mut searcher = <&str as Pattern<&str>>::into_searcher("o");
    /// # (|| -> Option<()> {
    /// let range = searcher.search(span.borrow())?;
    /// let [left, middle, right] = unsafe { span.split_around(range) };
    /// # Some(()) })();
    /// ```
    #[inline]
    pub unsafe fn split_around(self, subrange: Range<<H::Target as Hay>::Index>) -> [Self; 3] {
        let self_range = self.haystack.borrow_range(self.range.clone());
        let [left, middle, right] = self.haystack.split_around_for_span(subrange.clone());

        let left_range = left.do_restore_range(self.range.clone(), self_range.start..subrange.start);
        let right_range = right.do_restore_range(self.range.clone(), subrange.end..self_range.end);
        let middle_range = middle.do_restore_range(self.range, subrange);

        [
            Self { haystack: left, range: left_range },
            Self { haystack: middle, range: middle_range },
            Self { haystack: right, range: right_range },
        ]
    }

    /// Slices this span to the given range.
    ///
    /// # Safety
    ///
    /// `subrange` must be a valid range relative to `self.borrow()`.
    #[inline]
    pub unsafe fn slice_unchecked(self, subrange: Range<<H::Target as Hay>::Index>) -> Self {
        let haystack = self.haystack.slice_unchecked_for_span(subrange.clone());
        let range = haystack.do_restore_range(self.range, subrange);
        Self { haystack, range }
    }
}

unsafe impl<'a, A: Hay + ?Sized + 'a> Haystack for &'a A {
    #[inline]
    fn empty() -> Self {
        A::empty()
    }

    #[inline]
    unsafe fn split_around(self, range: Range<A::Index>) -> [Self; 3] {
        [
            self.slice_unchecked(self.start_index()..range.start),
            self.slice_unchecked(range.clone()),
            self.slice_unchecked(range.end..self.end_index()),
        ]
    }

    #[inline]
    unsafe fn slice_unchecked(self, range: Range<A::Index>) -> Self {
        A::slice_unchecked(self, range)
    }

    #[inline]
    fn restore_range(&self, _: Range<A::Index>, _: Range<A::Index>) -> Range<A::Index> {
        unreachable!()
    }
}

impl<'a, A: Hay + ?Sized + 'a> SharedHaystack for &'a A {}
