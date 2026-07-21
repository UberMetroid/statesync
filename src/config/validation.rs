use anyhow::{Result, anyhow};
use super::{Config, ServerConfig, MAX_NAME_LEN, MAX_URL_LEN, MAX_KEY_LEN, MAX_MAPPING_GROUPS, MAX_GROUP_MEMBERS, MAX_MEMBER_LEN};

pub(super) fn validate_server(s: &ServerConfig) -> Result<()> {
    if s.name.is_empty() || s.name.len() > MAX_NAME_LEN {
        return Err(anyhow!(
            "server name must be 1..={} chars (got {})",
            MAX_NAME_LEN,
            s.name.len()
        ));
    }
    if s.url.len() > MAX_URL_LEN || !(s.url.starts_with("http://") || s.url.starts_with("https://"))
    {
        return Err(anyhow!(
            "server '{}': url must start with http:// or https:// and be <={} chars",
            s.name,
            MAX_URL_LEN
        ));
    }
    if s.url.starts_with("http://") && !s.allow_insecure_http {
        return Err(anyhow!(
            "server '{}': http:// url rejected (set allow_insecure_http: true to override)",
            s.name
        ));
    }
    if s.api_key.len() > MAX_KEY_LEN {
        return Err(anyhow!(
            "server '{}': api_key too long ({} > {})",
            s.name,
            s.api_key.len(),
            MAX_KEY_LEN
        ));
    }
    match s.sync_direction.as_str() {
        "both" | "send" | "receive" => {}
        _ => {
            return Err(anyhow!(
                "server '{}': sync_direction must be one of both|send|receive",
                s.name
            ));
        }
    }
    Ok(())
}

pub fn validate_config(cfg: &Config) -> Result<()> {
    if cfg.servers.len() > 20 {
        return Err(anyhow!(
            "too many servers configured ({} > 20)",
            cfg.servers.len()
        ));
    }
    let mut names = std::collections::HashSet::new();
    for s in &cfg.servers {
        validate_server(s)?;
        if !names.insert(s.name.to_lowercase()) {
            return Err(anyhow!("duplicate server name '{}' in config", s.name));
        }
    }
    if cfg.user_mappings.len() > MAX_MAPPING_GROUPS {
        return Err(anyhow!(
            "too many user_mapping groups ({} > {})",
            cfg.user_mappings.len(),
            MAX_MAPPING_GROUPS
        ));
    }
    for group in &cfg.user_mappings {
        if group.len() > MAX_GROUP_MEMBERS {
            return Err(anyhow!(
                "user_mapping group has too many members ({} > {})",
                group.len(),
                MAX_GROUP_MEMBERS
            ));
        }
        for name in group {
            if name.is_empty() || name.len() > MAX_MEMBER_LEN {
                return Err(anyhow!(
                    "user_mapping member name must be 1..={} chars",
                    MAX_MEMBER_LEN
                ));
            }
        }
    }
    Ok(())
}

pub fn is_loopback_bind(addr: &str) -> bool {
    let host = if let Some(rest) = addr.strip_prefix('[') {
        match rest.find(']') {
            Some(end) => &rest[..end],
            None => return false,
        }
    } else if addr.matches(':').count() > 1 {
        return addr == "::1";
    } else {
        match addr.rsplit_once(':') {
            Some((h, _)) => h,
            None => addr,
        }
    };
    matches!(host, "127.0.0.1" | "::1" | "localhost")
}
