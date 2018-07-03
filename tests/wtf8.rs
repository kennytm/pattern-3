extern crate pattern_3;

use pattern_3::Wtf8;
use pattern_3::ext::*;

#[test]
fn test_trim_start_low_surrogate() {
    unsafe {
        let pat = Wtf8::from_bytes_unchecked(b"\xed\xb0\x80");
        let a = Wtf8::from_bytes_unchecked(b"\x90\x80\x80aaa");
        assert_eq!(trim_start(a, pat), Wtf8::from_str("aaa"));

        let b = Wtf8::from_bytes_unchecked(b"\x90\x80\x80\xed\xb0\x80bbb");
        assert_eq!(trim_start(b, pat), Wtf8::from_str("bbb"));

        let c = Wtf8::from_bytes_unchecked(b"\xed\xb0\x80\xed\xb0\x80ccc");
        assert_eq!(trim_start(c, pat), Wtf8::from_str("ccc"));

        let d = Wtf8::from_bytes_unchecked(b"\xbf\xb0\x80ddd");
        assert_eq!(trim_start(d, pat), Wtf8::from_str("ddd"));

        let e = Wtf8::from_bytes_unchecked(b"\xe3\xb0\x80eee");
        assert_eq!(trim_start(e, pat), Wtf8::from_str("㰀eee"));
    }
}

#[test]
fn test_trim_start_high_surrogate() {
    unsafe {
        let pat = Wtf8::from_bytes_unchecked(b"\xed\xa0\x80");
        let a = Wtf8::from_bytes_unchecked(b"\xf0\x90\x80\x80");
        assert_eq!(trim_start(a, pat), Wtf8::from_bytes_unchecked(b"\x90\x80\x80"));

        let b = Wtf8::from_bytes_unchecked(b"\xed\xa0\x80bbb");
        assert_eq!(trim_start(b, pat), Wtf8::from_str("bbb"));

        let c = Wtf8::from_bytes_unchecked(b"\xed\xa0\x80\xf0\x90\x8f\xbfccc");
        assert_eq!(trim_start(c, pat), Wtf8::from_bytes_unchecked(b"\x90\x8f\xbfccc"));
    }
}

#[test]
fn test_trim_end_high_surrogate() {
    unsafe {
        let pat = Wtf8::from_bytes_unchecked(b"\xed\xa0\x80");
        let a = Wtf8::from_bytes_unchecked(b"aaa\xf0\x90\x80");
        assert_eq!(trim_end(a, pat), Wtf8::from_str("aaa"));

        let b = Wtf8::from_bytes_unchecked(b"bbb\xed\xa0\x80\xf0\x90\x80");
        assert_eq!(trim_end(b, pat), Wtf8::from_str("bbb"));

        let c = Wtf8::from_bytes_unchecked(b"ccc\xed\xa0\x80\xed\xa0\x80");
        assert_eq!(trim_end(c, pat), Wtf8::from_str("ccc"));

        let d = Wtf8::from_bytes_unchecked(b"ddd\xf0\x90\x8f");
        assert_eq!(trim_end(d, pat), Wtf8::from_str("ddd"));

        let e = Wtf8::from_bytes_unchecked(b"eee\xf0\x91\x80");
        assert_eq!(trim_end(e, pat), e);

        let f = Wtf8::from_bytes_unchecked(b"fff\xed\xb0\x80");
        assert_eq!(trim_end(f, pat), f);
    }
}

#[test]
fn test_trim_end_low_surrogate() {
    unsafe {
        let pat = Wtf8::from_bytes_unchecked(b"\xed\xb0\x80");
        let a = Wtf8::from_bytes_unchecked(b"\xf0\x90\x80\x80");
        assert_eq!(trim_end(a, pat), Wtf8::from_bytes_unchecked(b"\xf0\x90\x80"));

        let b = Wtf8::from_bytes_unchecked(b"bbb\xed\xb0\x80");
        assert_eq!(trim_end(b, pat), Wtf8::from_str("bbb"));

        let c = Wtf8::from_bytes_unchecked(b"ccc\xf4\x8f\xb0\x80\xed\xb0\x80");
        assert_eq!(trim_end(c, pat), Wtf8::from_bytes_unchecked(b"ccc\xf4\x8f\xb0"));
    }
}

