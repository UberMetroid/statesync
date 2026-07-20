use serde::{Deserialize, Serialize};

pub mod helpers;
pub mod loader;
pub mod validation;
#[cfg(test)]
pub mod tests;

pub use helpers::redacted_url;
pub use loader::{load_or_create_default, write_default_config_to_disk, get_config_path, default_config};
pub use validation::{is_loopback_bind, validate_config};

pub const MAX_NAME_LEN: usize = 64;
pub const MAX_URL_LEN: usize = 512;
pub const MAX_KEY_LEN: usize = 256;
pub const MAX_MAPPING_GROUPS: usize = 128;
pub const MAX_GROUP_MEMBERS: usize = 32;
pub const MAX_MEMBER_LEN: usize = 64;
pub const MAX_CONFIG_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub name: String,
    pub url: String,
    pub api_key: String,
    pub is_emby: bool,
    #[serde(default = "default_sync_direction")]
    pub sync_direction: String, // "both", "send", "receive"
    #[serde(default = "default_allow_insecure_http")]
    pub allow_insecure_http: bool,
}

fn default_allow_insecure_http() -> bool {
    true
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
    #[serde(default)]
    pub last_full_sync: Option<crate::sync_force::ForceSyncStatus>,
}

fn default_threshold_seconds() -> u64 {
    5
}
