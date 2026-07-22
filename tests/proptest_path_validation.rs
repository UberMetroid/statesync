//! Property tests for API path / name validators (injection boundaries).

use proptest::prelude::*;
use statesync::web_api::validation::{valid_item_id, valid_server_name};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// Any item id containing path separators is invalid.
    #[test]
    fn item_id_rejects_path_separators(
        prefix in "[A-Za-z0-9_-]{0,20}",
        suffix in "[A-Za-z0-9_-]{0,20}",
        sep in prop::sample::select(vec!['/', '\\', ' ', '.', '?']),
    ) {
        let id = format!("{}{}{}", prefix, sep, suffix);
        prop_assume!(!id.is_empty());
        prop_assert!(!valid_item_id(&id));
    }

    /// Valid item ids: nonempty, <=64, only [A-Za-z0-9_-]
    #[test]
    fn item_id_accepts_safe_charset(s in "[A-Za-z0-9_-]{1,64}") {
        prop_assert!(valid_item_id(&s));
    }

    /// Empty and overlong item ids are rejected.
    #[test]
    fn item_id_boundary_empty_and_long(extra in 0usize..8) {
        prop_assert!(!valid_item_id(""));
        let long = "a".repeat(65 + extra);
        prop_assert!(!valid_item_id(&long));
        prop_assert!(valid_item_id(&"a".repeat(64)));
        prop_assert!(valid_item_id("a"));
    }

    /// Server names reject path traversal and slashes.
    #[test]
    fn server_name_rejects_traversal_and_slashes(
        a in "[A-Za-z0-9]{1,10}",
        b in "[A-Za-z0-9]{1,10}",
    ) {
        let slash = format!("{}/{}", a, b);
        let back = format!("{}\\{}", a, b);
        let dots = format!("{}..{}", a, b);
        prop_assert!(!valid_server_name(&slash));
        prop_assert!(!valid_server_name(&back));
        prop_assert!(!valid_server_name(&dots));
        prop_assert!(!valid_server_name(""));
        prop_assert!(!valid_server_name(&"x".repeat(65)));
    }

    /// Server names accept alnum + limited punctuation (incl. spaces for auto names).
    #[test]
    fn server_name_accepts_common_forms(s in "[A-Za-z0-9][A-Za-z0-9 ._@:-]{0,30}") {
        prop_assume!(!s.contains("..") && !s.contains('/') && !s.contains('\\'));
        prop_assume!(s.len() <= 64 && !s.is_empty());
        prop_assert!(valid_server_name(&s));
    }
}
