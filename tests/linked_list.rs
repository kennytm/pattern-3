#![feature(box_into_raw_non_null)]
#![feature(specialization)]

extern crate pattern_3;

use pattern_3::*;

use std::fmt;
use std::mem::{forget, replace, transmute};
use std::ops::{Deref, Range};
use std::ptr::NonNull;
use std::marker::PhantomData;

//------------------------------------------------------------------------------
// Define a simple (unsafe) doubly-linked list.

struct Node<T> {
    prev: Option<NonNull<Node<T>>>,
    next: Option<NonNull<Node<T>>>,
    content_after: Option<T>,
}
struct DList<T> {
    head: NonNull<Node<T>>,
    tail: NonNull<Node<T>>,
}

impl<T> Drop for DList<T> {
    fn drop(&mut self) {
        let mut next = Some(self.head);
        while let Some(cur) = next {
            unsafe {
                next = cur.as_ref().next;
                drop(Box::from_raw(cur.as_ptr()));
            }
        }
    }
}

impl<T> DList<T> {
    fn new() -> Self {
        let node = Box::new(Node {
            prev: None,
            next: None,
            content_after: None,
        });
        let node_raw = Box::into_raw_non_null(node);
        DList {
            head: node_raw,
            tail: node_raw,
        }
    }

    fn push_front(&mut self, value: T) {
        let new_head = Box::new(Node {
            prev: None,
            next: Some(self.head),
            content_after: Some(value),
        });
        let mut old_head = replace(&mut self.head, Box::into_raw_non_null(new_head));
        unsafe {
            old_head.as_mut().prev = Some(self.head);
        }
    }

    fn pop_front(&mut self) -> Option<T> {
        unsafe {
            let new_head = self.head.as_ref().next?;
            let old_head = replace(&mut self.head, new_head);
            self.head.as_mut().prev = None;
            Box::from_raw(old_head.as_ptr()).content_after
        }
    }

    fn push_back(&mut self, value: T) {
        let new_tail = Box::new(Node {
            prev: Some(self.tail),
            next: None,
            content_after: None,
        });
        let mut old_tail = replace(&mut self.tail, Box::into_raw_non_null(new_tail));
        unsafe {
            let old_tail_mut = old_tail.as_mut();
            old_tail_mut.next = Some(self.tail);
            old_tail_mut.content_after = Some(value);
        }
    }

    fn pop_back(&mut self) -> Option<T> {
        unsafe {
            let mut new_tail = self.tail.as_ref().prev?;
            let res = {
                let new_tail_mut = new_tail.as_mut();
                new_tail_mut.next = None;
                new_tail_mut.content_after.take()
            };
            drop(Box::from_raw(replace(&mut self.tail, new_tail).as_ptr()));
            res
        }
    }

    unsafe fn split_at_unchecked(self, cursor: Cursor<T>) -> (Self, Self) {
        let mut new_head = cursor.ptr;
        if new_head == self.head {
            (DList::new(), self)
        } else if new_head == self.tail {
            (self, DList::new())
        } else {
            let new_tail = Box::new(Node {
                prev: new_head.as_mut().prev.take(),
                next: None,
                content_after: None,
            });
            let new_tail = Box::into_raw_non_null(new_tail);
            if let Some(mut next) = new_head.as_ref().next {
                next.as_mut().prev = Some(new_head);
            }
            if let Some(mut prev) = new_tail.as_ref().prev {
                prev.as_mut().next = Some(new_tail);
            }
            let left = DList { head: self.head, tail: new_tail };
            let right = DList { head: new_head, tail: self.tail };
            forget(self);
            (left, right)
        }
    }
}

#[test]
fn test_linked_list() {
    let mut list = DList::new();
    list.push_front(1);
    list.push_back(2);
    list.push_front(3);
    list.push_front(4);
    list.push_back(5);
    list.push_back(6);
    assert_eq!(list.pop_front(), Some(4));
    assert_eq!(list.pop_back(), Some(6));
    assert_eq!(list.pop_front(), Some(3));
    assert_eq!(list.pop_front(), Some(1));
    assert_eq!(list.pop_back(), Some(5));
    assert_eq!(list.pop_back(), Some(2));
    list.push_back(7);
    assert_eq!(list.pop_front(), Some(7));
    assert_eq!(list.pop_front(), None);
    assert_eq!(list.pop_back(), None);
}

