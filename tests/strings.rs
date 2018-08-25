extern crate pattern_3;

use pattern_3::*;
use pattern_3::ext::*;

#[test]
fn test_find() {
    assert_eq!(find("hello", 'l'), Some(2));
    assert_eq!(find("hello", |c:char| c == 'o'), Some(4));
    assert!(find("hello", 'x').is_none());
    assert!(find("hello", |c:char| c == 'x').is_none());
    assert_eq!(find("ประเทศไทย中华Việt Nam", '华'), Some(30));
    assert_eq!(find("ประเทศไทย中华Việt Nam", |c: char| c == '华'), Some(30));
}

#[test]
fn test_rfind() {
    assert_eq!(rfind("hello", 'l'), Some(3));
    assert_eq!(rfind("hello", |c:char| c == 'o'), Some(4));
    assert!(rfind("hello", 'x').is_none());
    assert!(rfind("hello", |c:char| c == 'x').is_none());
    assert_eq!(rfind("ประเทศไทย中华Việt Nam", '华'), Some(30));
    assert_eq!(rfind("ประเทศไทย中华Việt Nam", |c: char| c == '华'), Some(30));
}

#[test]
fn test_trim_left_matches() {
    let v: &[char] = &[];
    assert_eq!(trim_start(" *** foo *** ", v), " *** foo *** ");
    let chars: &[char] = &['*', ' '];
    assert_eq!(trim_start(" *** foo *** ", chars), "foo *** ");
    assert_eq!(trim_start(" ***  *** ", chars), "");
    assert_eq!(trim_start("foo *** ", chars), "foo *** ");

    assert_eq!(trim_start("11foo1bar11", '1'), "foo1bar11");
    let chars: &[char] = &['1', '2'];
    assert_eq!(trim_start("12foo1bar12", chars), "foo1bar12");
    assert_eq!(trim_start("123foo1bar123", |c: char| c.is_numeric()), "foo1bar123");
}

#[test]
fn test_trim_right_matches() {
    let v: &[char] = &[];
    assert_eq!(trim_end(" *** foo *** ", v), " *** foo *** ");
    let chars: &[char] = &['*', ' '];
    assert_eq!(trim_end(" *** foo *** ", chars), " *** foo");
    assert_eq!(trim_end(" ***  *** ", chars), "");
    assert_eq!(trim_end(" *** foo", chars), " *** foo");

    assert_eq!(trim_end("11foo1bar11", '1'), "11foo1bar");
    let chars: &[char] = &['1', '2'];
    assert_eq!(trim_end("12foo1bar12", chars), "12foo1bar");
    assert_eq!(trim_end("123foo1bar123", |c: char| c.is_numeric()), "123foo1bar");
}

#[test]
fn test_trim_matches() {
    let v: &[char] = &[];
    assert_eq!(trim(" *** foo *** ", v), " *** foo *** ");
    let chars: &[char] = &['*', ' '];
    assert_eq!(trim(" *** foo *** ", chars), "foo");
    assert_eq!(trim(" ***  *** ", chars), "");
    assert_eq!(trim("foo", chars), "foo");

    assert_eq!(trim("11foo1bar11", '1'), "foo1bar");
    let chars: &[char] = &['1', '2'];
    assert_eq!(trim("12foo1bar12", chars), "foo1bar");
    assert_eq!(trim("123foo1bar123", |c: char| c.is_numeric()), "foo1bar");
}

#[test]
fn test_contains_char() {
    assert!(contains("abc", 'b'));
    assert!(contains("a", 'a'));
    assert!(!contains("abc", 'd'));
    assert!(!contains("", 'a'));
}

#[test]
fn test_splitn_char_iterator() {
    let data = "\nMäry häd ä little lämb\nLittle lämb\n";

    let split: Vec<&str> = splitn(data, 4, ' ').collect();
    assert_eq!(split, ["\nMäry", "häd", "ä", "little lämb\nLittle lämb\n"]);

    let split: Vec<&str> = splitn(data, 4, |c: char| c == ' ').collect();
    assert_eq!(split, ["\nMäry", "häd", "ä", "little lämb\nLittle lämb\n"]);

    // Unicode
    let split: Vec<&str> = splitn(data, 4, 'ä').collect();
    assert_eq!(split, ["\nM", "ry h", "d ", " little lämb\nLittle lämb\n"]);

    let split: Vec<&str> = splitn(data, 4, |c: char| c == 'ä').collect();
    assert_eq!(split, ["\nM", "ry h", "d ", " little lämb\nLittle lämb\n"]);
}

