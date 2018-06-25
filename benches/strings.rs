// Ensure our own implementation is not slower than the std implementation

#![feature(test)]

extern crate test;
extern crate pattern_3;

use test::Bencher;

macro_rules! make_test_inner {
    ($s:ident, $code:expr, $name:ident, $str:expr, $iters:expr) => {
        #[bench]
        fn $name(bencher: &mut Bencher) {
            let mut $s = $str;
            black_box(&mut $s);
            bencher.iter(|| for _ in 0..$iters { black_box($code); });
        }
    }
}

const LOREM_IPSUM: &str = "\
Lorem ipsum dolor sit amet, consectetur adipiscing elit. Suspendisse quis lorem sit amet dolor \
ultricies condimentum. Praesent iaculis purus elit, ac malesuada quam malesuada in. Duis sed orci \
eros. Suspendisse sit amet magna mollis, mollis nunc luctus, imperdiet mi. Integer fringilla non \
sem ut lacinia. Fusce varius tortor a risus porttitor hendrerit. Morbi mauris dui, ultricies nec \
tempus vel, gravida nec quam.

In est dui, tincidunt sed tempus interdum, adipiscing laoreet ante. Etiam tempor, tellus quis \
sagittis interdum, nulla purus mattis sem, quis auctor erat odio ac tellus. In nec nunc sit amet \
diam volutpat molestie at sed ipsum. Vestibulum laoreet consequat vulputate. Integer accumsan \
lorem ac dignissim placerat. Suspendisse convallis faucibus lorem. Aliquam erat volutpat. In vel \
eleifend felis. Sed suscipit nulla lorem, sed mollis est sollicitudin et. Nam fermentum egestas \
interdum. Curabitur ut nisi justo.

Sed sollicitudin ipsum tellus, ut condimentum leo eleifend nec. Cras ut velit ante. Phasellus nec \
mollis odio. Mauris molestie erat in arcu mattis, at aliquet dolor vehicula. Quisque malesuada \
lectus sit amet nisi pretium, a condimentum ipsum porta. Morbi at dapibus diam. Praesent egestas \
est sed risus elementum, eu rutrum metus ultrices. Etiam fermentum consectetur magna, id rutrum \
felis accumsan a. Aliquam ut pellentesque libero. Sed mi nulla, lobortis eu tortor id, suscipit \
ultricies neque. Morbi iaculis sit amet risus at iaculis. Praesent eget ligula quis turpis \
feugiat suscipit vel non arcu. Interdum et malesuada fames ac ante ipsum primis in faucibus. \
Aliquam sit amet placerat lorem.

Cras a lacus vel ante posuere elementum. Nunc est leo, bibendum ut facilisis vel, bibendum at \
mauris. Nullam adipiscing diam vel odio ornare, luctus adipiscing mi luctus. Nulla facilisi. \
Mauris adipiscing bibendum neque, quis adipiscing lectus tempus et. Sed feugiat erat et nisl \
lobortis pharetra. Donec vitae erat enim. Nullam sit amet felis et quam lacinia tincidunt. Aliquam \
suscipit dapibus urna. Sed volutpat urna in magna pulvinar volutpat. Phasellus nec tellus ac diam \
cursus accumsan.

Nam lectus enim, dapibus non nisi tempor, consectetur convallis massa. Maecenas eleifend dictum \
feugiat. Etiam quis mauris vel risus luctus mattis a a nunc. Nullam orci quam, imperdiet id \
vehicula in, porttitor ut nibh. Duis sagittis adipiscing nisl vitae congue. Donec mollis risus eu \
leo suscipit, varius porttitor nulla porta. Pellentesque ut sem nec nisi euismod vehicula. Nulla \
malesuada sollicitudin quam eu fermentum!";

macro_rules! make_test {
    ($name:ident, $s:ident, $code:expr) => {
        make_test!($name, $s, $code, 1);
    };
    ($name:ident, $s:ident, $code:expr, $iters:expr) => {
        mod $name {
            use test::Bencher;
            use test::black_box;

            // Short strings: 65 bytes each
            make_test_inner!($s, $code, short_ascii,
                "Mary had a little lamb, Little lamb Mary had a littl lamb, lamb!", $iters);
            make_test_inner!($s, $code, short_mixed,
                "ศไทย中华Việt Nam; Mary had a little lamb, Little lam!", $iters);
            make_test_inner!($s, $code, short_pile_of_poo,
                "💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩!", $iters);
            make_test_inner!($s, $code, long_lorem_ipsum, ::LOREM_IPSUM, $iters);
        }
    }
}

