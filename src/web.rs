use std::sync::Arc;
use axum::{
    routing::get,
    Json, Router, response::Html, Extension,
};
use tokio::sync::{mpsc, Mutex};
use serde_json::json;

use crate::config::{Config, ServerConfig};
use crate::state::AppState;

pub struct WebServerState {
    pub app_state: Arc<Mutex<AppState>>,
    pub reload_tx: mpsc::Sender<()>,
}

pub fn create_router(web_state: Arc<WebServerState>) -> Router {
    Router::new()
        .route("/", get(serve_index))
        .route("/api/config", get(get_config).post(post_config))
        .route("/api/status", get(get_status))
        .layer(Extension(web_state))
}

async fn serve_index() -> Html<&'static str> {
    Html(include_str!("index.html"))
}

async fn get_config(Extension(state): Extension<Arc<WebServerState>>) -> Json<Config> {
    let app_state = state.app_state.lock().await;
    // Build Config DTO
    let mut servers = Vec::new();
    for cache in &app_state.caches {
        servers.push(ServerConfig {
            name: cache.name.clone(),
            url: String::new(), // Don't expose URLs/keys in get config directly for security, or keep them?
            // Actually, to edit them, we should expose the URLs. We will keep API keys masked or expose for simplicity.
            api_key: String::new(), 
            is_emby: false,
        });
    }
    // Better yet: we just read the raw config file to return it accurately!
    let current_config = Config::load().unwrap_or(Config { servers: vec![], sync_threshold_seconds: 5 });
    Json(current_config)
}

async fn post_config(
    Extension(state): Extension<Arc<WebServerState>>,
    Json(new_config): Json<Config>,
) -> Json<serde_json::Value> {
    let path = crate::config::get_config_path();
    
    // Save to file
    let serialized = serde_json::to_string_pretty(&new_config).unwrap_or_default();
    if let Err(e) = std::fs::write(path, serialized) {
        return Json(json!({ "status": "error", "message": format!("Failed to save config: {}", e) }));
    }

    // Trigger reload
    let _ = state.reload_tx.send(()).await;
    Json(json!({ "status": "ok", "message": "Configuration updated. Reloading sync..." }))
}

async fn get_status(Extension(state): Extension<Arc<WebServerState>>) -> Json<serde_json::Value> {
    let app_state = state.app_state.lock().await;
    let mut servers_status = Vec::new();
    
    for cache in &app_state.caches {
        servers_status.push(json!({
            "name": cache.name,
            "users_count": cache.users.len(),
            "media_count": cache.id_to_providers.len()
        }));
    }
    
    Json(json!({
        "status": "active",
        "servers": servers_status
    }))
}
