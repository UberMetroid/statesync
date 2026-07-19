use axum::{
    Extension, Json, body::Body, extract::Query, http::StatusCode, response::IntoResponse,
    response::Response,
};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use crate::client::MediaClient;
use crate::config::{Config, validate_config};
use crate::web::WebServerState;

const ITEM_ID_RE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-";
const MAX_ITEM_ID_LEN: usize = 64;
const MAX_SERVER_NAME_LEN: usize = 64;

#[derive(Debug, Deserialize)]
pub struct TestConnRequest {
    pub url: String,
    pub api_key: String,
    pub is_emby: bool,
}

pub fn mask_api_key(key: &str) -> String {
    if key.is_empty() {
        "".to_string()
    } else if key.len() <= 8 {
        "••••••••".to_string()
    } else {
        format!("{}••••••••{}", &key[..4], &key[key.len() - 4..])
    }
}

pub async fn get_config() -> Json<Config> {
    let mut config = Config::load().unwrap_or(Config {
        servers: vec![],
        sync_threshold_seconds: 5,
        user_mappings: vec![],
    });
    for s in &mut config.servers {
        s.api_key = mask_api_key(&s.api_key);
    }
    Json(config)
}

pub async fn post_config(
    Extension(state): Extension<Arc<WebServerState>>,
    Json(mut new_config): Json<Config>,
) -> Json<serde_json::Value> {
    if let Ok(old_config) = Config::load() {
        for s in &mut new_config.servers {
            if s.api_key.contains('•') || s.api_key.trim().is_empty() {
                if let Some(old_s) = old_config
                    .servers
                    .iter()
                    .find(|os| os.url == s.url || os.name == s.name)
                {
                    s.api_key = old_s.api_key.clone();
                }
            }
        }
    }

    if let Err(_e) = validate_config(&new_config) {
        return Json(json!({ "status": "error", "message": "Invalid configuration" }));
    }

    let path = crate::config::get_config_path();
    let serialized = match serde_json::to_string_pretty(&new_config) {
        Ok(s) => s,
        Err(_) => {
            return Json(
                json!({ "status": "error", "message": "Failed to serialize configuration" }),
            );
        }
    };
    if std::fs::write(path, serialized).is_err() {
        return Json(json!({ "status": "error", "message": "Failed to write configuration file" }));
    }

    let _ = state.reload_tx.send(()).await;
    Json(json!({ "status": "ok", "message": "Configuration saved. Sync service is reloading..." }))
}

pub async fn test_connection(Json(req): Json<TestConnRequest>) -> Json<serde_json::Value> {
    if req.url.len() > 512 || !(req.url.starts_with("http://") || req.url.starts_with("https://")) {
        return Json(json!({
            "status": "error",
            "message": "Invalid URL (must be http(s):// and <= 512 chars)"
        }));
    }
    if req.api_key.len() > 256 {
        return Json(json!({
            "status": "error",
            "message": "API key too long"
        }));
    }
    let client = MediaClient::new(req.url, req.api_key, req.is_emby);
    match client.get_users().await {
        Ok(users) => Json(json!({
            "status": "ok",
            "message": format!("Success! Connected to server and found {} users.", users.len())
        })),
        Err(_) => Json(json!({
            "status": "error",
            "message": "Connection failed (see server logs for details)"
        })),
    }
}

fn valid_item_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= MAX_ITEM_ID_LEN && id.bytes().all(|b| ITEM_ID_RE.contains(&b))
}

fn valid_server_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= MAX_SERVER_NAME_LEN
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.')
}

pub async fn serve_poster(
    Extension(_state): Extension<Arc<WebServerState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let server_name = params.get("server").cloned().unwrap_or_default();
    let item_id = params.get("item_id").cloned().unwrap_or_default();

    if !valid_server_name(&server_name) || !valid_item_id(&item_id) {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from("Bad Request"))
            .unwrap();
    }

    let config = match Config::load() {
        Ok(cfg) => cfg,
        Err(_) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Internal Error"))
                .unwrap();
        }
    };
    let server_cfg = match config.servers.iter().find(|s| s.name == server_name) {
        Some(s) => s,
        None => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Not Found"))
                .unwrap();
        }
    };

    let client = MediaClient::new(
        server_cfg.url.clone(),
        server_cfg.api_key.clone(),
        server_cfg.is_emby,
    );
    let path = format!("/Items/{}/Images/Primary", item_id);
    let url = client.url_path(&path);
    let builder = client.add_auth_headers(client.client.get(&url));

    match tokio::time::timeout(std::time::Duration::from_secs(10), builder.send()).await {
        Ok(Ok(resp)) => {
            let content_type = resp
                .headers()
                .get("content-type")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("image/jpeg")
                .to_string();
            if let Ok(bytes) = resp.bytes().await {
                let mut res = Response::new(Body::from(bytes));
                if let Ok(val) = axum::http::HeaderValue::from_str(&content_type) {
                    res.headers_mut()
                        .insert(axum::http::header::CONTENT_TYPE, val);
                }
                return res;
            }
        }
        Ok(Err(_)) | Err(_) => {}
    }
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::empty())
        .unwrap()
}