#[test]
fn test_linked_list_split() {
    let mut list = DList::new();
    list.push_back(1);
    list.push_back(2);
    list.push_back(3);
    let cursor = list.end_cursor();
    list.push_back(4);
    list.push_back(5);
    list.push_back(6);

    let (mut left, mut right) = unsafe { list.split_at_unchecked(cursor) };

    assert_eq!(left.pop_front(), Some(1));
    assert_eq!(left.pop_front(), Some(2));
    assert_eq!(left.pop_front(), Some(3));
    assert_eq!(left.pop_front(), None);

    assert_eq!(right.pop_front(), Some(4));
    assert_eq!(right.pop_front(), Some(5));
    assert_eq!(right.pop_front(), Some(6));
    assert_eq!(right.pop_front(), None);
}


#[test]
fn test_linked_list_drop() {
    use std::cell::RefCell;

    struct LeakCounter<'a> {
        counter: &'a RefCell<u32>,
        result: u32,
    }
    impl<'a> Drop for LeakCounter<'a> {
        fn drop(&mut self) {
            *self.counter.borrow_mut() += self.result;
        }
    }

    let counter = RefCell::new(0);
    {
        let mut list = DList::new();
        list.push_front(LeakCounter { counter: &counter, result: 1 });
        list.push_back(LeakCounter { counter: &counter, result: 2 });
        list.push_back(LeakCounter { counter: &counter, result: 4 });
        list.push_front(LeakCounter { counter: &counter, result: 8 });

        let lc = list.pop_back().unwrap();
        assert_eq!(lc.result, 4);
        forget(lc);

        let lc = list.pop_front().unwrap();
        assert_eq!(lc.result, 8);
        forget(lc);
    }

    assert_eq!(*counter.borrow(), 3);
}


//------------------------------------------------------------------------------
// Define a cursor to the linked list.

struct Cursor<T> {
    ptr: NonNull<Node<T>>,
}

impl<T> Clone for Cursor<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> fmt::Debug for Cursor<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Cursor")
            .field("ptr", &self.ptr)
            .finish()
    }
}
impl<T> PartialEq for Cursor<T> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}
impl<T> Copy for Cursor<T> {}
impl<T> Eq for Cursor<T> {}

impl<T> Cursor<T> {
    fn empty() -> Self {
        Cursor {
            ptr: NonNull::from(&Node {
                prev: None,
                next: None,
                content_after: None,
            }),
        }
    }

    fn next(self) -> Option<Self> {
        unsafe {
            let next = self.ptr.as_ref().next?;
            Some(Cursor { ptr: next })
        }
    }
    fn prev(self) -> Option<Self> {
        unsafe {
            let prev = self.ptr.as_ref().prev?;
            Some(Cursor { ptr: prev })
        }
    }
    unsafe fn content_after<'d>(self) -> Option<&'d T> {
        (*self.ptr.as_ptr()).content_after.as_ref()
    }
}

#[test]
fn test_cursor() {
    let mut list = DList::new();
    list.push_back(2);

    let front = list.start_cursor();
    let back = list.end_cursor();
    assert_eq!(front.next(), Some(back));
    assert_eq!(front.prev(), None);
    assert_eq!(back.prev(), Some(front));
    assert_eq!(back.next(), None);

    unsafe {
        assert_eq!(front.content_after(), Some(&2));
        assert_eq!(back.content_after(), None);
    }

    assert_eq!(Cursor::<i32>::empty(), Cursor::<i32>::empty());
}

//------------------------------------------------------------------------------
// Define a slice of the linked list (simulating custom DST).

struct Slice<T>([Node<T>]);

impl<T> Deref for DList<T> {
    type Target = Slice<T>;

    fn deref(&self) -> &Slice<T> {
        let start = Cursor { ptr: self.head };
        let end = Cursor { ptr: self.tail };
        unsafe { Slice::from_cursors(start, end) }
    }
}

impl<T> Slice<T> {
    unsafe fn from_cursors<'a>(start: Cursor<T>, end: Cursor<T>) -> &'a Self {
        transmute((start, end))
    }

    fn as_cursors(&self) -> (Cursor<T>, Cursor<T>) {
        unsafe { transmute(self) }
    }

    fn front(&self) -> Option<&T> {
        unsafe {
            self.as_cursors().0.content_after()
        }
    }

    fn back(&self) -> Option<&T> {
        unsafe {
            self.as_cursors().1.prev()?.content_after()
        }
    }

    fn start_cursor(&self) -> Cursor<T> {
        self.as_cursors().0
    }

