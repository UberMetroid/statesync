use std::env;
use anyhow::{Result, Context, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub name: String,
    pub url: String,
    pub api_key: String,
    pub is_emby: bool,
    #[serde(default = "default_sync_direction")]
    pub sync_direction: String, // "both", "send", "receive"
}

fn default_sync_direction() -> String {
    "both".to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub servers: Vec<ServerConfig>,
    #[serde(default = "default_threshold_seconds")]
    pub sync_threshold_seconds: u64,
    #[serde(default)]
    pub user_mappings: Vec<Vec<String>>,
}

fn default_threshold_seconds() -> u64 {
    5
}

impl Config {
    pub fn load() -> Result<Self> {
        let mut servers = Vec::new();

        // 1. Check for flat environment variables: STATESYNC_SERVER_0_URL, etc. (for Unraid form inputs)
        for i in 0..20 {
            let url_var = format!("STATESYNC_SERVER_{}_URL", i);
            if let Ok(url) = env::var(&url_var) {
                if url.trim().is_empty() {
                    continue;
                }
                let name_var = format!("STATESYNC_SERVER_{}_NAME", i);
                let key_var = format!("STATESYNC_SERVER_{}_API_KEY", i);
                let type_var = format!("STATESYNC_SERVER_{}_TYPE", i);
                let dir_var = format!("STATESYNC_SERVER_{}_DIRECTION", i);

                let name = env::var(&name_var).unwrap_or_else(|_| format!("Server {}", i));
                let api_key = env::var(&key_var)
                    .with_context(|| format!("Missing API key environment variable: {}", key_var))?;
                
                let is_emby = env::var(&type_var)
                    .map(|val| val.to_lowercase() == "emby")
                    .unwrap_or(false);
                
                let sync_direction = env::var(&dir_var).unwrap_or_else(|_| "both".to_string());

                servers.push(ServerConfig {
                    name,
                    url,
                    api_key,
                    is_emby,
                    sync_direction,
                });
            }
        }

        // 2. Fallback to standard two-server environment variables
        if servers.is_empty() {
            let emby_url = env::var("STATESYNC_EMBY_URL").ok();
            let emby_key = env::var("STATESYNC_EMBY_API_KEY").ok();
            let jf_url = env::var("STATESYNC_JELLYFIN_URL").ok();
            let jf_key = env::var("STATESYNC_JELLYFIN_API_KEY").ok();

            if let (Some(e_url), Some(e_key), Some(j_url), Some(j_key)) = (emby_url, emby_key, jf_url, jf_key) {
                servers.push(ServerConfig { name: "Emby".to_string(), url: e_url, api_key: e_key, is_emby: true, sync_direction: "both".to_string() });
                servers.push(ServerConfig { name: "Jellyfin".to_string(), url: j_url, api_key: j_key, is_emby: false, sync_direction: "both".to_string() });
            }
        }

        // 3. Fallback to config.json
        if servers.is_empty() {
            let paths = [get_config_path(), "/etc/statesync/config.json", "/app/config.json", "config.json"];
            for path in &paths {
                if std::path::Path::new(path).exists() {
                    let data = std::fs::read_to_string(path)
                        .with_context(|| format!("Failed to read configuration file: {}", path))?;
                    let config: Config = serde_json::from_str(&data)
                        .context("Failed to parse configuration file")?;
                    return Ok(config);
                }
            }
        }

        if servers.is_empty() {
            return Err(anyhow!(
                "Configuration not found. Please provide environment variables (e.g. STATESYNC_SERVER_0_URL and STATESYNC_SERVER_0_API_KEY) or a config.json file."
            ));
        }

        let threshold = env::var("STATESYNC_SYNC_THRESHOLD_SECONDS")
            .ok()
            .and_then(|val| val.parse::<u64>().ok())
            .unwrap_or(5);

        Ok(Self {
            servers,
            sync_threshold_seconds: threshold,
            user_mappings: Vec::new(),
        })
    }
}

pub fn get_config_path() -> &'static str {
    if std::path::Path::new("/config").exists() {
        "/config/config.json"
    } else if std::path::Path::new("/etc/statesync").exists() {
        "/etc/statesync/config.json"
    } else if std::path::Path::new("/app").exists() {
        "/app/config.json"
    } else {
        "config.json"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_deserialization_with_defaults() {
        let json = r#"{
            "servers": [
                {
                    "name": "green",
                    "url": "http://localhost:8096",
                    "api_key": "secret",
                    "is_emby": true
                }
            ]
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.servers.len(), 1);
        assert_eq!(config.servers[0].name, "green");
        assert_eq!(config.sync_threshold_seconds, 5);
        assert_eq!(config.servers[0].sync_direction, "both");
        assert!(config.user_mappings.is_empty());
    }

    #[test]
    fn test_config_with_custom_user_mappings() {
        let json = r#"{
            "servers": [],
            "sync_threshold_seconds": 10,
            "user_mappings": [
                ["john doe", "john"],
                ["jane", "jane_doe"]
            ]
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.sync_threshold_seconds, 10);
        assert_eq!(config.user_mappings.len(), 2);
        assert_eq!(config.user_mappings[0], vec!["john doe", "john"]);
    }
}
