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
