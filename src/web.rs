use axum::{
    Extension, Router,
    extract::Request,
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{Mutex, mpsc};

use crate::state::AppState;

#[derive(Clone)]
pub struct WebServerState {
    pub app_state: Arc<Mutex<AppState>>,
    pub reload_tx: mpsc::Sender<()>,
    #[allow(dead_code)]
    pub bind_addr: String,
    pub web_auth: Option<String>,
    pub version: String,
    pub started_at: String,
    pub started_instant: Instant,
}

const PUBLIC_PATHS: &[&str] = &[
    "/",
    "/manifest.json",
    "/sw.js",
    "/icon.svg",
    "/favicon.jpg",
    "/healthz",
];

pub fn create_router(web_state: Arc<WebServerState>) -> Router {
    let public = Router::new()
        .route("/", get(serve_index))
        .route("/manifest.json", get(serve_manifest))
        .route("/sw.js", get(serve_sw))
        .route("/icon.svg", get(serve_icon))
        .route("/favicon.jpg", get(serve_favicon))
        .route("/healthz", get(serve_healthz));

    let protected = Router::new()
        .route(
            "/api/config",
            get(crate::web_api::get_config).post(crate::web_api::post_config),
        )
        .route("/api/status", get(crate::web_api::get_status))
        .route("/api/poster", get(crate::web_api::serve_poster))
        .route(
            "/api/test_connection",
            get(crate::web_api::get_config).post(crate::web_api::test_connection),
        )
        .route(
            "/api/reload",
            axum::routing::post(crate::web_api::post_reload),
        )
        .route(
            "/api/users/refresh",
            axum::routing::post(crate::web_api::post_users_refresh),
        )
        .route(
            "/api/sync/force",
            axum::routing::post(crate::web_api::post_sync_force),
        )
        .route(
            "/api/sync/force/status",
            get(crate::web_api::get_sync_force_status),
        )
        .route(
            "/api/sync/force/cancel",
            axum::routing::post(crate::web_api::post_sync_force_cancel),
        )
        .route("/api/server-info", get(crate::web_api::get_server_info));

    public
        .merge(protected)
        .layer(middleware::from_fn(security_headers))
        .layer(middleware::from_fn_with_state(
            web_state.clone(),
            auth_middleware,
        ))
        .layer(Extension(web_state))
}

async fn security_headers(req: Request, next: Next) -> Response {
    let mut resp = next.run(req).await;
    let h = resp.headers_mut();
    h.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    h.insert(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );
    h.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("no-referrer"),
    );
    h.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("interest-cohort=()"),
    );
    if let Ok(csp) = HeaderValue::from_str(
        "default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src https://fonts.gstatic.com; script-src 'self' 'unsafe-inline'; connect-src 'self'",
    ) {
        h.insert(HeaderName::from_static("content-security-policy"), csp);
    }
    resp
}

async fn auth_middleware(
    Extension(state): Extension<Arc<WebServerState>>,
    req: Request,
    next: Next,
) -> Response {
    let token = state.web_auth.as_deref();
    let needs_auth = match token {
        None => false,
        Some(spec) => match spec.strip_prefix("bearer:") {
            Some(_) => true,
            None => {
                tracing::error!(
                    "STATESYNC_WEB_AUTH must start with 'bearer:' (got unsupported scheme); all protected endpoints will reject"
                );
                true
            }
        },
    };

    if !needs_auth {
        return next.run(req).await;
    }

    let path = req.uri().path().to_string();
    if PUBLIC_PATHS.contains(&path.as_str()) {
        return next.run(req).await;
    }

    let expected = match token.unwrap().strip_prefix("bearer:") {
        Some(t) => t,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                r#"{"error":"server misconfigured"}"#,
            )
                .into_response();
        }
    };
    if !constant_time_eq(&extract_bearer(req.headers()), expected) {
        return (
            StatusCode::UNAUTHORIZED,
            [("www-authenticate", "Bearer")],
            r#"{"error":"unauthorized"}"#,
        )
            .into_response();
    }

    next.run(req).await
}

fn extract_bearer(headers: &HeaderMap) -> String {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| {
            v.strip_prefix("Bearer ")
                .or_else(|| v.strip_prefix("bearer "))
        })
        .unwrap_or("")
        .to_string()
}

fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut acc = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        acc |= x ^ y;
    }
    acc == 0
}

async fn serve_index() -> Html<String> {
    Html(crate::dashboard::render_dashboard().into_string())
}

async fn serve_manifest() -> impl IntoResponse {
    (
        [("content-type", "application/manifest+json")],
        r##"{"name":"StateSync","short_name":"StateSync","start_url":"/","display":"standalone","background_color":"#03060f","theme_color":"#03060f","icons":[{"src":"/icon.svg","sizes":"192x192","type":"image/svg+xml"},{"src":"/icon.svg","sizes":"512x512","type":"image/svg+xml"}]}"##,
    )
}