#[test]
fn test_rsplitn_char_iterator() {
    let data = "\nMäry häd ä little lämb\nLittle lämb\n";

    let mut split: Vec<&str> = rsplitn(data, 4, ' ').collect();
    split.reverse();
    assert_eq!(split, ["\nMäry häd ä", "little", "lämb\nLittle", "lämb\n"]);

    let mut split: Vec<&str> = rsplitn(data, 4, |c: char| c == ' ').collect();
    split.reverse();
    assert_eq!(split, ["\nMäry häd ä", "little", "lämb\nLittle", "lämb\n"]);

    // Unicode
    let mut split: Vec<&str> = rsplitn(data, 4, 'ä').collect();
    split.reverse();
    assert_eq!(split, ["\nMäry häd ", " little l", "mb\nLittle l", "mb\n"]);

    let mut split: Vec<&str> = rsplitn(data, 4, |c: char| c == 'ä').collect();
    split.reverse();
    assert_eq!(split, ["\nMäry häd ", " little l", "mb\nLittle l", "mb\n"]);
}

#[test]
fn test_split_char_iterator_no_trailing() {
    let data = "\nMäry häd ä little lämb\nLittle lämb\n";

    let split: Vec<&str> = split(data, '\n').collect();
    assert_eq!(split, ["", "Märy häd ä little lämb", "Little lämb", ""]);

    let split: Vec<&str> = split_terminator(data, '\n').collect();
    assert_eq!(split, ["", "Märy häd ä little lämb", "Little lämb"]);
}

#[test]
fn test_rsplit() {
    let data = "\nMäry häd ä little lämb\nLittle lämb\n";

    let split: Vec<&str> = rsplit(data, ' ').collect();
    assert_eq!(split, ["lämb\n", "lämb\nLittle", "little", "ä", "häd", "\nMäry"]);

    let split: Vec<&str> = rsplit(data, "lämb").collect();
    assert_eq!(split, ["\n", "\nLittle ", "\nMäry häd ä little "]);

    let split: Vec<&str> = rsplit(data, |c: char| c == 'ä').collect();
    assert_eq!(split, ["mb\n", "mb\nLittle l", " little l", "d ", "ry h", "\nM"]);
}

#[test]
fn test_rsplitn() {
    let data = "\nMäry häd ä little lämb\nLittle lämb\n";

    let split: Vec<&str> = rsplitn(data, 2, ' ').collect();
    assert_eq!(split, ["lämb\n", "\nMäry häd ä little lämb\nLittle"]);

    let split: Vec<&str> = rsplitn(data, 2, "lämb").collect();
    assert_eq!(split, ["\n", "\nMäry häd ä little lämb\nLittle "]);

    let split: Vec<&str> = rsplitn(data, 2, |c: char| c == 'ä').collect();
    assert_eq!(split, ["mb\n", "\nMäry häd ä little lämb\nLittle l"]);
}