#[test]
fn test_match_string_with_surrogates() {
    unsafe {
        let haystack = &Wtf8::from_str("\u{10000}a\u{10000}a\u{10000}\u{10000}")[2..16];
        // 0..3 = U+DC00
        // 3..4 = 'a'
        // 4..6 = U+D800
        // 6..8 = U+DC00
        // 8..9 = 'a'
        // 9..11 = U+D800
        // 11..13 = U+DC00
        // 13..16 = U+D800

        let pat = "a";
        let matched_pat = Wtf8::from_str(pat);
        assert_eq!(match_ranges(haystack, pat).collect::<Vec<_>>(), vec![
            (3..4, matched_pat),
            (8..9, matched_pat),
        ]);
        assert_eq!(rmatch_ranges(haystack, pat).collect::<Vec<_>>(), vec![
            (8..9, matched_pat),
            (3..4, matched_pat),
        ]);

        let pat = Wtf8::from_bytes_unchecked(b"\xed\xb0\x80a");
        assert_eq!(match_ranges(haystack, pat).collect::<Vec<_>>(), vec![
            (0..4, pat),
            (6..9, pat),
        ]);
        assert_eq!(rmatch_ranges(haystack, pat).collect::<Vec<_>>(), vec![
            (6..9, pat),
            (0..4, pat),
        ]);

        let pat = Wtf8::from_bytes_unchecked(b"a\xed\xa0\x80");
        assert_eq!(match_ranges(haystack, pat).collect::<Vec<_>>(), vec![
            (3..6, pat),
            (8..11, pat),
        ]);
        assert_eq!(rmatch_ranges(haystack, pat).collect::<Vec<_>>(), vec![
            (8..11, pat),
            (3..6, pat),
        ]);

        let pat = "\u{10000}";
        let matched_pat = Wtf8::from_str(pat);
        assert_eq!(match_ranges(haystack, pat).collect::<Vec<_>>(), vec![
            (4..8, matched_pat),
            (9..13, matched_pat),
        ]);
        assert_eq!(rmatch_ranges(haystack, pat).collect::<Vec<_>>(), vec![
            (9..13, matched_pat),
            (4..8, matched_pat),
        ]);

        let pat = Wtf8::from_bytes_unchecked(b"\xed\xa0\x80");
        assert_eq!(match_ranges(haystack, pat).collect::<Vec<_>>(), vec![
            (4..6, pat),
            (9..11, pat),
            (13..16, pat),
        ]);
        assert_eq!(rmatch_ranges(haystack, pat).collect::<Vec<_>>(), vec![
            (13..16, pat),
            (9..11, pat),
            (4..6, pat),
        ]);

        let pat = Wtf8::from_bytes_unchecked(b"\xed\xb0\x80");
        assert_eq!(match_ranges(haystack, pat).collect::<Vec<_>>(), vec![
            (0..3, pat),
            (6..8, pat),
            (11..13, pat),
        ]);
        assert_eq!(rmatch_ranges(haystack, pat).collect::<Vec<_>>(), vec![
            (11..13, pat),
            (6..8, pat),
            (0..3, pat),
        ]);

        let pat = Wtf8::from_bytes_unchecked(b"\xed\xb0\x80\xed\xa0\x80");
        assert_eq!(match_ranges(haystack, pat).collect::<Vec<_>>(), vec![
            (11..16, pat),
        ]);
        assert_eq!(rmatch_ranges(haystack, pat).collect::<Vec<_>>(), vec![
            (11..16, pat),
        ]);
    }
}