pub async fn get_status(
    Extension(state): Extension<Arc<WebServerState>>,
) -> Json<serde_json::Value> {
    use crate::config::redacted_url;
    let app_state = state.app_state.lock().await;
    let config = Config::load().unwrap_or_else(|_| Config {
        servers: vec![],
        sync_threshold_seconds: 5,
        user_mappings: vec![],
    });
    let redacted_url_for = |name: &str| -> String {
        config
            .servers
            .iter()
            .find(|s| s.name == name)
            .map(|s| redacted_url(&s.url))
            .unwrap_or_default()
    };

    let mut servers_status = Vec::new();
    for (i, cache) in app_state.caches.iter().enumerate() {
        let ws_status = app_state
            .websocket_statuses
            .get(i)
            .cloned()
            .unwrap_or_else(|| "Offline".to_string());
        servers_status.push(json!({
            "name": cache.name,
            "url": redacted_url_for(&cache.name),
            "users_count": cache.users.len(),
            "media_count": cache.id_to_providers.len(),
            "websocket_status": ws_status
        }));
    }

    let mut mapped_users = Vec::new();
    let mut processed = Vec::new();
    for (srv_idx, cache) in app_state.caches.iter().enumerate() {
        for username in cache.users.keys() {
            if processed.contains(&(srv_idx, username.clone())) {
                continue;
            }
            let mut group = vec![None; app_state.caches.len()];
            group[srv_idx] = Some(username.clone());
            processed.push((srv_idx, username.clone()));
            let mut has_any_match = false;
            for (other_idx, other_cache) in app_state.caches.iter().enumerate() {
                if other_idx == srv_idx {
                    continue;
                }
                let matched_name = crate::state::find_mapped_user_id(
                    username,
                    &other_cache.users,
                    &config.user_mappings,
                )
                .and_then(|target_id| {
                    other_cache
                        .users
                        .iter()
                        .find(|(_, id)| *id == &target_id)
                        .map(|(name, _)| name.clone())
                });
                if let Some(name) = matched_name {
                    group[other_idx] = Some(name.clone());
                    processed.push((other_idx, name));
                    has_any_match = true;
                }
            }
            if has_any_match {
                mapped_users.push(group);
            }
        }
    }

    let mut active_sessions = Vec::new();
    for ((server, _), (user, item, position, is_paused, item_id)) in &app_state.active_sessions {
        let poster_url = format!(
            "/api/poster?server={}&item_id={}",
            utf8_percent_encode(server, NON_ALPHANUMERIC),
            item_id
        );
        active_sessions.push(json!({
            "server": server,
            "user": user,
            "item": item,
            "position": position,
            "is_paused": is_paused,
            "poster_url": poster_url
        }));
    }

    let public_logs: Vec<_> = app_state
        .sync_logs
        .iter()
        .map(|log| {
            json!({
                "timestamp": log.timestamp,
                "level": log.level,
                "message": log.message,
                "source_name": log.source_name,
                "source_is_emby": log.source_is_emby,
                "target_name": log.target_name,
                "target_is_emby": log.target_is_emby,
            })
        })
        .collect();

    Json(json!({
        "status": "active",
        "servers": servers_status,
        "mapped_users": mapped_users,
        "active_sessions": active_sessions,
        "sync_logs": public_logs
    }))
}

pub async fn post_reload(Extension(state): Extension<Arc<WebServerState>>) -> impl IntoResponse {
    if let Err(e) = state.reload_tx.send(()).await {
        tracing::error!("Failed to trigger config reload: {}", e);
        return Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from("Failed to trigger reload"))
            .unwrap();
    }
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from("Reload triggered successfully"))
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_api_key() {
        assert_eq!(mask_api_key(""), "");
        assert_eq!(mask_api_key("12345"), "••••••••");
        assert_eq!(mask_api_key("12345678"), "••••••••");
        assert_eq!(mask_api_key("123456789"), "1234••••••••6789");
        assert_eq!(mask_api_key("my_secret_token_1234"), "my_s••••••••1234");
    }

    #[test]
    fn test_valid_item_id() {
        assert!(valid_item_id("abc123XYZ_-"));
        assert!(!valid_item_id(""));
        assert!(!valid_item_id("../etc/passwd"));
        assert!(!valid_item_id("a b"));
        assert!(!valid_item_id(&"a".repeat(MAX_ITEM_ID_LEN + 1)));
    }

    #[test]
    fn test_valid_server_name() {
        assert!(valid_server_name("green"));
        assert!(valid_server_name("my-server_01.local"));
        assert!(!valid_server_name(""));
        assert!(!valid_server_name("../etc"));
        assert!(!valid_server_name("name with space"));
    }
}