#[test]
fn test_split_char_iterator() {
    let data = "\nMäry häd ä little lämb\nLittle lämb\n";

    let s: Vec<&str> = split(data, ' ').collect();
    assert_eq!(s, ["\nMäry", "häd", "ä", "little", "lämb\nLittle", "lämb\n"]);

    let mut rsplit: Vec<&str> = split(data, ' ').rev().collect();
    rsplit.reverse();
    assert_eq!(rsplit, ["\nMäry", "häd", "ä", "little", "lämb\nLittle", "lämb\n"]);

    let s: Vec<&str> = split(data, |c: char| c == ' ').collect();
    assert_eq!(s, ["\nMäry", "häd", "ä", "little", "lämb\nLittle", "lämb\n"]);

    let mut rsplit: Vec<&str> = split(data, |c: char| c == ' ').rev().collect();
    rsplit.reverse();
    assert_eq!(rsplit, ["\nMäry", "häd", "ä", "little", "lämb\nLittle", "lämb\n"]);

    // Unicode
    let s: Vec<&str> = split(data, 'ä').collect();
    assert_eq!(s, ["\nM", "ry h", "d ", " little l", "mb\nLittle l", "mb\n"]);

    let mut rsplit: Vec<&str> = split(data, 'ä').rev().collect();
    rsplit.reverse();
    assert_eq!(rsplit, ["\nM", "ry h", "d ", " little l", "mb\nLittle l", "mb\n"]);

    let s: Vec<&str> = split(data, |c: char| c == 'ä').collect();
    assert_eq!(s, ["\nM", "ry h", "d ", " little l", "mb\nLittle l", "mb\n"]);

    let mut rsplit: Vec<&str> = split(data, |c: char| c == 'ä').rev().collect();
    rsplit.reverse();
    assert_eq!(rsplit, ["\nM", "ry h", "d ", " little l", "mb\nLittle l", "mb\n"]);
}

#[test]
fn test_rev_split_char_iterator_no_trailing() {
    let data = "\nMäry häd ä little lämb\nLittle lämb\n";

    let mut split: Vec<&str> = split(data, '\n').rev().collect();
    split.reverse();
    assert_eq!(split, ["", "Märy häd ä little lämb", "Little lämb", ""]);

    let mut split: Vec<&str> = split_terminator(data, '\n').rev().collect();
    split.reverse();
    assert_eq!(split, ["", "Märy häd ä little lämb", "Little lämb"]);
}


macro_rules! generate_iterator_test {
    {
        $name:ident {
            $(
                ($($arg:expr),*) -> [$($t:tt)*];
            )*
        }
        with $fwd:expr, $bwd:expr;
    } => {
        #[test]
        fn $name() {
            $(
                {
                    let res = vec![$($t)*];

                    let fwd_vec: Vec<_> = ($fwd)($($arg),*).collect();
                    assert_eq!(fwd_vec, res);

                    let mut bwd_vec: Vec<_> = ($bwd)($($arg),*).collect();
                    bwd_vec.reverse();
                    assert_eq!(bwd_vec, res);
                }
            )*
        }
    };
    {
        $name:ident {
            $(
                ($($arg:expr),*) -> [$($t:tt)*];
            )*
        }
        with $fwd:expr;
    } => {
        #[test]
        fn $name() {
            $(
                {
                    let res = vec![$($t)*];

                    let fwd_vec: Vec<_> = ($fwd)($($arg),*).collect();
                    assert_eq!(fwd_vec, res);
                }
            )*
        }
    }
}

generate_iterator_test! {
    double_ended_split {
        ("foo.bar.baz", '.') -> ["foo", "bar", "baz"];
        ("foo::bar::baz", "::") -> ["foo", "bar", "baz"];
    }
    with split, rsplit;
}

generate_iterator_test! {
    double_ended_split_terminator {
        ("foo;bar;baz;", ';') -> ["foo", "bar", "baz"];
    }
    with split_terminator, rsplit_terminator;
}

generate_iterator_test! {
    double_ended_matches {
        ("a1b2c3", char::is_numeric) -> ["1", "2", "3"];
    }
    with matches, rmatches;
}

generate_iterator_test! {
    double_ended_match_indices {
        ("a1b2c3", char::is_numeric) -> [(1, "1"), (3, "2"), (5, "3")];
    }
    with match_indices, rmatch_indices;
}

generate_iterator_test! {
    not_double_ended_splitn {
        ("foo::bar::baz", 2, "::") -> ["foo", "bar::baz"];
    }
    with splitn;
}

generate_iterator_test! {
    not_double_ended_rsplitn {
        ("foo::bar::baz", 2, "::") -> ["baz", "foo::bar"];
    }
    with rsplitn;
}