// std:
//     long_lorem_ipsum:    ~260 ns/iter
//     short_ascii:          ~20 ns/iter
//     short_mixed:          ~20 ns/iter
//     short_pile_of_pool:   ~20 ns/iter
// pat3:
//     long_lorem_ipsum:     ~50 ns/iter
//     short_ascii:          ~15 ns/iter
//     short_mixed:          ~15 ns/iter
//     short_pile_of_pool:   ~15 ns/iter
make_test!(std_contains_bang_char, s, s.contains('!'));
make_test!(pat3_contains_bang_char, s, ::pattern_3::ext::contains(s, '!'));

// std:
//     long_lorem_ipsum:    ~260 ns/iter
//     short_ascii:          ~20 ns/iter
//     short_mixed:          ~20 ns/iter
//     short_pile_of_pool:   ~20 ns/iter
// pat3:
//     long_lorem_ipsum:     ~50 ns/iter
//     short_ascii:          ~10 ns/iter
//     short_mixed:          ~10 ns/iter
//     short_pile_of_pool:   ~10 ns/iter
make_test!(std_find_underscore_char, s, s.find('_'));
make_test!(pat3_find_underscore_char, s, ::pattern_3::ext::find(s, '_'));

// std:
//     long_lorem_ipsum:    ~270 ns/iter
//     short_ascii:          ~20 ns/iter
//     short_mixed:          ~20 ns/iter
//     short_pile_of_pool:   ~20 ns/iter
// pat3:
//     long_lorem_ipsum:    ~200 ns/iter
//     short_ascii:          ~20 ns/iter
//     short_mixed:          ~20 ns/iter
//     short_pile_of_pool:   ~20 ns/iter
make_test!(std_rfind_underscore_char, s, s.rfind('_'));
make_test!(pat3_rfind_underscore_char, s, ::pattern_3::ext::rfind(s, '_'));

// std:
//     long_lorem_ipsum:    ~270 ns/iter
//     short_ascii:          ~10 ns/iter
//     short_mixed:          ~10 ns/iter
//     short_pile_of_pool:   ~20 ns/iter
// pat3:
//     long_lorem_ipsum:     ~55 ns/iter
//     short_ascii:          ~15 ns/iter
//     short_mixed:          ~15 ns/iter
//     short_pile_of_pool:   ~15 ns/iter
make_test!(std_find_zzz_char, s, s.find('\u{1F4A4}'));
make_test!(pat3_find_zzz_char, s, ::pattern_3::ext::find(s, '\u{1F4A4}'));

// std:
//     long_lorem_ipsum:    ~280 ns/iter
//     short_ascii:          ~20 ns/iter
//     short_mixed:          ~20 ns/iter
//     short_pile_of_pool:   ~20 ns/iter
// pat3:
//     long_lorem_ipsum:    ~220 ns/iter
//     short_ascii:          ~20 ns/iter
//     short_mixed:          ~20 ns/iter
//     short_pile_of_pool:   ~20 ns/iter
make_test!(std_rfind_zzz_char, s, s.rfind('\u{1F4A4}'));
make_test!(pat3_rfind_zzz_char, s, ::pattern_3::ext::rfind(s, '\u{1F4A4}'));

// std:
//     long_lorem_ipsum:    ~300 ns/iter
//     short_ascii:         ~300 ns/iter
//     short_mixed:         ~750 ns/iter
//     short_pile_of_pool:  ~840 ns/iter
// pat3:
//     long_lorem_ipsum:    ~180 ns/iter
//     short_ascii:         ~180 ns/iter
//     short_mixed:         ~180 ns/iter
//     short_pile_of_pool:  ~180 ns/iter
make_test!(std_starts_with_ascii_char, s, s.starts_with('/'), 256);
make_test!(pat3_starts_with_ascii_char, s, ::pattern_3::ext::starts_with(s, '/'), 256);

