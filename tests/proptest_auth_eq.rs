//! Property tests for constant-time bearer compare (auth helper).

use proptest::prelude::*;
use statesync::web::constant_time_eq;

#[test]
fn positive_equal_strings() {
    assert!(constant_time_eq("token", "token"));
    assert!(constant_time_eq("", ""));
}

#[test]
fn negative_unequal_content_or_length() {
    assert!(!constant_time_eq("token", "Token"));
    assert!(!constant_time_eq("ab", "abc"));
    assert!(!constant_time_eq("abc", "ab"));
}

#[test]
fn boundary_single_char() {
    assert!(constant_time_eq("a", "a"));
    assert!(!constant_time_eq("a", "b"));
    assert!(!constant_time_eq("a", ""));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn reflexive(s in "\\PC{0,64}") {
        prop_assert!(constant_time_eq(&s, &s));
    }

    #[test]
    fn symmetric(a in "\\PC{0,32}", b in "\\PC{0,32}") {
        prop_assert_eq!(constant_time_eq(&a, &b), constant_time_eq(&b, &a));
    }

    #[test]
    fn different_length_never_equal(
        a in "\\PC{1,16}",
        b in "\\PC{1,16}",
    ) {
        prop_assume!(a.len() != b.len());
        prop_assert!(!constant_time_eq(&a, &b));
    }

    #[test]
    fn one_bit_flip_never_equal(s in "[a-zA-Z0-9]{8,32}") {
        let mut t = s.clone().into_bytes();
        t[0] ^= 0x01;
        let other = String::from_utf8_lossy(&t).into_owned();
        prop_assume!(other != s);
        prop_assert!(!constant_time_eq(&s, &other));
    }
}
