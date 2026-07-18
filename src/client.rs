use std::collections::HashMap;
use serde::Deserialize;
use reqwest::Client;
use anyhow::{Result, Context, anyhow};

#[derive(Debug, Clone, Deserialize)]
pub struct WsMessage {
    #[serde(alias = "messageType", alias = "MessageType")]
    pub message_type: String,
    #[serde(alias = "data", alias = "Data")]
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserDataChangedInfo {
    #[serde(alias = "userId", alias = "UserId")]
    pub user_id: String,
    #[serde(alias = "userDataList", alias = "UserDataList")]
    pub user_data_list: Vec<UserDataEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserDataEntry {
    #[serde(alias = "itemId", alias = "ItemId")]
    pub item_id: String,
    #[serde(alias = "played", alias = "Played")]
    pub played: bool,
    #[serde(alias = "playbackPositionTicks", alias = "PlaybackPositionTicks")]
    pub playback_position_ticks: Option<i64>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct SessionInfo {
    #[serde(alias = "id", alias = "Id")]
    pub id: String,
    #[serde(alias = "userName", alias = "UserName")]
    pub user_name: Option<String>,
    #[serde(alias = "nowPlayingItem", alias = "NowPlayingItem")]
    pub now_playing_item: Option<NowPlayingItem>,
    #[serde(alias = "playState", alias = "PlayState")]
    pub play_state: Option<PlayState>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct NowPlayingItem {
    #[serde(alias = "id", alias = "Id")]
    pub id: String,
    #[serde(alias = "name", alias = "Name")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct PlayState {
    #[serde(alias = "positionTicks", alias = "PositionTicks")]
    pub position_ticks: Option<i64>,
    #[serde(alias = "isPaused", alias = "IsPaused")]
    pub is_paused: Option<bool>,
}

pub struct MediaClient {
    pub client: Client,
    pub url: String,
    pub api_key: String,
    pub is_emby: bool,
}

impl MediaClient {
    pub fn new(url: String, api_key: String, is_emby: bool) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .connect_timeout(std::time::Duration::from_secs(15))
            .build()
            .unwrap_or_else(|_| Client::new());
        Self {
            client,
            url: url.trim_end_matches('/').to_string(),
            api_key,
            is_emby,
        }
    }

    pub fn url_path(&self, path: &str) -> String {
        let prefix = if self.is_emby { "/emby" } else { "" };
        format!("{}{}{}", self.url, prefix, path)
    }

    pub fn add_auth_headers(&self, mut builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if self.is_emby {
            builder = builder.header("X-Emby-Token", &self.api_key);
        } else {
            builder = builder.header("X-MediaBrowser-Token", &self.api_key);
        }
        builder
    }

    pub async fn get_users(&self) -> Result<HashMap<String, String>> {
        let url = self.url_path("/Users");
        let resp = self.add_auth_headers(self.client.get(&url))
            .send()
            .await
            .context("Failed to get users list")?;
        
        let data: serde_json::Value = resp.json()
            .await
            .context("Failed to parse users response")?;
        
        let mut map = HashMap::new();
        if let Some(arr) = data.as_array() {
            for u in arr {
                if let (Some(name), Some(id)) = (u.get("Name").and_then(|n| n.as_str()), u.get("Id").and_then(|id| id.as_str())) {
                    map.insert(name.to_lowercase(), id.to_string());
                }
            }
        }
        Ok(map)
    }

    pub async fn get_library_items(&self) -> Result<HashMap<String, (String, String)>> {
        let url = self.url_path("/Items?Recursive=true&Fields=ProviderIds&IncludeItemTypes=Movie,Episode");
        let resp = self.add_auth_headers(self.client.get(&url))
            .send()
            .await
            .context("Failed to get library items")?;
        
        let data: serde_json::Value = resp.json()
            .await
            .context("Failed to parse library response")?;
        
        let mut map = HashMap::new();
        if let Some(arr) = data.get("Items").and_then(|i| i.as_array()) {
            for item in arr {
                if let Some(id) = item.get("Id").and_then(|id| id.as_str()) {
                    let mut imdb = String::new();
                    let mut tmdb = String::new();
                    if let Some(providers) = item.get("ProviderIds") {
                        if let Some(val) = providers.get("Imdb").and_then(|v| v.as_str()) { imdb = val.to_string(); }
                        if let Some(val) = providers.get("Tmdb").and_then(|v| v.as_str()) { tmdb = val.to_string(); }
                    }
                    map.insert(id.to_string(), (imdb, tmdb));
                }
            }
        }
        Ok(map)
    }

    pub async fn get_item_providers(&self, user_id: &str, item_id: &str) -> Result<(String, String)> {
        let path = format!("/Users/{}/Items/{}", user_id, item_id);
        let url = self.url_path(&path);
        let resp = self.add_auth_headers(self.client.get(&url))
            .send()
            .await
            .context("Failed to get item details")?;
        
        let data: serde_json::Value = resp.json()
            .await
            .context("Failed to parse item response")?;
        
        let mut imdb = String::new();
        let mut tmdb = String::new();
        if let Some(providers) = data.get("ProviderIds") {
            if let Some(val) = providers.get("Imdb").and_then(|v| v.as_str()) { imdb = val.to_string(); }
            if let Some(val) = providers.get("Tmdb").and_then(|v| v.as_str()) { tmdb = val.to_string(); }
        }
        Ok((imdb, tmdb))
    }

    pub async fn find_item_by_provider(&self, user_id: &str, imdb_id: &str, tmdb_id: &str) -> Result<Option<(String, String, String)>> {
        let mut path = format!("/Users/{}/Items?Recursive=true&Fields=ProviderIds", user_id);
        if !imdb_id.is_empty() {
            path.push_str(&format!("&AnyProviderIdTypes=Imdb&ProviderIds={}", imdb_id));
        } else if !tmdb_id.is_empty() {
            path.push_str(&format!("&AnyProviderIdTypes=Tmdb&ProviderIds={}", tmdb_id));
        } else {
            return Ok(None);
        }

        let url = self.url_path(&path);
        let resp = self.add_auth_headers(self.client.get(&url))
            .send()
            .await
            .context("Failed to search item by provider ID")?;

        let data: serde_json::Value = resp.json()
            .await
            .context("Failed to parse search response")?;

        if let Some(items) = data.get("Items").and_then(|i| i.as_array()) {
            if let Some(item) = items.first() {
                if let Some(id) = item.get("Id").and_then(|id| id.as_str()) {
                    let mut imdb = String::new();
                    let mut tmdb = String::new();
                    if let Some(providers) = item.get("ProviderIds") {
                        if let Some(val) = providers.get("Imdb").and_then(|v| v.as_str()) { imdb = val.to_string(); }
                        if let Some(val) = providers.get("Tmdb").and_then(|v| v.as_str()) { tmdb = val.to_string(); }
                    }
                    return Ok(Some((id.to_string(), imdb, tmdb)));
                }
            }
        }
        Ok(None)
    }

    pub async fn update_progress(&self, user_id: &str, item_id: &str, position_ticks: i64, played: bool) -> Result<()> {
        let path = format!("/Users/{}/Items/{}/UserData", user_id, item_id);
        let url = self.url_path(&path);
        let body = serde_json::json!({
            "PlaybackPositionTicks": position_ticks,
            "Played": played,
        });
        
        let resp = self.add_auth_headers(self.client.post(&url))
            .json(&body)
            .send()
            .await
            .context("Failed to send UserData progress update request")?;
        
        if !resp.status().is_success() {
            let body_text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("UserData progress update failed: {} - {}", url, body_text));
        }
        Ok(())
    }
}