// std:
//     long_lorem_ipsum:    ~300 ns/iter
//     short_ascii:         ~300 ns/iter
//     short_mixed:         ~750 ns/iter
//     short_pile_of_pool:  ~840 ns/iter
// pat3:
//     long_lorem_ipsum:    ~270 ns/iter
//     short_ascii:         ~270 ns/iter
//     short_mixed:         ~270 ns/iter
//     short_pile_of_pool:  ~270 ns/iter
make_test!(std_starts_with_unichar, s, s.starts_with('\u{1F4A4}'), 256);
make_test!(pat3_starts_with_unichar, s, ::pattern_3::ext::starts_with(s, '\u{1F4A4}'), 256);

// std:
//     long_lorem_ipsum:    ~260 ns/iter
//     short_ascii:         ~260 ns/iter
//     short_mixed:         ~260 ns/iter
//     short_pile_of_pool:  ~260 ns/iter
// pat3:
//     long_lorem_ipsum:    ~180 ns/iter
//     short_ascii:         ~180 ns/iter
//     short_mixed:         ~180 ns/iter
//     short_pile_of_pool:  ~180 ns/iter
make_test!(std_ends_with_ascii_char, s, s.ends_with('/'), 256);
make_test!(pat3_ends_with_ascii_char, s, ::pattern_3::ext::ends_with(s, '/'), 256);

// std:
//     long_lorem_ipsum:    ~210 ns/iter
//     short_ascii:         ~220 ns/iter
//     short_mixed:         ~210 ns/iter
//     short_pile_of_pool:  ~210 ns/iter
// pat3:
//     long_lorem_ipsum:    ~290 ns/iter [*slower*]
//     short_ascii:         ~290 ns/iter [*slower*]
//     short_mixed:         ~290 ns/iter [*slower*]
//     short_pile_of_pool:  ~290 ns/iter [*slower*]
make_test!(std_ends_with_unichar, s, s.ends_with('\u{1F4A4}'), 256);
make_test!(pat3_ends_with_unichar, s, ::pattern_3::ext::ends_with(s, '\u{1F4A4}'), 256);

// std:
//     long_lorem_ipsum:    ~130 ns/iter
//     short_ascii:         ~130 ns/iter
//     short_mixed:         ~160 ns/iter
//     short_pile_of_pool: ~1500 ns/iter
// pat3:
//     long_lorem_ipsum:     ~60 ns/iter
//     short_ascii:          ~60 ns/iter
//     short_mixed:         ~100 ns/iter
//     short_pile_of_pool: ~1100 ns/iter
make_test!(std_trim_left_unichar, s, s.trim_left_matches('💩'), 16);
make_test!(pat3_trim_left_unichar, s, ::pattern_3::ext::trim_start(s, '💩'), 16);

// std:
//     long_lorem_ipsum:     ~60 ns/iter
//     short_ascii:          ~60 ns/iter
//     short_mixed:          ~60 ns/iter
//     short_pile_of_pool:  ~100 ns/iter
// pat3:
//     long_lorem_ipsum:     ~70 ns/iter [*slower*]
//     short_ascii:          ~70 ns/iter [*slower*]
//     short_mixed:          ~70 ns/iter [*slower*]
//     short_pile_of_pool:  ~130 ns/iter [*slower*]
make_test!(std_trim_right_unichar, s, s.trim_right_matches('!'), 16);
make_test!(pat3_trim_right_unichar, s, ::pattern_3::ext::trim_end(s, '!'), 16);

make_test!(std_trim_both_unichar, s, s.trim_matches('💩'), 16);
make_test!(pat3_trim_both_unichar, s, ::pattern_3::ext::trim(s, '💩'), 16);

// std:
//     long_lorem_ipsum:   ~1100 ns/iter
//     short_ascii:        ~1100 ns/iter
//     short_mixed:        ~1100 ns/iter
//     short_pile_of_pool: ~2600 ns/iter
// pat3:
//     long_lorem_ipsum:     ~30 ns/iter
//     short_ascii:          ~40 ns/iter
//     short_mixed:          ~40 ns/iter
//     short_pile_of_pool:  ~140 ns/iter
make_test!(std_trim_left_unicode_string, s, s.trim_left_matches("💩💩"), 16);
make_test!(pat3_trim_left_unicode_string, s, ::pattern_3::ext::trim_start(s, "💩💩"), 16);

