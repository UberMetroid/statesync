//! Unit tests for private/crate helpers (origin_only, name_set surfaces).

#[cfg(test)]
mod origin_only_tests {
    use crate::config::helpers::origin_only;

    #[test]
    fn strips_path_after_host() {
        assert_eq!(
            origin_only("http://10.0.0.1:8096/web/index.html"),
            "http://10.0.0.1:8096"
        );
    }

    #[test]
    fn keeps_ipv6_authority() {
        assert_eq!(origin_only("http://[::1]:8096/path"), "http://[::1]:8096");
    }

    #[test]
    fn empty_authority_returns_empty() {
        assert_eq!(origin_only("http://"), "");
    }

    #[test]
    fn no_scheme_passthrough_trimmed() {
        assert_eq!(origin_only("not-a-url/"), "not-a-url");
    }
}

#[cfg(test)]
mod ipv4_and_metadata_tests {
    use crate::config::url_safety::{ipv4_octets, is_cloud_metadata_host};

    #[test]
    fn dotted_decimal_parse() {
        assert_eq!(ipv4_octets("10.0.0.1"), Some([10, 0, 0, 1]));
        assert_eq!(ipv4_octets("256.0.0.1"), None);
        assert_eq!(ipv4_octets("1.2.3"), None);
    }

    #[test]
    fn integer_and_hex_forms() {
        // 169.254.169.254
        assert_eq!(ipv4_octets("2852039166"), Some([169, 254, 169, 254]));
        assert_eq!(ipv4_octets("0xA9FEA9FE"), Some([169, 254, 169, 254]));
    }

    #[test]
    fn metadata_host_detection() {
        assert!(is_cloud_metadata_host("169.254.169.254"));
        assert!(is_cloud_metadata_host("169.254.1.1"));
        assert!(is_cloud_metadata_host("metadata.google.internal"));
        assert!(is_cloud_metadata_host("fe80::1"));
        assert!(!is_cloud_metadata_host("10.0.0.5"));
        assert!(!is_cloud_metadata_host("localhost"));
    }
}