#[test]
fn test_edge_cases() {
    assert_eq!(split("", ',').collect::<Vec<_>>(), vec![""]);
    assert_eq!(split_terminator("", ',').collect::<Vec<_>>(), Vec::<&str>::new());
    assert_eq!(rsplit("", ',').collect::<Vec<_>>(), vec![""]);
    assert_eq!(rsplit_terminator("", ',').collect::<Vec<_>>(), Vec::<&str>::new());

    assert_eq!(split(",", ',').collect::<Vec<_>>(), vec!["", ""]);
    assert_eq!(split_terminator(",", ',').collect::<Vec<_>>(), vec![""]);
    assert_eq!(rsplit(",", ',').collect::<Vec<_>>(), vec!["", ""]);
    assert_eq!(rsplit_terminator(",", ',').collect::<Vec<_>>(), vec![""]);

    assert_eq!(split("?", ',').collect::<Vec<_>>(), vec!["?"]);
    assert_eq!(split_terminator("?", ',').collect::<Vec<_>>(), vec!["?"]);
    assert_eq!(rsplit("?", ',').collect::<Vec<_>>(), vec!["?"]);
    assert_eq!(rsplit_terminator("?", ',').collect::<Vec<_>>(), vec!["?"]);

    assert_eq!(splitn("a.b.c.d", 0, '.').collect::<Vec<_>>(), Vec::<&str>::new());
    assert_eq!(rsplitn("a.b.c.d", 0, '.').collect::<Vec<_>>(), Vec::<&str>::new());
    assert_eq!(splitn("a.b.c.d", 1, '.').collect::<Vec<_>>(), vec!["a.b.c.d"]);
    assert_eq!(rsplitn("a.b.c.d", 1, '.').collect::<Vec<_>>(), vec!["a.b.c.d"]);
    assert_eq!(splitn("a.b.c.d", 2, '.').collect::<Vec<_>>(), vec!["a", "b.c.d"]);
    assert_eq!(rsplitn("a.b.c.d", 2, '.').collect::<Vec<_>>(), vec!["d", "a.b.c"]);
    assert_eq!(splitn("a.b.c.d", !0, '.').collect::<Vec<_>>(), vec!["a", "b", "c", "d"]);
    assert_eq!(rsplitn("a.b.c.d", !0, '.').collect::<Vec<_>>(), vec!["d", "c", "b", "a"]);
}

#[test]
fn test_find_str() {
    // byte positions
    assert_eq!(find("", ""), Some(0));
    assert!(find("banana", "apple pie").is_none());

    let data = "abcabc";
    assert_eq!(find(&data[0..6], "ab"), Some(0));
    assert_eq!(find(&data[2..6], "ab"), Some(3 - 2));
    assert!(find(&data[2..4], "ab").is_none());

    let string = "ประเทศไทย中华Việt Nam";
    let mut data = String::from(string);
    data.push_str(string);
    assert!(find(&*data, "ไท华").is_none());
    assert_eq!(find(&data[0..43], ""), Some(0));
    assert_eq!(find(&data[6..43], ""), Some(6 - 6));

    assert_eq!(find(&data[0..43], "ประ"), Some( 0));
    assert_eq!(find(&data[0..43], "ทศไ"), Some(12));
    assert_eq!(find(&data[0..43], "ย中"), Some(24));
    assert_eq!(find(&data[0..43], "iệt"), Some(34));
    assert_eq!(find(&data[0..43], "Nam"), Some(40));

    assert_eq!(find(&data[43..86], "ประ"), Some(43 - 43));
    assert_eq!(find(&data[43..86], "ทศไ"), Some(55 - 43));
    assert_eq!(find(&data[43..86], "ย中"), Some(67 - 43));
    assert_eq!(find(&data[43..86], "iệt"), Some(77 - 43));
    assert_eq!(find(&data[43..86], "Nam"), Some(83 - 43));

    // find every substring -- assert that it finds it, or an earlier occurrence.
    let string = "Việt Namacbaabcaabaaba";
    for (i, ci) in string.char_indices() {
        let ip = i + ci.len_utf8();
        for j in string[ip..].char_indices()
                             .map(|(i, _)| i)
                             .chain(Some(string.len() - ip))
        {
            let pat = &string[i..ip + j];
            assert!(match find(string, pat) {
                None => false,
                Some(x) => x <= i,
            });
            assert!(match rfind(string, pat) {
                None => false,
                Some(x) => x >= i,
            });
        }
    }
}