// std:
//     long_lorem_ipsum:    ~500 ns/iter
//     short_ascii:         ~380 ns/iter
//     short_mixed:         ~500 ns/iter
//     short_pile_of_pool:  ~430 ns/iter
// pat3:
//     long_lorem_ipsum:     ~60 ns/iter
//     short_ascii:          ~40 ns/iter
//     short_mixed:          ~60 ns/iter
//     short_pile_of_pool:   ~40 ns/iter
make_test!(std_trim_right_ascii_string, s, s.trim_right_matches("m!"), 16);
make_test!(pat3_trim_right_ascii_string, s, ::pattern_3::ext::trim_end(s, "m!"), 16);

// std:
//     long_lorem_ipsum:    ~250 ns/iter
//     short_ascii:         ~210 ns/iter
//     short_mixed:         ~630 ns/iter
//     short_pile_of_pool: ~1300 ns/iter
// pat3:
//     long_lorem_ipsum:    ~110 ns/iter
//     short_ascii:         ~100 ns/iter
//     short_mixed:         ~400 ns/iter
//     short_pile_of_pool: ~1100 ns/iter
make_test!(std_find_fn, s, s.find(|c: char| c == ' '), 16);
make_test!(pat3_find_fn, s, ::pattern_3::ext::find(s, |c: char| c == ' '), 16);

// std:
//     long_lorem_ipsum:    ~250 ns/iter
//     short_ascii:         ~140 ns/iter
//     short_mixed:         ~120 ns/iter
//     short_pile_of_pool: ~1100 ns/iter
// pat3:
//     long_lorem_ipsum:    ~220 ns/iter
//     short_ascii:         ~160 ns/iter [*slower*]
//     short_mixed:         ~160 ns/iter [*slower*]
//     short_pile_of_pool: ~1000 ns/iter
make_test!(std_rfind_fn, s, s.rfind(|c: char| c == ' '), 16);
make_test!(pat3_rfind_fn, s, ::pattern_3::ext::rfind(s, |c: char| c == ' '), 16);

// std:
//     long_lorem_ipsum:   ~4500 ns/iter
//     short_ascii:         ~120 ns/iter
//     short_mixed:           ~6 ns/iter
//     short_pile_of_pool:    ~7 ns/iter
// pat3:
//     long_lorem_ipsum:   ~1700 ns/iter
//     short_ascii:          ~50 ns/iter
//     short_mixed:           ~5 ns/iter
//     short_pile_of_pool:    ~6 ns/iter
make_test!(std_trim_left_ascii_char, s, s.trim_left_matches(|c: char| c.is_ascii()));
make_test!(pat3_trim_left_ascii_char, s, ::pattern_3::ext::trim_start(s, |c: char| c.is_ascii()));

// std:
//     long_lorem_ipsum:   ~2900 ns/iter
//     short_ascii:          ~80 ns/iter
//     short_mixed:          ~60 ns/iter
//     short_pile_of_pool:    ~6 ns/iter
// pat3:
//     long_lorem_ipsum:   ~1300 ns/iter
//     short_ascii:          ~50 ns/iter
//     short_mixed:          ~40 ns/iter
//     short_pile_of_pool:    ~5 ns/iter
make_test!(std_trim_right_ascii_char, s, s.trim_right_matches(|c: char| c.is_ascii()));
make_test!(pat3_trim_right_ascii_char, s, ::pattern_3::ext::trim_end(s, |c: char| c.is_ascii()));

make_test!(std_trim_both_ascii_char, s, s.trim_matches(|c: char| c.is_ascii()));
make_test!(pat3_trim_both_ascii_char, s, ::pattern_3::ext::trim(s, |c: char| c.is_ascii()));

// std:
//     long_lorem_ipsum:   ~4800 ns/iter
//     short_ascii:         ~110 ns/iter
//     short_mixed:         ~110 ns/iter
//     short_pile_of_pool:   ~90 ns/iter
// pat3:
//     long_lorem_ipsum:   ~3800 ns/iter
//     short_ascii:          ~90 ns/iter
//     short_mixed:          ~90 ns/iter
//     short_pile_of_pool:  ~100 ns/iter
make_test!(std_contains_bang_str, s, s.contains("!"));
make_test!(pat3_contains_bang_str, s, ::pattern_3::ext::contains(s, "!"));

