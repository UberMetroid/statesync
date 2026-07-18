use std::env;
use anyhow::{Result, Context, anyhow};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub name: String,
    pub url: String,
    pub api_key: String,
    pub is_emby: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub servers: Vec<ServerConfig>,
    #[serde(default = "default_threshold_seconds")]
    pub sync_threshold_seconds: u64,
}

fn default_threshold_seconds() -> u64 {
    5
}

impl Config {
    pub fn load() -> Result<Self> {
        // 1. Try environment variable JSON string first (allows multi-server container configuration)
        if let Ok(servers_json) = env::var("STATESYNC_SERVERS_JSON") {
            if let Ok(servers) = serde_json::from_str::<Vec<ServerConfig>>(&servers_json) {
                let threshold = env::var("STATESYNC_SYNC_THRESHOLD_SECONDS")
                    .ok()
                    .and_then(|val| val.parse::<u64>().ok())
                    .unwrap_or(5);
                return Ok(Self { servers, sync_threshold_seconds: threshold });
            }
        }

        // 2. Support backward-compatible standard two-server environment variables
        let emby_url = env::var("STATESYNC_EMBY_URL").ok();
        let emby_key = env::var("STATESYNC_EMBY_API_KEY").ok();
        let jf_url = env::var("STATESYNC_JELLYFIN_URL").ok();
        let jf_key = env::var("STATESYNC_JELLYFIN_API_KEY").ok();
        let threshold = env::var("STATESYNC_SYNC_THRESHOLD_SECONDS")
            .ok()
            .and_then(|val| val.parse::<u64>().ok());

        if let (Some(e_url), Some(e_key), Some(j_url), Some(j_key)) = (emby_url, emby_key, jf_url, jf_key) {
            return Ok(Self {
                servers: vec![
                    ServerConfig { name: "Emby".to_string(), url: e_url, api_key: e_key, is_emby: true },
                    ServerConfig { name: "Jellyfin".to_string(), url: j_url, api_key: j_key, is_emby: false },
                ],
                sync_threshold_seconds: threshold.unwrap_or(5),
            });
        }

        // 3. Fall back to config.json
        let paths = ["/etc/statesync/config.json", "/app/config.json", "config.json"];
        for path in &paths {
            if std::path::Path::new(path).exists() {
                let data = std::fs::read_to_string(path)
                    .with_context(|| format!("Failed to read configuration file: {}", path))?;
                let config: Config = serde_json::from_str(&data)
                    .context("Failed to parse configuration file")?;
                return Ok(config);
            }
        }

        Err(anyhow!(
            "Configuration not found. Please provide STATESYNC_SERVERS_JSON env variable, standard server env variables, or a config.json file."
        ))
    }
}
