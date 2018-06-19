extern crate pattern_3;

use pattern_3::ext::{trim, trim_start};

#[test]
fn test_trim_slice_fn() {
    #[inline(never)]
    fn sw1(a: &[u64], b: u64) -> &[u64] {
        trim(a, |c: &u64| *c == b)
    }

    const E: &[u64] = &[];
    assert_eq!(&[2,3,4], sw1(&[1,2,3,4], 1));
    assert_eq!(&[4,3,2], sw1(&[4,3,2,1,1], 1));
    assert_eq!(&[1,5,2,8], sw1(&[1,5,2,8], 4));
    assert_eq!(E, sw1(&[6,6,6,6], 6));
    assert_eq!(E, sw1(E, 23));
    assert_eq!(E, sw1(&[24], 24));
    assert_eq!(&[5], sw1(&[5], 25));
    assert_eq!(&[75,6,77], sw1(&[6,6,75,6,77,6,6,6], 6));
}

#[test]
fn test_trim_string_char() {
    #[inline(never)]
    fn trim_string_char(s: &str, c: char) -> &str {
        trim(s, c)
    }

    assert_eq!("", trim_string_char("aaaaaaaa", 'a'));
    assert_eq!("bbaabb", trim_string_char("abbaabbaaa", 'a'));
    assert_eq!("baaab", trim_string_char("baaab", 'a'));
    assert_eq!("颫颫", trim_string_char("風風風颫颫風", '風'));
    assert_eq!("風風風颫颫風", trim_string_char("風風風颫颫風", '颫'));
    assert_eq!("", trim_string_char("風風風", '風'));
    assert_eq!("風風風", trim_string_char("風風風", '颫'));
    assert_eq!("", trim_string_char("", 'a'));
    assert_eq!("", trim_string_char("", '風'));
}

#[test]
fn test_trim_string_fn() {
    assert_eq!("", trim("abcdefg", |c: char| c.is_ascii()));
    assert_eq!("αbβcγdδeε", trim_start("aαbβcγdδeε", |c: char| c.is_ascii()));
    assert_eq!("abcdefg", trim("abcdefg", |c: char| !c.is_ascii()));
    assert_eq!("aαbβcγdδe", trim("aαbβcγdδeε", |c: char| !c.is_ascii()));
}
