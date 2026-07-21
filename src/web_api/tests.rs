#[cfg(test)]
mod tests {
    use super::super::config::mask_api_key;
    use super::super::validation::{valid_item_id, valid_server_name};

    #[test]
    fn test_mask_api_key() {
        assert_eq!(mask_api_key(""), "");
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
        assert!(!valid_item_id(&"a".repeat(65)));
    }

    #[test]
    fn test_valid_server_name() {
        assert!(valid_server_name("green"));
        assert!(valid_server_name("my-server_01.local"));
        assert!(!valid_server_name(""));
        assert!(!valid_server_name("../etc"));
        assert!(!valid_server_name("name with space"));
    }

    #[test]
    fn test_valid_server_url() {
        assert!(super::super::server::valid_server_url("http://localhost:8096"));
        assert!(super::super::server::valid_server_url("https://emby.example.com"));
        assert!(!super::super::server::valid_server_url("ftp://localhost:8096"));
        assert!(!super::super::server::valid_server_url("http://localhost/../etc"));
        assert!(!super::super::server::valid_server_url(&format!("http://{}", "a".repeat(510))));
    }

    #[tokio::test]
    async fn test_cache_stats() {
        use crate::state::{AppState, ServerCache};
        use std::sync::Arc;
        use tokio::sync::Mutex;

        let app_state = Arc::new(Mutex::new(AppState::new(vec![])));
        let stats = super::super::status::cache_stats(&app_state).await;
        assert_eq!(stats.total_servers, 0);
        assert_eq!(stats.total_users, 0);

        let cache = ServerCache {
            name: "test_server".to_string(),
            users: [("alice".to_string(), "u1".to_string())].into_iter().collect(),
            imdb_to_id: std::collections::HashMap::new(),
            tmdb_to_id: std::collections::HashMap::new(),
            id_to_providers: std::collections::HashMap::new(),
        };
        let app_state_with_cache = Arc::new(Mutex::new(AppState::new(vec![cache])));
        let stats2 = super::super::status::cache_stats(&app_state_with_cache).await;
        assert_eq!(stats2.total_servers, 1);
        assert_eq!(stats2.total_users, 1);
    }

    #[tokio::test]
    async fn test_post_reload_channel_closed() {
        use axum::Extension;
        use crate::web::WebServerState;
        use crate::state::AppState;
        use tokio::sync::mpsc;
        use std::sync::Arc;
        use tokio::sync::Mutex;
        use axum::response::IntoResponse;

        let app_state = Arc::new(Mutex::new(AppState::new(vec![])));
        let (tx, rx) = mpsc::channel(1);
        drop(rx); // close receiver to force send failure

        let web_state = Arc::new(WebServerState {
            app_state,
            reload_tx: tx,
            bind_addr: "127.0.0.1:0".to_string(),
            web_auth: None,
            version: "test".to_string(),
            started_at: "2025-01-01".to_string(),
            started_instant: std::time::Instant::now(),
        });

        let resp = super::super::sync::post_reload(Extension(web_state)).await.into_response();
        assert_eq!(resp.status(), axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_post_reload_success() {
        use axum::Extension;
        use crate::web::WebServerState;
        use crate::state::AppState;
        use tokio::sync::mpsc;
        use std::sync::Arc;
        use tokio::sync::Mutex;
        use axum::response::IntoResponse;

        let app_state = Arc::new(Mutex::new(AppState::new(vec![])));
        let (tx, _rx) = mpsc::channel(2);

        let web_state = Arc::new(WebServerState {
            app_state,
            reload_tx: tx,
            bind_addr: "127.0.0.1:0".to_string(),
            web_auth: None,
            version: "test".to_string(),
            started_at: "2025-01-01".to_string(),
            started_instant: std::time::Instant::now(),
        });

        let resp = super::super::sync::post_reload(Extension(web_state)).await.into_response();
        assert_eq!(resp.status(), axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_test_connection_invalid_params() {
        use axum::Json;
        use super::super::server::{test_connection, TestConnRequest};

        let req_bad_url = TestConnRequest {
            url: "ftp://bad-scheme".to_string(),
            api_key: "key".to_string(),
            is_emby: false,
        };
        let res = test_connection(Json(req_bad_url)).await;
        assert_eq!(res.get("status").unwrap().as_str().unwrap(), "error");

        let req_long_key = TestConnRequest {
            url: "http://localhost:8096".to_string(),
            api_key: "a".repeat(257),
            is_emby: false,
        };
        let res2 = test_connection(Json(req_long_key)).await;
        assert_eq!(res2.get("status").unwrap().as_str().unwrap(), "error");
    }

    #[tokio::test]
    async fn test_test_connection_returns_detailed_error() {
        use axum::Json;
        use super::super::server::{test_connection, TestConnRequest};

        let req_fail = TestConnRequest {
            url: "http://127.0.0.1:1".to_string(), // Unreachable port
            api_key: "key".to_string(),
            is_emby: false,
        };
        let res = test_connection(Json(req_fail)).await;
        assert_eq!(res.get("status").unwrap().as_str().unwrap(), "error");
        let msg = res.get("message").unwrap().as_str().unwrap();
        assert!(msg.contains("Connection failed"));
    }
}
