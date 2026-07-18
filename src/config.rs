use std::env;
use anyhow::{Result, Context, anyhow};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub url: String,
    pub api_key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub emby: ServerConfig,
    pub jellyfin: ServerConfig,
    #[serde(default = "default_threshold_seconds")]
    pub sync_threshold_seconds: u64,
}

fn default_threshold_seconds() -> u64 {
    5
}

impl Config {
    pub fn load() -> Result<Self> {
        // 1. Try reading from environment variables first (allows easy container configuration)
        let emby_url = env::var("STATESYNC_EMBY_URL").ok();
        let emby_key = env::var("STATESYNC_EMBY_API_KEY").ok();
        let jf_url = env::var("STATESYNC_JELLYFIN_URL").ok();
        let jf_key = env::var("STATESYNC_JELLYFIN_API_KEY").ok();
        let threshold = env::var("STATESYNC_SYNC_THRESHOLD_SECONDS")
            .ok()
            .and_then(|val| val.parse::<u64>().ok());

        if let (Some(e_url), Some(e_key), Some(j_url), Some(j_key)) = (emby_url, emby_key, jf_url, jf_key) {
            return Ok(Self {
                emby: ServerConfig { url: e_url, api_key: e_key },
                jellyfin: ServerConfig { url: j_url, api_key: j_key },
                sync_threshold_seconds: threshold.unwrap_or(5),
            });
        }

        // 2. Fall back to config.json
        // Look in /etc/statesync/config.json, then /app/config.json, then ./config.json
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
            "Configuration not found. Please provide environment variables (STATESYNC_EMBY_URL, STATESYNC_EMBY_API_KEY, STATESYNC_JELLYFIN_URL, STATESYNC_JELLYFIN_API_KEY) or a config.json file in /etc/statesync/ or current directory."
        ))
    }
}
