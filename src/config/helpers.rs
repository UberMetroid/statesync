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

/// Ensure a media-server URL has a scheme and no trailing slash.
/// Bare hosts like `192.168.1.50:8096` become `http://192.168.1.50:8096`.
/// Existing schemes (`http`, `https`, `ftp`, `ws`, …) are left alone.
pub fn normalize_server_url(url: &str) -> String {
    let t = url.trim().trim_end_matches('/');
    if t.is_empty() {
        return String::new();
    }
    let lower = t.to_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") {
        return t.to_string();
    }
    // Already has some other scheme (ftp, ws, …) — do not rewrite.
    if let Some(idx) = t.find("://") {
        if idx > 0 && t[..idx].chars().all(|c| c.is_ascii_alphabetic()) {
            return t.to_string();
        }
    }
    // Bare host / host:port / IP — default to http for LAN media servers.
    format!("http://{}", t)
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