// std:
//     long_lorem_ipsum:   ~5900 ns/iter
//     short_ascii:         ~180 ns/iter
//     short_mixed:         ~160 ns/iter
//     short_pile_of_pool:  ~100 ns/iter
// pat3:
//     long_lorem_ipsum:   ~4700 ns/iter
//     short_ascii:         ~130 ns/iter
//     short_mixed:         ~130 ns/iter
//     short_pile_of_pool:  ~100 ns/iter
make_test!(std_match_indices_a_str, s, s.match_indices("a").count());
make_test!(pat3_match_indices_a_str, s, ::pattern_3::ext::match_indices(s, "a").count());

// std:
//     long_lorem_ipsum:   ~5900 ns/iter
//     short_ascii:         ~190 ns/iter
//     short_mixed:         ~150 ns/iter
//     short_pile_of_pool:  ~100 ns/iter
// pat3:
//     long_lorem_ipsum:   ~5400 ns/iter
//     short_ascii:         ~180 ns/iter
//     short_mixed:         ~150 ns/iter
//     short_pile_of_pool:  ~100 ns/iter
make_test!(std_split_a_str, s, s.split("a").count());
make_test!(pat3_split_a_str, s, ::pattern_3::ext::split(s, "a").count());

// std:
//     long_lorem_ipsum:   ~9300 ns/iter
//     short_ascii:         ~200 ns/iter
//     short_mixed:         ~180 ns/iter
//     short_pile_of_pool:  ~100 ns/iter
// pat3:
//     long_lorem_ipsum:   ~8300 ns/iter
//     short_ascii:         ~200 ns/iter
//     short_mixed:         ~180 ns/iter
//     short_pile_of_pool:  ~100 ns/iter
make_test!(std_split_space_str, s, s.split(" ").count());
make_test!(pat3_split_space_str, s, ::pattern_3::ext::split(s, " ").count());

// std:
//     long_lorem_ipsum:   ~2500 ns/iter
//     short_ascii:         ~100 ns/iter
//     short_mixed:          ~90 ns/iter
//     short_pile_of_pool:   ~70 ns/iter
// pat3:
//     long_lorem_ipsum:   ~1800 ns/iter
//     short_ascii:          ~90 ns/iter
//     short_mixed:          ~70 ns/iter
//     short_pile_of_pool:   ~60 ns/iter
make_test!(std_split_ad_str, s, s.split("ad").count());
make_test!(pat3_split_ad_str, s, ::pattern_3::ext::split(s, "ad").count());

// std:     ~70 ns/iter
// pat3:    ~40 ns/iter
#[bench]
fn std_split_unicode_ascii(b: &mut Bencher) {
    let s = "ประเทศไทย中华Việt Namประเทศไทย中华Việt Nam";
    b.iter(|| assert_eq!(s.split('V').count(), 3));
}
#[bench]
fn pat3_split_unicode_ascii(b: &mut Bencher) {
    let s = "ประเทศไทย中华Việt Namประเทศไทย中华Việt Nam";
    b.iter(|| assert_eq!(pattern_3::ext::split(s, 'V').count(), 3));
}

// std:     ~110 ns/iter
// pat3:    ~100 ns/iter
#[bench]
fn std_split_ascii(b: &mut Bencher) {
    let s = "Mary had a little lamb, Little lamb, little-lamb.";
    let len = s.split(' ').count();
    b.iter(|| assert_eq!(s.split(' ').count(), len));
}
#[bench]
fn pat3_split_ascii(b: &mut Bencher) {
    let s = "Mary had a little lamb, Little lamb, little-lamb.";
    let len = s.split(' ').count();
    b.iter(|| assert_eq!(pattern_3::ext::split(s, ' ').count(), len));
}

