//! Now-playing poster proxy (Emby/Jellyfin Primary art — not screenshots).
//! Fetches once per item and serves from an in-memory cache.

use super::poster_cache::{self, CachedPoster};
use super::validation::{valid_item_id, valid_server_name};
use crate::client::MediaClient;
use crate::config::Config;
use crate::web::WebServerState;
use axum::{Extension, body::Body, extract::Query, http::StatusCode, response::Response};
use std::sync::Arc;
use std::time::Duration;

const MAX_POSTER_BYTES: u64 = 2 * 1024 * 1024;

pub async fn serve_poster(
    Extension(_state): Extension<Arc<WebServerState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let server_name = params.get("server").cloned().unwrap_or_default();
    let item_id = params.get("item_id").cloned().unwrap_or_default();

    if !valid_server_name(&server_name) || !valid_item_id(&item_id) {
        return bad_request();
    }

    if let Some(cached) = poster_cache::get(&server_name, &item_id) {
        return poster_response(cached.bytes, &cached.content_type);
    }

    let config = match Config::load() {
        Ok(cfg) => cfg,
        Err(_) => return internal_error(),
    };
    let server_cfg = match config.servers.iter().find(|s| s.name == server_name) {
        Some(s) => s,
        None => return not_found("Not Found"),
    };

    let client = MediaClient::new(
        server_cfg.url.clone(),
        server_cfg.api_key.clone(),
        server_cfg.is_emby,
    );
    // Primary = library cover/poster art (not a live screenshot).
    let path = format!("/Items/{}/Images/Primary?maxWidth=120&quality=80", item_id);
    let url = client.url_path(&path);
    let builder = client.add_auth_headers(client.client.get(&url));

    match tokio::time::timeout(Duration::from_secs(10), builder.send()).await {
        Ok(Ok(resp)) => {
            if !resp.status().is_success() {
                return not_found("No poster");
            }
            if let Some(len) = resp.content_length() {
                if len > MAX_POSTER_BYTES {
                    return not_found("No poster");
                }
            }
            let content_type = resp
                .headers()
                .get("content-type")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("image/jpeg")
                .to_string();
            let is_image = content_type.starts_with("image/");
            if let Ok(bytes) = resp.bytes().await {
                if bytes.len() as u64 > MAX_POSTER_BYTES || !is_image || bytes.is_empty() {
                    return not_found("No poster");
                }
                let bytes = bytes.to_vec();
                poster_cache::put(
                    &server_name,
                    &item_id,
                    CachedPoster {
                        bytes: bytes.clone(),
                        content_type: content_type.clone(),
                    },
                );
                return poster_response(bytes, &content_type);
            }
        }
        Ok(Err(_)) | Err(_) => {}
    }
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())
        .unwrap_or_else(|_| internal_error())
}

fn poster_response(bytes: Vec<u8>, content_type: &str) -> Response {
    let mut res = Response::new(Body::from(bytes));
    if let Ok(val) = axum::http::HeaderValue::from_str(content_type) {
        res.headers_mut()
            .insert(axum::http::header::CONTENT_TYPE, val);
    }
    // Stable per item_id — browser should keep it without revalidation noise.
    res.headers_mut().insert(
        axum::http::header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("private, max-age=86400, immutable"),
    );
    res
}

fn bad_request() -> Response {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::from("Bad Request"))
        .unwrap_or_else(|_| internal_error())
}

fn not_found(msg: &'static str) -> Response {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from(msg))
        .unwrap_or_else(|_| internal_error())
}

fn internal_error() -> Response {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from("Internal Server Error"))
        .unwrap_or_default()
}
