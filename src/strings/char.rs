use pattern::{Pattern, Searcher, ReverseSearcher, DoubleEndedSearcher};
use haystack::Haystack;
use super::StrLike;
use memchr::{memchr, memrchr};

#[derive(Debug, Clone)]
pub struct CharSearcher {
    // safety invariant: `utf8_size` must be less than 5
    utf8_size: usize,

    /// A utf8 encoded copy of the `needle`
    utf8_encoded: [u8; 4],
}

impl CharSearcher {
    #[inline]
    fn as_bytes(&self) -> &[u8] {
        unsafe {
            self.utf8_encoded.get_unchecked(..self.utf8_size)
        }
    }

    #[inline]
    fn last_byte(&self) -> u8 {
        unsafe {
            *self.utf8_encoded.get_unchecked(self.utf8_size - 1)
        }
    }

    fn next<H>(&self, rest: &mut H) -> (H, Option<H>)
    where
        H: StrLike + Haystack,
    {
        let h = rest.collapse_to_end();
        unsafe {
            let index = {
                let mut finger = 0;
                let mut bytes = h.as_bytes();
                loop {
                    if let Some(index) = memchr(self.last_byte(), bytes.get_unchecked(finger..)) {
                        finger += index + 1;
                        if finger >= self.utf8_size {
                            let found = bytes.get_unchecked((finger - self.utf8_size)..finger);
                            if found == self.as_bytes() {
                                break Some(finger);
                            }
                        }
                    } else {
                        break None;
                    }
                }
            };
            if let Some(index) = index {
                let (before, found, after) = h.split_around_unchecked((index - self.utf8_size)..index);
                *rest = after;
                (before, Some(found))
            } else {
                (h, None)
            }
        }
    }

    fn next_back<H>(&self, rest: &mut H) -> (Option<H>, H)
    where
        H: StrLike + Haystack,
    {
        let h = rest.collapse_to_start();
        unsafe {
            let index = {
                let mut bytes = h.as_bytes();
                loop {
                    if let Some(index) = memrchr(self.last_byte(), bytes) {
                        let index = index + 1;
                        if index >= self.utf8_size {
                            let found = bytes.get_unchecked((index - self.utf8_size)..index);
                            if found == self.as_bytes() {
                                break Some(index);
                            }
                        }
                        bytes = bytes.get_unchecked(..(index - 1));
                    } else {
                        break None;
                    }
                }
            };
            if let Some(index) = index {
                let (before, found, after) = h.split_around_unchecked((index - self.utf8_size)..index);
                *rest = before;
                (Some(found), after)
            } else {
                (None, h)
            }
        }
    }

    #[inline]
    fn is_suffix_of(self, haystack: &str) -> bool {
        let bytes = haystack.as_bytes();
        if self.utf8_size <= bytes.len() {
            let suffix = unsafe { bytes.get_unchecked((bytes.len() - self.utf8_size)..) };
            suffix == self.as_bytes()
        } else {
            false
        }
    }
}

#[inline]
fn is_prefix_of(c: char, haystack: &str) -> bool {
    let mut utf8_encoded = [0u8; 4];
    let encoded = c.encode_utf8(&mut utf8_encoded);
    haystack.as_bytes().get(..encoded.len()) == Some(encoded.as_bytes())
}

#[inline]
fn is_suffix_of(c: char, haystack: &str) -> bool {
    let mut utf8_encoded = [0u8; 4];
    let encoded = c.encode_utf8(&mut utf8_encoded);
    if encoded.len() > haystack.len() {
        return false;
    }
    let suffix = unsafe { haystack.get_unchecked((haystack.len() - encoded.len())..) };
    suffix == encoded
}


macro_rules! impl_pattern_for_str_like {
    ([$($gen:tt)*] $ty:ty) => {
        impl $($gen)* Pattern<$ty> for char {
            type Searcher = CharSearcher;

            #[inline]
            fn into_searcher(self) -> Self::Searcher {
                let mut utf8_encoded = [0u8; 4];
                let utf8_size = self.encode_utf8(&mut utf8_encoded).len();
                CharSearcher {
                    utf8_size,
                    utf8_encoded,
                }
            }

            #[inline]
            fn is_prefix_of(self, haystack: $ty) -> bool {
                is_prefix_of(self, &*haystack)
            }

            #[inline]
            fn trim_start(&mut self, haystack: &mut $ty) {
                (|c: char| c == *self).trim_start(haystack)
            }

            #[inline]
            fn is_suffix_of(self, haystack: $ty) -> bool {
                is_suffix_of(self, &*haystack)
            }

            #[inline]
            fn trim_end(&mut self, haystack: &mut $ty) {
                (|c: char| c == *self).trim_end(haystack)
            }
        }

        unsafe impl $($gen)* Searcher<$ty> for CharSearcher {
            #[inline]
            fn search(&mut self, haystack: &mut $ty) -> ($ty, Option<$ty>) {
                self.next(haystack)
            }
        }

        unsafe impl $($gen)* ReverseSearcher<$ty> for CharSearcher {
            #[inline]
            fn rsearch(&mut self, haystack: &mut $ty) -> (Option<$ty>, $ty) {
                self.next_back(haystack)
            }
        }

        unsafe impl $($gen)* DoubleEndedSearcher<$ty> for CharSearcher {}
    }
}

impl_pattern_for_str_like!([<'h>] &'h str);
impl_pattern_for_str_like!([<'h>] &'h mut str);
