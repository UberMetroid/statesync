//! Property-based tests for URL / origin / SSRF protocol helpers (RFC-shaped inputs).

use proptest::prelude::*;
use statesync::config::{
    extract_host, name_from_url, normalize_server_url, redacted_url, valid_server_url,
    validate_upstream_url,
};

/// Host labels that are safe for LAN-style origins (no metadata, no spaces).
fn arb_safe_host() -> impl Strategy<Value = String> {
    prop::collection::vec("[a-z][a-z0-9-]{0,8}", 1..3usize).prop_map(|parts| parts.join("."))
}

fn arb_port() -> impl Strategy<Value = u16> {
    1u16..65535
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// normalize is idempotent for http(s) origins it produces.
    #[test]
    fn normalize_idempotent_http_url(
        host in arb_safe_host(),
        port in arb_port(),
        path in prop::option::of("[a-z0-9/]{0,24}"),
    ) {
        let raw = format!(
            "http://{}:{}/{}",
            host,
            port,
            path.unwrap_or_default().trim_start_matches('/')
        );
        let once = normalize_server_url(&raw);
        let twice = normalize_server_url(&once);
        prop_assert_eq!(&once, &twice);
        prop_assert!(valid_server_url(&twice));
        if let Some(rest) = twice.split_once("://").map(|(_, r)| r) {
            prop_assert!(!rest.contains('/'), "path leaked: {}", twice);
        }
    }

    /// Bare host:port always becomes http://host:port
    #[test]
    fn bare_host_port_becomes_http(host in arb_safe_host(), port in arb_port()) {
        let raw = format!("{}:{}", host, port);
        let n = normalize_server_url(&raw);
        let expected = format!("http://{}:{}", host, port);
        prop_assert_eq!(n, expected);
    }

    /// Redaction never expands the authority; strips path when present.
    #[test]
    fn redacted_url_never_leaks_path_body(
        host in arb_safe_host(),
        port in arb_port(),
        seg in "[a-z]{1,12}",
    ) {
        let with_path = format!("https://{}:{}/{}", host, port, seg);
        let r = redacted_url(&with_path);
        let needle = format!("/{}", seg);
        prop_assert!(!r.contains(&needle));
    }

    /// name_from_url keeps host:port for distinct ports.
    #[test]
    fn name_from_url_includes_port(host in arb_safe_host(), port in arb_port()) {
        let u = format!("http://{}:{}/web/index.html", host, port);
        let name = name_from_url(&u);
        let port_s = port.to_string();
        prop_assert!(name.contains(&host));
        prop_assert!(name.contains(&port_s));
    }

    /// Metadata IPv4 integer forms are always blocked when used as host.
    #[test]
    fn metadata_integer_ip_always_blocked(low in 0u16..256u16) {
        let n = (169u32 << 24) | (254u32 << 16) | ((low as u32) << 8) | 1u32;
        let url = format!("http://{}/", n);
        prop_assert!(validate_upstream_url(&url).is_err());
    }

    /// extract_host of normalized http URL is non-empty when URL is valid.
    #[test]
    fn extract_host_nonempty_for_valid(host in arb_safe_host(), port in arb_port()) {
        let u = normalize_server_url(&format!("http://{}:{}", host, port));
        let h = extract_host(&u);
        prop_assert!(h.as_ref().is_some_and(|s| !s.is_empty()));
    }

    /// Userinfo smuggling: host after @ is what validate uses (metadata still blocked).
    #[test]
    fn userinfo_cannot_hide_metadata_host(user in "[a-z]{1,8}") {
        let url = format!("http://{}@169.254.169.254/", user);
        prop_assert!(validate_upstream_url(&url).is_err());
        let host = extract_host(&url);
        prop_assert_eq!(host.as_deref(), Some("169.254.169.254"));
    }

    /// Empty / garbage inputs: normalize does not panic; invalid URLs rejected.
    #[test]
    fn normalize_garbage_no_panic(s in "\\PC{0,200}") {
        let n = normalize_server_url(&s);
        let lower = n.to_lowercase();
        if !(lower.starts_with("http://") || lower.starts_with("https://")) {
            prop_assert!(!valid_server_url(&n) || n.is_empty());
        }
        let _ = validate_upstream_url(&s);
        let _ = redacted_url(&s);
        let _ = name_from_url(&s);
    }
}