#[test]
fn test_starts_with() {
    assert!(starts_with("", ""));
    assert!(starts_with("abc", ""));
    assert!(starts_with("abc", "a"));
    assert!(!starts_with("a", "abc"));
    assert!(!starts_with("", "abc"));
    assert!(!starts_with("ödd", "-"));
    assert!(starts_with("ödd", "öd"));
}

#[test]
fn test_ends_with() {
    assert!(ends_with("", ""));
    assert!(ends_with("abc", ""));
    assert!(ends_with("abc", "c"));
    assert!(!ends_with("a", "abc"));
    assert!(!ends_with("", "abc"));
    assert!(!ends_with("ddö", "-"));
    assert!(ends_with("ddö", "dö"));
}

#[test]
fn test_contains() {
    assert!(contains("abcde", "bcd"));
    assert!(contains("abcde", "abcd"));
    assert!(contains("abcde", "bcde"));
    assert!(contains("abcde", ""));
    assert!(contains("", ""));
    assert!(!contains("abcde", "def"));
    assert!(!contains("", "a"));

    let data = "ประเทศไทย中华Việt Nam";
    assert!(contains(data, "ประเ"));
    assert!(contains(data, "ะเ"));
    assert!(contains(data, "中华"));
    assert!(!contains(data, "ไท华"));
}

#[test]
fn test_splitator() {
    fn t(s: &str, sep: &str, u: &[&str]) {
        let v: Vec<&str> = split(s, sep).collect();
        assert_eq!(v, u);
    }
    t("--1233345--", "12345", &["--1233345--"]);
    t("abc::hello::there", "::", &["abc", "hello", "there"]);
    t("::hello::there", "::", &["", "hello", "there"]);
    t("hello::there::", "::", &["hello", "there", ""]);
    t("::hello::there::", "::", &["", "hello", "there", ""]);
    t("ประเทศไทย中华Việt Nam", "中华", &["ประเทศไทย", "Việt Nam"]);
    t("zzXXXzzYYYzz", "zz", &["", "XXX", "YYY", ""]);
    t("zzXXXzYYYz", "XXX", &["zz", "zYYYz"]);
    t(".XXX.YYY.", ".", &["", "XXX", "YYY", ""]);
    t("", ".", &[""]);
    t("zz", "zz", &["",""]);
    t("ok", "z", &["ok"]);
    t("zzz", "zz", &["","z"]);
    t("zzzzz", "zz", &["","","z"]);
}

#[test]
fn test_pattern_deref_forward() {
    let data = "aabcdaa";
    assert!(contains(data, "bcd"));
    assert!(contains(data, &"bcd"));
    assert!(contains(data, &"bcd".to_string()));
}

#[test]
fn test_empty_match_indices() {
    let data = "aä中!";
    let vec: Vec<_> = match_indices(data, "").collect();
    assert_eq!(vec, [(0, ""), (1, ""), (3, ""), (6, ""), (7, "")]);
}

fn check_contains_all_substrings(s: &str) {
    assert!(contains(s, ""));
    for i in 0..s.len() {
        for j in i+1..s.len() + 1 {
            assert!(contains(s, &s[i..j]));
        }
    }
}

#[test]
fn strslice_issue_16589() {
    assert!(contains("bananas", "nana"));

    // prior to the fix for #16589, x.contains("abcdabcd") returned false
    // test all substrings for good measure
    check_contains_all_substrings("012345678901234567890123456789bcdabcdabcd");
}

#[test]
fn test_strslice_contains() {
    let x = "There are moments, Jeeves, when one asks oneself, 'Do trousers matter?'";
    check_contains_all_substrings(x);
}

#[test]
fn strslice_issue_16878() {
    assert!(!contains("1234567ah012345678901ah", "hah"));
    assert!(!contains("00abc01234567890123456789abc", "bcabc"));
}

#[test]
fn starts_with_in_unicode() {
    assert!(!starts_with("├── Cargo.toml", "# "));
}

