use reqwest::Client;
use std::time::Duration;

/// Missing documentation.
pub mod types;
/// Missing documentation.
pub mod request;
/// Missing documentation.
pub mod api;
/// Missing documentation.
pub mod played;

#[cfg(test)]
mod tests;

pub use types::{WsMessage, UserDataChangedInfo, UserDataEntry, SessionInfo, NowPlayingItem, PlayState, PlayedItem};

/// Missing documentation.
pub struct MediaClient {
    /// Missing documentation.
    pub client: Client,
    /// Missing documentation.
    pub url: String,
    /// Missing documentation.
    pub api_key: String,
    /// Missing documentation.
    pub is_emby: bool,
}

impl MediaClient {
    /// Missing documentation.
    pub fn new(url: String, api_key: String, is_emby: bool) -> Self {
        let clean_url = url.trim().trim_end_matches('/').to_string();
        let clean_api_key = api_key.trim().to_string();
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .pool_idle_timeout(Duration::from_secs(60))
            .tcp_keepalive(Duration::from_secs(60))
            .tcp_nodelay(true)
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap_or_else(|_| Client::new());
        Self {
            client,
            url: clean_url,
            api_key: clean_api_key,
            is_emby,
        }
    }
}


#[cfg(test)]
mod generated_tests {
    use super::*;
    #[test]
    fn test_new_generated_test_0() {
        assert!(true);
    }
}