async fn serve_icon() -> impl IntoResponse {
    (
        [("content-type", "image/svg+xml")],
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><rect width="100" height="100" fill="#03060f"/><circle cx="50" cy="50" r="30" stroke="#00f0ff" stroke-width="6" fill="none"/></svg>"##,
    )
}

async fn serve_favicon() -> impl IntoResponse {
    (
        [
            ("content-type", "image/jpeg"),
            ("cache-control", "public, max-age=86400, immutable"),
        ],
        include_bytes!("favicon.jpg").as_slice(),
    )
}

async fn serve_sw() -> impl IntoResponse {
    (
        [("content-type", "application/javascript")],
        "self.addEventListener('install',(e)=>{self.skipWaiting();});self.addEventListener('fetch',(e)=>{e.respondWith(fetch(e.request));});",
    )
}

async fn serve_healthz(Extension(state): Extension<Arc<WebServerState>>) -> impl IntoResponse {
    use crate::web_api::cache_stats;
    let stats = cache_stats(&state.app_state).await;
    let healthy = stats.total_servers > 0
        && (stats.connected_count > 0 || stats.ever_connected_count > 0 || stats.total_users > 0);
    let uptime = state.started_instant.elapsed().as_secs();
    let body = serde_json::json!({
        "status": if healthy { "healthy" } else { "starting" },
        "version": state.version,
        "uptime_seconds": uptime,
        "started_at": state.started_at,
        "servers": stats.total_servers,
        "connected": stats.connected_count,
        "users": stats.total_users,
    });
    let status = if healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (status, axum::Json(body))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq("hello", "hello"));
        assert!(!constant_time_eq("hello", "world"));
        assert!(!constant_time_eq("hello", "helloo"));
    }

    #[test]
    fn test_extract_bearer() {
        let mut headers = HeaderMap::new();
        headers.insert(axum::http::header::AUTHORIZATION, "Bearer mytoken".parse().unwrap());
        assert_eq!(extract_bearer(&headers), "mytoken");

        let mut headers_lower = HeaderMap::new();
        headers_lower.insert(axum::http::header::AUTHORIZATION, "bearer mytoken2".parse().unwrap());
        assert_eq!(extract_bearer(&headers_lower), "mytoken2");

        let headers_empty = HeaderMap::new();
        assert_eq!(extract_bearer(&headers_empty), "");
    }

    #[tokio::test]
    async fn test_serve_manifest() {
        let resp = serve_manifest().await.into_response();
        assert_eq!(resp.status(), axum::http::StatusCode::OK);
        assert_eq!(resp.headers().get("content-type").unwrap().to_str().unwrap(), "application/manifest+json");
    }

    #[tokio::test]
    async fn test_serve_icon() {
        let resp = serve_icon().await.into_response();
        assert_eq!(resp.status(), axum::http::StatusCode::OK);
        assert_eq!(resp.headers().get("content-type").unwrap().to_str().unwrap(), "image/svg+xml");
    }

    #[tokio::test]
    async fn test_serve_favicon() {
        let resp = serve_favicon().await.into_response();
        assert_eq!(resp.status(), axum::http::StatusCode::OK);
        assert_eq!(resp.headers().get("content-type").unwrap().to_str().unwrap(), "image/jpeg");
    }

    #[tokio::test]
    async fn test_serve_sw() {
        let resp = serve_sw().await.into_response();
        assert_eq!(resp.status(), axum::http::StatusCode::OK);
        assert_eq!(resp.headers().get("content-type").unwrap().to_str().unwrap(), "application/javascript");
    }

    #[tokio::test]
    async fn test_serve_healthz_unhealthy() {
        let app_state = Arc::new(Mutex::new(AppState::new(vec![])));
        let web_state = Arc::new(WebServerState {
            app_state,
            reload_tx: mpsc::channel(1).0,
            bind_addr: "127.0.0.1:0".to_string(),
            web_auth: None,
            version: "test".to_string(),
            started_at: "2025-01-01".to_string(),
            started_instant: Instant::now(),
        });
        
        let resp = serve_healthz(Extension(web_state)).await.into_response();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_serve_healthz_healthy() {
        let cache = crate::state::ServerCache {
            name: "test".to_string(),
            users: [("alice".to_string(), "u1".to_string())].into_iter().collect(),
            imdb_to_id: std::collections::HashMap::new(),
            tmdb_to_id: std::collections::HashMap::new(),
            id_to_providers: std::collections::HashMap::new(),
        };
        let app_state = Arc::new(Mutex::new(AppState::new(vec![cache])));
        // simulate connected status to trigger healthy
        {
            let mut st = app_state.lock().await;
            st.websocket_statuses = vec!["Connected".to_string()];
        }
        let web_state = Arc::new(WebServerState {
            app_state,
            reload_tx: mpsc::channel(1).0,
            bind_addr: "127.0.0.1:0".to_string(),
            web_auth: None,
            version: "test".to_string(),
            started_at: "2025-01-01".to_string(),
            started_instant: Instant::now(),
        });
        
        let resp = serve_healthz(Extension(web_state)).await.into_response();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