#[test]
fn starts_short_long() {
    assert!(!starts_with("", "##"));
    assert!(!starts_with("##", "####"));
    assert!(starts_with("####", "##"));
    assert!(!starts_with("##ä", "####"));
    assert!(starts_with("####ä", "##"));
    assert!(!starts_with("##", "####ä"));
    assert!(starts_with("##ä##", "##ä"));

    assert!(starts_with("", ""));
    assert!(starts_with("ä", ""));
    assert!(starts_with("#ä", ""));
    assert!(starts_with("##ä", ""));
    assert!(starts_with("ä###", ""));
    assert!(starts_with("#ä##", ""));
    assert!(starts_with("##ä#", ""));
}

#[test]
fn contains_weird_cases() {
    assert!(contains("* \t", ' '));
    assert!(!contains("* \t", '?'));
    assert!(!contains("* \t", '\u{1F4A9}'));
}

#[test]
fn trim_ws() {
    assert_eq!(trim_start(" \t  a \t  ", |c: char| c.is_whitespace()),
                    "a \t  ");
    assert_eq!(trim_end(" \t  a \t  ", |c: char| c.is_whitespace()),
               " \t  a");
    assert_eq!(trim(" \t  a \t  ", |c: char| c.is_whitespace()),
                    "a");
    assert_eq!(trim_start(" \t   \t  ", |c: char| c.is_whitespace()),
                         "");
    assert_eq!(trim_end(" \t   \t  ", |c: char| c.is_whitespace()),
               "");
    assert_eq!(trim(" \t   \t  ", |c: char| c.is_whitespace()),
               "");
}

#[test]
fn different_str_pattern_forwarding_lifetimes() {
    use pattern_3::Needle;

    fn foo<'a, P>(p: P) where for<'b> &'b P: Needle<&'a str> {
        for _ in 0..3 {
            find("asdf", &p);
        }
    }

    foo::<&str>("x");
}

#[test]
fn test_trim_strings() {
    assert_eq!(trim_start("foo:bar:foo:baz", "foo:"), "bar:foo:baz");
    assert_eq!(trim_start("fo:bar:foo:baz", "foo:"), "fo:bar:foo:baz");
    assert_eq!(trim_start("bar:baz", "foo:"), "bar:baz");
    assert_eq!(trim_start("foo:foo:", "foo:"), "");
    assert_eq!(trim_start("", "foo:"), "");
    assert_eq!(trim_start("adadad", ""), "adadad");

    assert_eq!(trim_end("foo:bar:foo:baz", ":baz"), "foo:bar:foo");
    assert_eq!(trim_end("foo:bar:foo:baz", ":bar"), "foo:bar:foo:baz");
    assert_eq!(trim_end(":baz:baz", ":baz"), "");
    assert_eq!(trim_end("", ":baz"), "");
    assert_eq!(trim_end("adadad", ""), "adadad");

    assert_eq!(trim_start("aaaaaaaa", "aaa"), "aa");
    assert_eq!(trim_start("ababababa", "ab"), "a");
    assert_eq!(trim_end("aaaaaaaa", "aaa"), "aa");
    assert_eq!(trim_end("ababababa", "ba"), "a");
}

// fn str_replacen<'a>(src: &'a str, from: impl pattern_3::Needle<&'a str>, to: &'a str, n: usize) -> String {
fn str_replacen<'a, P>(src: &'a str, from: P, to: &'a str, n: usize) -> String
where
    P: Needle<&'a str>,
    P::Searcher: Searcher<str>, // FIXME: RFC 2089
    P::Consumer: Consumer<str>,
{
    let mut res = String::with_capacity(src.len());
    replacen_with(src, from, |_| to, n, |h| res.push_str(h));
    res
}

fn str_replace<'a, P>(src: &'a str, from: P, to: &'a str) -> String
where
    P: Needle<&'a str>,
    P::Searcher: Searcher<str>, // FIXME: RFC 2089
    P::Consumer: Consumer<str>,
{
    let mut res = String::with_capacity(src.len());
    replace_with(src, from, |_| to, |h| res.push_str(h));
    res
}

