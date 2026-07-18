use std::collections::HashMap;
use std::time::Instant;
use anyhow::Result;
use crate::client::MediaClient;

#[derive(Debug, Clone)]
pub struct ServerCache {
    pub name: String,
    pub users: HashMap<String, String>, // username (lowercase) -> UserId
    pub imdb_to_id: HashMap<String, String>, // ImdbId -> ItemId
    pub tmdb_to_id: HashMap<String, String>, // TmdbId -> ItemId
    pub id_to_providers: HashMap<String, (String, String)>, // ItemId -> (ImdbId, TmdbId)
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SyncHistoryValue {
    pub position_ticks: i64,
    pub timestamp: Instant,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SyncLogEntry {
    pub timestamp: String,
    pub level: String, // "info", "warn", "error", "success"
    pub message: String,
    pub source_name: Option<String>,
    pub source_is_emby: Option<bool>,
    pub target_name: Option<String>,
    pub target_is_emby: Option<bool>,
}

pub struct AppState {
    pub caches: Vec<ServerCache>,
    pub last_syncs: HashMap<(String, String), SyncHistoryValue>,
    pub websocket_statuses: Vec<String>,
    pub sync_logs: Vec<SyncLogEntry>,
    pub active_sessions: HashMap<(String, String), (String, String, f64, bool, String)>,
}

impl AppState {
    pub fn new(caches: Vec<ServerCache>) -> Self {
        let count = caches.len();
        Self {
            caches,
            last_syncs: HashMap::new(),
            websocket_statuses: vec!["Offline".to_string(); count],
            sync_logs: Vec::new(),
            active_sessions: HashMap::new(),
        }
    }

    pub fn log_event(&mut self, level: &str, msg: &str) {
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        self.sync_logs.insert(0, SyncLogEntry {
            timestamp,
            level: level.to_string(),
            message: msg.to_string(),
            source_name: None,
            source_is_emby: None,
            target_name: None,
            target_is_emby: None,
        });
        if self.sync_logs.len() > 30 {
            self.sync_logs.truncate(30);
        }
    }

    pub fn log_sync(&mut self, entry: SyncLogEntry) {
        self.sync_logs.insert(0, entry);
        if self.sync_logs.len() > 30 {
            self.sync_logs.truncate(30);
        }
    }
}

pub async fn init_server_cache(name: &str, client: &MediaClient) -> Result<ServerCache> {
    let users = client.get_users().await?;
    let items = client.get_library_items().await?;
    
    let mut imdb_to_id = HashMap::new();
    let mut tmdb_to_id = HashMap::new();
    let mut id_to_providers = HashMap::new();
    
    for (id, (imdb, tmdb)) in items {
        if !imdb.is_empty() { imdb_to_id.insert(imdb.clone(), id.clone()); }
        if !tmdb.is_empty() { tmdb_to_id.insert(tmdb.clone(), id.clone()); }
        id_to_providers.insert(id, (imdb, tmdb));
    }
    
    Ok(ServerCache {
        name: name.to_string(),
        users,
        imdb_to_id,
        tmdb_to_id,
        id_to_providers,
    })
}

// Safe substring containment matching (allows "john doe" <-> "john" but prevents "John Doe" <-> "John Smith")
pub fn find_mapped_user_id(
    source_username: &str,
    target_users: &HashMap<String, String>,
    custom_mappings: &[Vec<String>],
) -> Option<String> {
    let src_lower = source_username.to_lowercase();

    // 1. Try Custom Mappings first
    for group in custom_mappings {
        if group.iter().any(|u| u.to_lowercase() == src_lower) {
            for mapped_name in group {
                let mapped_lower = mapped_name.to_lowercase();
                if mapped_lower != src_lower {
                    if let Some(id) = target_users.get(&mapped_lower) {
                        return Some(id.clone());
                    }
                }
            }
        }
    }

    // 2. Fall back to exact case-insensitive match
    if let Some(id) = target_users.get(&src_lower) {
        return Some(id.clone());
    }

    // 3. Fall back to safe substring containment matching
    for (tgt_name, tgt_id) in target_users {
        let tgt_lower = tgt_name.to_lowercase();
        if src_lower.contains(&tgt_lower) || tgt_lower.contains(&src_lower) {
            return Some(tgt_id.clone());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_username_match() {
        let mut target_users = HashMap::new();
        target_users.insert("john".to_string(), "id123".to_string());
        let mapped = find_mapped_user_id("JOHN", &target_users, &[]);
        assert_eq!(mapped, Some("id123".to_string()));
    }

    #[test]
    fn test_substring_username_match() {
        let mut target_users = HashMap::new();
        target_users.insert("john".to_string(), "id123".to_string());
        let mapped = find_mapped_user_id("John Doe", &target_users, &[]);
        assert_eq!(mapped, Some("id123".to_string()));
        
        let mut target_users2 = HashMap::new();
        target_users2.insert("john doe".to_string(), "id456".to_string());
        let mapped2 = find_mapped_user_id("john", &target_users2, &[]);
        assert_eq!(mapped2, Some("id456".to_string()));
    }

    #[test]
    fn test_custom_username_mapping_override() {
        let mut target_users = HashMap::new();
        target_users.insert("john_alt".to_string(), "id999".to_string());
        target_users.insert("john".to_string(), "id123".to_string());
        let custom_mappings = vec![vec!["john_special".to_string(), "john_alt".to_string()]];
        let mapped = find_mapped_user_id("john_special", &target_users, &custom_mappings);
        assert_eq!(mapped, Some("id999".to_string()));
    }

    #[test]
    fn test_username_collision_prevention() {
        let mut target_users = HashMap::new();
        target_users.insert("john smith".to_string(), "id777".to_string());
        let mapped = find_mapped_user_id("john doe", &target_users, &[]);
        assert_eq!(mapped, None);
    }
}