    fn end_cursor(&self) -> Cursor<T> {
        self.as_cursors().1
    }

    fn iter(&self) -> Iter<'_, T> {
        let (start, end) = self.as_cursors();
        Iter { start, end, _marker: PhantomData }
    }
}

struct Iter<'a, T: 'a> {
    start: Cursor<T>,
    end: Cursor<T>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: 'a> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.start == self.end {
                None
            } else {
                let next = self.start.next()?;
                replace(&mut self.start, next).content_after()
            }
        }
    }
}

impl<'a, T: 'a> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.start == self.end {
                None
            } else {
                self.end = self.end.prev()?;
                self.end.content_after()
            }
        }
    }
}

#[test]
fn test_slice() {
    let mut list = DList::new();
    list.push_back(2);
    list.push_back(4);
    list.push_back(6);

    assert_eq!(list.front(), Some(&2));
    assert_eq!(list.back(), Some(&6));

    list.pop_front();
    list.pop_back();

    assert_eq!(list.front(), Some(&4));
    assert_eq!(list.back(), Some(&4));

    list.pop_back();

    assert_eq!(list.front(), None);
    assert_eq!(list.back(), None);
}

#[test]
fn test_iter() {
    let mut list = DList::new();
    list.push_back(3);
    list.push_back(6);
    list.push_back(9);

    assert_eq!(list.iter().collect::<Vec<_>>(), vec![&3, &6, &9]);
    assert_eq!(list.iter().rev().collect::<Vec<_>>(), vec![&9, &6, &3]);
}

//------------------------------------------------------------------------------
// Now! We implement the haystack API on it.

impl<T> Hay for Slice<T> {
    type Index = Cursor<T>;

    fn empty<'a>() -> &'a Self {
        unsafe { Self::from_cursors(Cursor::empty(), Cursor::empty()) }
    }

    fn start_index(&self) -> Cursor<T> {
        self.as_cursors().0
    }

    fn end_index(&self) -> Cursor<T> {
        self.as_cursors().1
    }

    unsafe fn next_index(&self, index: Cursor<T>) -> Cursor<T> {
        index.next().unwrap()
    }

    unsafe fn prev_index(&self, index: Cursor<T>) -> Cursor<T> {
        index.prev().unwrap()
    }

    unsafe fn slice_unchecked(&self, range: Range<Cursor<T>>) -> &Self {
        Self::from_cursors(range.start, range.end)
    }
}

impl<T> Haystack for DList<T> {
    fn empty() -> Self {
        DList::new()
    }

    unsafe fn split_around(self, range: Range<Cursor<T>>) -> [Self; 3] {
        if range.start == range.end {
            let (left, right) = self.split_at_unchecked(range.start);
            [left, DList::new(), right]
        } else {
            let (left, haystack) = self.split_at_unchecked(range.start);
            let (middle, right) = haystack.split_at_unchecked(range.end);
            [left, middle, right]
        }
    }

    unsafe fn slice_unchecked(self, range: Range<Cursor<T>>) -> Self {
        let [_, middle, _] = self.split_around(range);
        middle
    }

    fn restore_range(&self, _: Range<Cursor<T>>, _: Range<Cursor<T>>) -> Range<Cursor<T>> {
        self.start_cursor()..self.end_cursor()
    }
}

//------------------------------------------------------------------------------
// Implement the searcher by equality

