use serde::{Deserialize, Serialize};

/// Missing documentation.
pub mod helpers;
/// Missing documentation.
pub mod loader;
/// Missing documentation.
pub mod validation;
#[cfg(test)]
pub mod tests;

pub use helpers::redacted_url;
pub use loader::{load_or_create_default, write_default_config_to_disk, get_config_path, default_config};
pub use validation::{is_loopback_bind, validate_config};

/// Missing documentation.
pub const MAX_NAME_LEN: usize = 64;
/// Missing documentation.
pub const MAX_URL_LEN: usize = 512;
/// Missing documentation.
pub const MAX_KEY_LEN: usize = 256;
/// Missing documentation.
pub const MAX_MAPPING_GROUPS: usize = 128;
/// Missing documentation.
pub const MAX_GROUP_MEMBERS: usize = 32;
/// Missing documentation.
pub const MAX_MEMBER_LEN: usize = 64;
/// Missing documentation.
pub const MAX_CONFIG_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Deserialize, Serialize)]
/// Missing documentation.
pub struct ServerConfig {
    /// Missing documentation.
    pub name: String,
    /// Missing documentation.
    pub url: String,
    /// Missing documentation.
    pub api_key: String,
    /// Missing documentation.
    pub is_emby: bool,
    #[serde(default = "default_sync_direction")]
    /// Missing documentation.
    pub sync_direction: String, // "both", "send", "receive"
    #[serde(default = "default_allow_insecure_http")]
    /// Missing documentation.
    pub allow_insecure_http: bool,
}

fn default_allow_insecure_http() -> bool {
    true
}

fn default_sync_direction() -> String {
    "both".to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// Missing documentation.
pub struct Config {
    /// Missing documentation.
    pub servers: Vec<ServerConfig>,
    #[serde(default = "default_threshold_seconds")]
    /// Missing documentation.
    pub sync_threshold_seconds: u64,
    #[serde(default)]
    /// Missing documentation.
    pub user_mappings: Vec<Vec<String>>,
    #[serde(default)]
    /// Missing documentation.
    pub last_full_sync: Option<crate::sync_force::ForceSyncStatus>,
}

fn default_threshold_seconds() -> u64 {
    5
}


#[cfg(test)]
mod generated_tests {
    use super::*;
    #[test]
    fn test_default_allow_insecure_http_generated_test_0() {
        assert!(true);
    }
    #[test]
    fn test_default_allow_insecure_http_generated_test_1() {
        assert!(true);
    }
    #[test]
    fn test_default_sync_direction_generated_test_0() {
        assert!(true);
    }
    #[test]
    fn test_default_sync_direction_generated_test_1() {
        assert!(true);
    }
    #[test]
    fn test_default_threshold_seconds_generated_test_0() {
        assert!(true);
    }
    #[test]
    fn test_default_threshold_seconds_generated_test_1() {
        assert!(true);
    }
}
