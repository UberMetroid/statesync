/// Missing documentation.
pub fn redacted_url(url: &str) -> String {
    let trimmed = url.trim_end_matches('/');
    if let Some(idx) = trimmed.find("://") {
        let rest = &trimmed[idx + 3..];
        if let Some(slash) = rest.find('/') {
            return format!("{}://{}/...", &trimmed[..idx], &rest[..slash]);
        }
        return format!("{}://{}", &trimmed[..idx], rest);
    }
    trimmed.to_string()
}

/// Normalize any user-pasted media-server address to `scheme://host[:port]`.
///
/// Accepts whatever people copy from a browser (web UI paths, `#!/…` fragments,
/// query strings, trailing slashes) or type bare (`host:8096`). Always returns
/// only the API base origin — never keeps `/web/index.html` or similar.
pub fn normalize_server_url(url: &str) -> String {
    let t = url.trim();
    if t.is_empty() {
        return String::new();
    }

    // Drop fragment first (#/web/... or #!/apikeys).
    let t = t.split('#').next().unwrap_or(t).trim();
    // Drop query string.
    let t = t.split('?').next().unwrap_or(t).trim();
    if t.is_empty() {
        return String::new();
    }

    let lower = t.to_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") {
        return origin_only(t);
    }
    // Already has some other scheme (ftp, ws, …) — do not rewrite.
    if let Some(idx) = t.find("://") {
        if idx > 0 && t[..idx].chars().all(|c| c.is_ascii_alphabetic()) {
            return t.trim_end_matches('/').to_string();
        }
    }
    // Bare host / host:port / IP — default to http for LAN media servers.
    origin_only(&format!("http://{}", t))
}

/// Keep only `scheme://host[:port]` (strip path after the authority).
fn origin_only(url: &str) -> String {
    let Some((scheme, rest)) = url.split_once("://") else {
        return url.trim_end_matches('/').to_string();
    };
    // rest may be host:port/path or [ipv6]:port/path
    let authority = if rest.starts_with('[') {
        // IPv6: [fe80::1]:8096/path
        match rest.find(']') {
            Some(end) => {
                let after = &rest[end + 1..];
                if let Some(slash) = after.find('/') {
                    format!("{}{}", &rest[..=end], &after[..slash])
                } else {
                    rest.to_string()
                }
            }
            None => rest.split('/').next().unwrap_or(rest).to_string(),
        }
    } else {
        rest.split('/').next().unwrap_or(rest).to_string()
    };
    let authority = authority.trim().trim_end_matches('/');
    if authority.is_empty() {
        return String::new();
    }
    format!("{}://{}", scheme, authority)
}

/// Derive a display name from a server URL (hostname preferred).
pub fn name_from_url(url: &str) -> String {
    let u = normalize_server_url(url);
    let without_scheme = u
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(u.as_str());
    let host_port = without_scheme
        .split('/')
        .next()
        .unwrap_or(without_scheme)
        .split('?')
        .next()
        .unwrap_or(without_scheme);
    let host = if host_port.starts_with('[') {
        host_port
            .trim_start_matches('[')
            .split(']')
            .next()
            .unwrap_or(host_port)
            .to_string()
    } else {
        host_port
            .rsplit_once(':')
            .map(|(h, port)| {
                // Keep host only when the suffix looks like a port.
                if port.chars().all(|c| c.is_ascii_digit()) {
                    h.to_string()
                } else {
                    host_port.to_string()
                }
            })
            .unwrap_or_else(|| host_port.to_string())
    };
    let host = host.trim();
    if host.is_empty() {
        "server".to_string()
    } else {
        host.to_string()
    }
}