struct ElemSearcher<'p, T: 'p>(&'p T);

unsafe impl<'p, T> Searcher<Slice<T>> for ElemSearcher<'p, T>
where
    T: PartialEq + 'p,
{
    fn search(&mut self, span: Span<&Slice<T>>) -> Option<Range<Cursor<T>>> {
        let mut range = span.into_parts().1;
        while range.start != range.end {
            let next = range.start.next().unwrap();
            if unsafe { range.start.content_after() }.unwrap() == self.0 {
                return Some(range.start..next);
            }
            range.start = next;
        }
        None
    }
}

unsafe impl<'p, T> ReverseSearcher<Slice<T>> for ElemSearcher<'p, T>
where
    T: PartialEq + 'p,
{
    fn rsearch(&mut self, span: Span<&Slice<T>>) -> Option<Range<Cursor<T>>> {
        let mut range = span.into_parts().1;
        while range.start != range.end {
            let prev = range.end.prev().unwrap();
            if unsafe { prev.content_after() }.unwrap() == self.0 {
                return Some(prev..range.end);
            }
            range.end = prev;
        }
        None
    }
}

unsafe impl<'p, T> DoubleEndedSearcher<Slice<T>> for ElemSearcher<'p, T>
where
    T: PartialEq + 'p,
{}

unsafe impl<'p, T> Checker<Slice<T>> for ElemSearcher<'p, T>
where
    T: PartialEq + 'p,
{
    fn check(&mut self, span: Span<&Slice<T>>) -> Option<Cursor<T>> {
        let range = span.into_parts().1;
        if range.start == range.end {
            return None;
        }
        if unsafe { range.start.content_after() }.unwrap() == self.0 {
            Some(range.start.next().unwrap())
        } else {
            None
        }
    }
}

unsafe impl<'p, T> ReverseChecker<Slice<T>> for ElemSearcher<'p, T>
where
    T: PartialEq + 'p,
{
    fn rcheck(&mut self, span: Span<&Slice<T>>) -> Option<Cursor<T>> {
        let range = span.into_parts().1;
        if range.start == range.end {
            return None;
        }
        let prev = range.end.prev().unwrap();
        if unsafe { prev.content_after() }.unwrap() == self.0 {
            Some(prev)
        } else {
            None
        }
    }
}

unsafe impl<'p, T> DoubleEndedChecker<Slice<T>> for ElemSearcher<'p, T>
where
    T: PartialEq + 'p,
{}

impl<'h, 'p, T> Pattern<&'h Slice<T>> for ElemSearcher<'p, T>
where
    T: PartialEq + 'p
{
    type Searcher = ElemSearcher<'p, T>;
    type Checker = ElemSearcher<'p, T>;

    fn into_searcher(self) -> Self::Searcher { self }
    fn into_checker(self) -> Self::Checker { self }
}

impl<'h, 'p, T> Pattern<DList<T>> for ElemSearcher<'p, T>
where
    T: PartialEq + 'p
{
    type Searcher = ElemSearcher<'p, T>;
    type Checker = ElemSearcher<'p, T>;

    fn into_searcher(self) -> Self::Searcher { self }
    fn into_checker(self) -> Self::Checker { self }
}

//------------------------------------------------------------------------------
// Implement the searcher for empty pattern

struct EmptyPattern;

impl<'h, T> Pattern<DList<T>> for EmptyPattern {
    type Searcher = pattern::EmptySearcher;
    type Checker = pattern::EmptySearcher;

    fn into_searcher(self) -> Self::Searcher { Self::Searcher::default() }
    fn into_checker(self) -> Self::Checker { Self::Checker::default() }
}

//------------------------------------------------------------------------------
// Test cases

#[test]
fn test_pattern_api() {
    let mut list = DList::new();
    list.push_back(1);
    list.push_back(2);
    list.push_back(3);
    list.push_back(1);
    list.push_back(4);
    list.push_back(5);
    list.push_back(6);
    list.push_back(1);
    list.push_back(7);
    list.push_back(8);

    assert_eq!(
        ext::split(&*list, ElemSearcher(&1))
            .map(|s| s.iter().collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        vec![
            vec![],
            vec![&2, &3],
            vec![&4, &5, &6],
            vec![&7, &8],
        ]
    );

    assert_eq!(
        ext::rsplit(list, ElemSearcher(&1))
            .map(|s| s.iter().cloned().collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        vec![
            vec![7, 8],
            vec![4, 5, 6],
            vec![2, 3],
            vec![],
        ]
    );
}

#[test]
fn test_empty_pattern_api() {
    let mut list = DList::new();
    list.push_back(1);
    list.push_back(2);

    assert_eq!(
        ext::split(list, EmptyPattern)
            .map(|s| { eprintln!("{:?}..{:?}", s.start_cursor(), s.end_cursor()); s.iter().cloned().collect::<Vec<_>>() })
            .collect::<Vec<_>>(),
        vec![
            vec![],
            vec![1],
            vec![2],
            vec![],
        ]
    );
}

#[test]
fn test_pattern_range() {
    let mut list = DList::new();
    list.push_back(1);
    list.push_back(2);
    list.push_back(1);
    list.push_back(1);
    list.push_back(3);

    for (range, slice) in ext::match_ranges(&*list, ElemSearcher(&1)) {
        assert_eq!(slice.start_cursor(), range.start);
        assert_eq!(slice.end_cursor(), range.end);
    }

    for (range, sublist) in ext::match_ranges(list, ElemSearcher(&1)) {
        assert_eq!(sublist.start_cursor(), range.start);
        assert_eq!(sublist.end_cursor(), range.end);
    }
}