#[test]
fn test_replacen() {
    assert_eq!(str_replacen("", 'a', "b", 5), "");
    assert_eq!(str_replacen("acaaa", "a", "b", 3), "bcbba");
    assert_eq!(str_replacen("aaaa", "a", "b", 0), "aaaa");

    let test = "test";
    assert_eq!(str_replacen(" test test ", test, "toast", 3), " toast toast ");
    assert_eq!(str_replacen(" test test ", test, "toast", 0), " test test ");
    assert_eq!(str_replacen(" test test ", test, "", 5), "   ");

    assert_eq!(str_replacen("qwer123zxc789", char::is_numeric, "", 3), "qwerzxc789");
}

#[test]
fn test_replace() {
    let a = "a";
    assert_eq!(str_replace("", a, "b"), "");
    assert_eq!(str_replace("a", a, "b"), "b");
    assert_eq!(str_replace("ab", a, "b"), "bb");
    let test = "test";
    assert_eq!(str_replace(" test test ", test, "toast"), " toast toast ");
    assert_eq!(str_replace(" test test ", test, ""), "   ");
}

#[test]
fn test_replace_2a() {
    let data = "ประเทศไทย中华";
    let repl = "دولة الكويت";

    let a = "ประเ";
    let a2 = "دولة الكويتทศไทย中华";
    assert_eq!(str_replace(data, a, repl), a2);
}

#[test]
fn test_replace_2b() {
    let data = "ประเทศไทย中华";
    let repl = "دولة الكويت";

    let b = "ะเ";
    let b2 = "ปรدولة الكويتทศไทย中华";
    assert_eq!(str_replace(data, b, repl), b2);
}

#[test]
fn test_replace_2c() {
    let data = "ประเทศไทย中华";
    let repl = "دولة الكويت";

    let c = "中华";
    let c2 = "ประเทศไทยدولة الكويت";
    assert_eq!(str_replace(data, c, repl), c2);
}

#[test]
fn test_replace_2d() {
    let data = "ประเทศไทย中华";
    let repl = "دولة الكويت";

    let d = "ไท华";
    assert_eq!(str_replace(data, d, repl), data);
}

#[test]
fn test_replace_pattern() {
    let data = "abcdαβγδabcdαβγδ";
    assert_eq!(str_replace(data, "dαβ", "😺😺😺"), "abc😺😺😺γδabc😺😺😺γδ");
    assert_eq!(str_replace(data, 'γ', "😺😺😺"), "abcdαβ😺😺😺δabcdαβ😺😺😺δ");
    assert_eq!(str_replace(data, &['a', 'γ'] as &[_], "😺😺😺"), "😺😺😺bcdαβ😺😺😺δ😺😺😺bcdαβ😺😺😺δ");
    assert_eq!(str_replace(data, |c| c == 'γ', "😺😺😺"), "abcdαβ😺😺😺δabcdαβ😺😺😺δ");
}


#[test]
fn test_mut_str() {
    use std::ops::Range;

    let mut s = String::from("a1b2c3d4e");
    {
        let res: &mut str = trim(&mut *s, |c: char| c.is_ascii_alphabetic());
        assert_eq!(res, "1b2c3d4");
    }
    {
        let res: Vec<&mut str> = split(&mut *s, |c: char| c.is_ascii_digit()).collect();
        assert_eq!(res, vec!["a", "b", "c", "d", "e"]);
    }
    {
        let res: Vec<(Range<usize>, &mut str)> = match_ranges(&mut *s, |c: char| c.is_ascii_digit()).collect();
        let res = res.into_iter().map(|(r, ss)| (r, &*ss)).collect::<Vec<_>>();
        assert_eq!(res, vec![
            (1..2, "1"),
            (3..4, "2"),
            (5..6, "3"),
            (7..8, "4"),
        ]);
    }
    {
        let res: Vec<(Range<usize>, &mut str)> = rmatch_ranges(&mut *s, |c: char| c.is_ascii_digit()).collect();
        let res = res.into_iter().map(|(r, ss)| (r, &*ss)).collect::<Vec<_>>();
        assert_eq!(res, vec![
            (7..8, "4"),
            (5..6, "3"),
            (3..4, "2"),
            (1..2, "1"),
        ]);
    }
}