// std:     ~80 ns/iter
// pat3:    ~50 ns/iter
#[bench]
fn std_split_extern_fn(b: &mut Bencher) {
    let s = "Mary had a little lamb, Little lamb, little-lamb.";
    let len = s.split(' ').count();
    fn pred(c: char) -> bool { c == ' ' }
    b.iter(|| assert_eq!(s.split(pred).count(), len));
}
#[bench]
fn pat3_split_extern_fn(b: &mut Bencher) {
    let s = "Mary had a little lamb, Little lamb, little-lamb.";
    let len = s.split(' ').count();
    fn pred(c: char) -> bool { c == ' ' }
    b.iter(|| assert_eq!(pattern_3::ext::split(s, pred).count(), len));
}

// std:     ~80 ns/iter
// pat3:    ~50 ns/iter
#[bench]
fn std_split_closure(b: &mut Bencher) {
    let s = "Mary had a little lamb, Little lamb, little-lamb.";
    let len = s.split(' ').count();
    b.iter(|| assert_eq!(s.split(|c: char| c == ' ').count(), len));
}
#[bench]
fn pat3_split_closure(b: &mut Bencher) {
    let s = "Mary had a little lamb, Little lamb, little-lamb.";
    let len = s.split(' ').count();
    b.iter(|| assert_eq!(::pattern_3::ext::split(s, |c: char| c == ' ').count(), len));
}

// std:     ~140 ns/iter
// pat3:    ~160 ns/iter [*slower*]
#[bench]
fn std_split_slice(b: &mut Bencher) {
    let s = "Mary had a little lamb, Little lamb, little-lamb.";
    let len = s.split(' ').count();
    let c: &[char] = &[' '];
    b.iter(|| assert_eq!(s.split(c).count(), len));
}
#[bench]
fn pat3_split_slice(b: &mut Bencher) {
    let s = "Mary had a little lamb, Little lamb, little-lamb.";
    let len = s.split(' ').count();
    let c: &[char] = &[' '];
    b.iter(|| assert_eq!(::pattern_3::ext::split(s, c).count(), len));
}

// std:     ~33 ns/iter
// pat3:    ~32 ns/iter
#[bench]
fn std_contains_short_short(b: &mut Bencher) {
    let haystack = "Lorem ipsum dolor sit amet, consectetur adipiscing elit.";
    let needle = "sit";
    b.iter(|| assert!(haystack.contains(needle)))
}
#[bench]
fn pat3_contains_short_short(b: &mut Bencher) {
    let haystack = "Lorem ipsum dolor sit amet, consectetur adipiscing elit.";
    let needle = "sit";
    b.iter(|| assert!(::pattern_3::ext::contains(haystack, needle)))
}

// std:     ~800 ns/iter
// pat3:    ~700 ns/iter
#[bench]
fn std_contains_short_long(b: &mut Bencher) {
    let haystack = LOREM_IPSUM;
    let needle = "english";
    b.iter(|| assert!(!haystack.contains(needle)));
}
#[bench]
fn pat3_contains_short_long(b: &mut Bencher) {
    let haystack = LOREM_IPSUM;
    let needle = "english";
    b.iter(|| assert!(!::pattern_3::ext::contains(haystack, needle)));
}

// std:     ~170 ns/iter
// pat3:    ~180 ns/iter
#[bench]
fn std_contains_bad_naive(b: &mut Bencher) {
    let haystack = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let needle = "aaaaaaaab";

    b.iter(|| assert!(!haystack.contains(needle)));
}
#[bench]
fn pat3_contains_bad_naive(b: &mut Bencher) {
    let haystack = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let needle = "aaaaaaaab";
    b.iter(|| assert!(!::pattern_3::ext::contains(haystack, needle)));
}

// std:     ~280 ns/iter
// pat3:    ~280 ns/iter
#[bench]
fn std_contains_equal(b: &mut Bencher) {
    let haystack = "Lorem ipsum dolor sit amet, consectetur adipiscing elit.";
    let needle = "Lorem ipsum dolor sit amet, consectetur adipiscing elit.";
    b.iter(|| assert!(haystack.contains(needle)))
}
#[bench]
fn pat3_contains_equal(b: &mut Bencher) {
    let haystack = "Lorem ipsum dolor sit amet, consectetur adipiscing elit.";
    let needle = "Lorem ipsum dolor sit amet, consectetur adipiscing elit.";
    b.iter(|| assert!(::pattern_3::ext::contains(haystack, needle)))
}
