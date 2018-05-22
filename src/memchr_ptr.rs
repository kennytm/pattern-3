// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// Original implementation taken from rust-memchr
// Copyright 2015 Andrew Gallant, bluss and Nicolas Koch

use std::cmp;
use std::mem;
use std::ptr;

const LO_U64: u64 = 0x0101010101010101;
const HI_U64: u64 = 0x8080808080808080;

// use truncation
const LO_USIZE: usize = LO_U64 as usize;
const HI_USIZE: usize = HI_U64 as usize;

/// Return `true` if `x` contains any zero byte.
///
/// From *Matters Computational*, J. Arndt
///
/// "The idea is to subtract one from each of the bytes and then look for
/// bytes where the borrow propagated all the way to the most significant
/// bit."
#[inline]
fn contains_zero_byte(x: usize) -> bool {
    x.wrapping_sub(LO_USIZE) & !x & HI_USIZE != 0
}

#[cfg(target_pointer_width = "16")]
#[inline]
fn repeat_byte(b: u8) -> usize {
    (b as usize) << 8 | b as usize
}

#[cfg(not(target_pointer_width = "16"))]
#[inline]
fn repeat_byte(b: u8) -> usize {
    use std::usize::MAX;
    (b as usize) * (MAX / 255)
}

unsafe fn memchr_ptr_slow(x: u8, mut start: *const u8, end: *const u8) -> *const u8 {
    while start != end {
        if *start == x {
            return start;
        }
        start = start.add(1);
    }
    ptr::null()
}

/// Return the first index matching the byte `x` in `text`.
pub fn memchr_ptr(x: u8, text: &[u8]) -> *const u8 {
    // Scan for a single byte value by reading two `usize` words at a time.
    //
    // Split `text` in three parts
    // - unaligned initial part, before the first word aligned address in text
    // - body, scan by 2 words at a time
    // - the last remaining part, < 2 word size
    let len = text.len();
    let ptr = text.as_ptr();
    let end_ptr = unsafe { ptr.add(len) };
    let usize_bytes = mem::size_of::<usize>();

    // search up to an aligned boundary
    let offset = ptr.align_offset(usize_bytes);
    let mut cur = ptr;
    if offset > 0 {
        let offset = cmp::min(offset, len);
        unsafe {
            cur = ptr.add(offset);
            let found_ptr = memchr_ptr_slow(x, ptr, cur);
            if found_ptr != ptr::null() {
                return found_ptr;
            }
        }
    }

    // search the body of the text
    let repeated_x = repeat_byte(x);

    unsafe {
        while cur <= end_ptr.sub(2 * usize_bytes) {
            let u = *(cur as *const usize);
            let v = *(cur.add(usize_bytes) as *const usize);

            // break if there is a matching byte
            let zu = contains_zero_byte(u ^ repeated_x);
            let zv = contains_zero_byte(v ^ repeated_x);
            if zu || zv {
                break;
            }

            cur = cur.add(2 * usize_bytes);
        }

        // find the byte after the point the body loop stopped
        memchr_ptr_slow(x, cur, end_ptr)
    }
}
