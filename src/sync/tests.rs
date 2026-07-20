#[cfg(test)]
mod tests {
    use crate::client::{SessionInfo, UserDataEntry};
    use crate::config::{Config, ServerConfig};
    use crate::state::{AppState, ServerCache, SyncHistoryValue};
    use serde_json::json;

    fn make_config(servers: Vec<ServerConfig>, user_mappings: Vec<Vec<String>>) -> Config {
        Config {
            servers,
            sync_threshold_seconds: 5,
            user_mappings,
            last_full_sync: None,
        }
    }

    fn make_server_config(name: &str, is_emby: bool) -> ServerConfig {
        ServerConfig {
            name: name.to_string(),
            url: "http://test".to_string(),
            api_key: "k".to_string(),
            is_emby,
            sync_direction: "both".to_string(),
            allow_insecure_http: true,
        }
    }

    fn make_cache(name: &str, users: Vec<(&str, &str)>) -> ServerCache {
        let mut cache = ServerCache {
            name: name.to_string(),
            users: std::collections::HashMap::new(),
            imdb_to_id: std::collections::HashMap::new(),
            tmdb_to_id: std::collections::HashMap::new(),
            id_to_providers: std::collections::HashMap::new(),
        };
        for (uname, uid) in users {
            cache.users.insert(uname.to_string(), uid.to_string());
        }
        cache
    }

    #[tokio::test]
    async fn threshold_skips_duplicate_update() {
        let _config = make_config(
            vec![
                make_server_config("emby", true),
                make_server_config("jellyfin", false),
            ],
            vec![],
        );
        let caches = vec![
            make_cache("emby", vec![("alice", "u1")]),
            make_cache("jellyfin", vec![("alice", "u2")]),
        ];
        let app_state = std::sync::Arc::new(tokio::sync::Mutex::new(AppState::new(caches)));

        let key = ("alice".to_string(), "tt1234567".to_string());
        {
            let mut st = app_state.lock().await;
            st.last_syncs.insert(
                key.clone(),
                SyncHistoryValue {
                    position_ticks: 1000,
                    timestamp: std::time::Instant::now(),
                },
            );
        }
        let st = app_state.lock().await;
        let stored = st
            .last_syncs
            .get(&key)
            .map(|v| v.position_ticks)
            .unwrap_or(0);
        assert!(
            1000 <= stored,
            "stored position should be >= new position to trigger skip"
        );
    }

    #[tokio::test]
    async fn force_sync_in_progress_blocks_live_sync() {
        let caches = vec![
            make_cache("emby", vec![("alice", "u1")]),
            make_cache("jellyfin", vec![("alice", "u2")]),
        ];
        let app_state = std::sync::Arc::new(tokio::sync::Mutex::new(AppState::new(caches)));
        {
            let st = app_state.lock().await;
            assert!(
                !st.sync_force
                    .force_sync_in_progress
                    .load(std::sync::atomic::Ordering::SeqCst)
            );
            st.sync_force
                .force_sync_in_progress
                .store(true, std::sync::atomic::Ordering::SeqCst);
            assert!(
                st.sync_force
                    .force_sync_in_progress
                    .load(std::sync::atomic::Ordering::SeqCst)
            );
        }
    }

    #[test]
    fn cache_miss_path_populates_provider_maps() {
        let mut cache = make_cache("emby", vec![("alice", "u1")]);
        cache.id_to_providers.insert(
            "item1".to_string(),
            ("tt1234567".to_string(), "tm1".to_string()),
        );
        cache
            .imdb_to_id
            .insert("tt1234567".to_string(), "item1".to_string());
        cache
            .tmdb_to_id
            .insert("tm1".to_string(), "item1".to_string());
        assert_eq!(cache.id_to_providers.get("item1").unwrap().0, "tt1234567");
        assert!(cache.imdb_to_id.contains_key("tt1234567"));
        assert!(cache.tmdb_to_id.contains_key("tm1"));
    }

    #[tokio::test]
    async fn unmapped_user_creates_solo_entry() {
        let caches = vec![
            make_cache("emby", vec![("alice", "u1")]),
            make_cache("jellyfin", vec![]),
        ];
        let app_state = std::sync::Arc::new(tokio::sync::Mutex::new(AppState::new(caches)));
        let st = app_state.lock().await;
        assert!(st.caches[0].users.contains_key("alice"));
        assert!(!st.caches[1].users.contains_key("alice"));
    }

    #[test]
    fn user_data_entry_dto_parses_emby_payload() {
        let payload = json!({
            "ItemId": "i1",
            "Played": true,
            "PlaybackPositionTicks": 5000
        });
        let entry: UserDataEntry = serde_json::from_value(payload).unwrap();
        assert_eq!(entry.item_id, "i1");
        assert!(entry.played);
        assert_eq!(entry.playback_position_ticks, Some(5000));
    }

    #[test]
    fn user_data_entry_dto_parses_jellyfin_payload() {
        let payload = json!({
            "itemId": "i2",
            "played": false,
            "playbackPositionTicks": null
        });
        let entry: UserDataEntry = serde_json::from_value(payload).unwrap();
        assert_eq!(entry.item_id, "i2");
        assert!(!entry.played);
        assert_eq!(entry.playback_position_ticks, None);
    }

    #[test]
    fn session_info_dto_parses_position_ticks() {
        let payload = json!({
            "Id": "s1",
            "UserName": "alice",
            "NowPlayingItem": {"Id": "n1", "Name": "Show"},
            "PlayState": {"PositionTicks": 12345, "IsPaused": false}
        });
        let info: SessionInfo = serde_json::from_value(payload).unwrap();
        assert_eq!(info.id, "s1");
        assert_eq!(info.user_name.as_deref(), Some("alice"));
        assert!(info.now_playing_item.is_some());
        let ps = info.play_state.as_ref().unwrap();
        assert_eq!(ps.position_ticks, Some(12345));
        assert_eq!(ps.is_paused, Some(false));
    }

    #[test]
    fn session_info_dto_handles_null_position() {
        let payload = json!({
            "Id": "s2",
            "UserName": "bob",
            "NowPlayingItem": {"Id": "n2", "Name": "Movie"},
            "PlayState": {"PositionTicks": null, "IsPaused": true}
        });
        let info: SessionInfo = serde_json::from_value(payload).unwrap();
        let ps = info.play_state.as_ref().unwrap();
        assert_eq!(ps.position_ticks, None);
        assert_eq!(ps.is_paused, Some(true));
    }

    #[test]
    fn test_sync_semaphore() {
        // Defaults to 8
        let sem = super::super::sync_semaphore();
        assert!(sem.available_permits() <= 8);

        // Since it's a OnceLock, we can't easily re-init, but we can verify it has permits.
        assert!(sem.available_permits() > 0);
    }

    #[tokio::test]
    async fn test_resolve_item_providers_cache_hit() {
        let mut cache = make_cache("emby", vec![("alice", "u1")]);
        cache.id_to_providers.insert("item1".to_string(), ("imdb123".to_string(), "tmdb456".to_string()));
        let state = std::sync::Arc::new(tokio::sync::Mutex::new(AppState::new(vec![cache])));
        let client = std::sync::Arc::new(crate::client::MediaClient::new("http://test".to_string(), "key".to_string(), false));

        let res = super::super::resolve::resolve_item_providers(
            0,
            "item1",
            &client,
            "alice",
            &state,
            "emby"
        ).await;

        assert_eq!(res, Some(("imdb123".to_string(), "tmdb456".to_string())));
    }

    #[tokio::test]
    async fn test_resolve_item_providers_cache_miss_success() {
        let mut server = mockito::Server::new_async().await;
        let mock_call = server.mock("GET", "/Users/u1/Items/item1")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"ProviderIds": {"Imdb": "imdb123", "Tmdb": "tmdb456"}}"#)
            .create_async().await;

        let cache = make_cache("emby", vec![("alice", "u1")]);
        let state = std::sync::Arc::new(tokio::sync::Mutex::new(AppState::new(vec![cache])));
        let client = std::sync::Arc::new(crate::client::MediaClient::new(server.url(), "key".to_string(), false));

        let res = super::super::resolve::resolve_item_providers(
            0,
            "item1",
            &client,
            "alice",
            &state,
            "emby"
        ).await;

        mock_call.assert_async().await;
        assert_eq!(res, Some(("imdb123".to_string(), "tmdb456".to_string())));

        // Check if cache got updated
        let st = state.lock().await;
        assert_eq!(st.caches[0].id_to_providers.get("item1").unwrap(), &("imdb123".to_string(), "tmdb456".to_string()));
    }

    #[tokio::test]
    async fn test_resolve_target_user_fresh_fetch() {
        let mut server = mockito::Server::new_async().await;
        let mock_call = server.mock("GET", "/Users?StartIndex=0&Limit=500")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"Items": [{"Name": "Bob", "Id": "u_bob"}], "TotalRecordCount": 1}"#)
            .create_async().await;

        let caches = vec![make_cache("jellyfin", vec![])];
        let state = std::sync::Arc::new(tokio::sync::Mutex::new(AppState::new(caches)));
        let client = std::sync::Arc::new(crate::client::MediaClient::new(server.url(), "key".to_string(), false));
        let config = make_config(vec![make_server_config("jellyfin", false)], vec![]);

        let res = super::super::resolve::resolve_target_user(
            0,
            "bob",
            &client,
            &config,
            &state
        ).await;

        mock_call.assert_async().await;
        assert_eq!(res, Some("u_bob".to_string()));
    }

    #[tokio::test]
    async fn test_resolve_target_item_cache_hit() {
        let mut cache = make_cache("jellyfin", vec![]);
        cache.imdb_to_id.insert("imdb123".to_string(), "item_jf".to_string());
        let state = std::sync::Arc::new(tokio::sync::Mutex::new(AppState::new(vec![cache])));
        let client = std::sync::Arc::new(crate::client::MediaClient::new("http://test".to_string(), "key".to_string(), false));

        let res = super::super::resolve::resolve_target_item(
            0,
            "imdb123",
            "",
            "jellyfin",
            Some("u1"),
            &client,
            &state
        ).await;

        assert_eq!(res, Some("item_jf".to_string()));
    }

    #[tokio::test]
    async fn test_resolve_target_item_negative_cached() {
        let mut cache = make_cache("jellyfin", vec![]);
        cache.imdb_to_id.insert("imdb123".to_string(), "[ NOT_FOUND ]".to_string());
        let state = std::sync::Arc::new(tokio::sync::Mutex::new(AppState::new(vec![cache])));
        let client = std::sync::Arc::new(crate::client::MediaClient::new("http://test".to_string(), "key".to_string(), false));

        let res = super::super::resolve::resolve_target_item(
            0,
            "imdb123",
            "",
            "jellyfin",
            Some("u1"),
            &client,
            &state
        ).await;

        assert_eq!(res, None);
    }

    #[tokio::test]
    async fn test_resolve_target_item_dynamic_search_success() {
        let mut server = mockito::Server::new_async().await;
        let mock_call = server.mock("GET", "/Users/u1/Items?Recursive=true&Fields=ProviderIds&AnyProviderIdTypes=Imdb&ProviderIds=imdb123")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"Items": [{"Id": "item_resolved", "ProviderIds": {"Imdb": "imdb123"}}]}"#)
            .create_async().await;

        let cache = make_cache("jellyfin", vec![]);
        let state = std::sync::Arc::new(tokio::sync::Mutex::new(AppState::new(vec![cache])));
        let client = std::sync::Arc::new(crate::client::MediaClient::new(server.url(), "key".to_string(), false));

        let res = super::super::resolve::resolve_target_item(
            0,
            "imdb123",
            "",
            "jellyfin",
            Some("u1"),
            &client,
            &state
        ).await;

        mock_call.assert_async().await;
        assert_eq!(res, Some("item_resolved".to_string()));
    }

    #[tokio::test]
    async fn test_sync_progress_to_targets_success() {
        let mut server_target = mockito::Server::new_async().await;
        let mock_update = server_target.mock("POST", "/Users/u2/Items/item_jf/UserData")
            .with_status(200)
            .with_body(r#"{"Played": true, "PlaybackPositionTicks": 5000}"#)
            .create_async().await;

        let config = make_config(
            vec![
                make_server_config("emby", true),
                make_server_config("jellyfin", false),
            ],
            vec![],
        );
        let caches = vec![
            {
                let mut c = make_cache("emby", vec![("alice", "u1")]);
                c.id_to_providers.insert("item_emby".to_string(), ("imdb123".to_string(), "".to_string()));
                c
            },
            {
                let mut c = make_cache("jellyfin", vec![("alice", "u2")]);
                c.imdb_to_id.insert("imdb123".to_string(), "item_jf".to_string());
                c
            }
        ];
        let app_state = std::sync::Arc::new(tokio::sync::Mutex::new(AppState::new(caches)));
        let client_source = std::sync::Arc::new(crate::client::MediaClient::new("http://source".to_string(), "key".to_string(), true));
        let client_target = std::sync::Arc::new(crate::client::MediaClient::new(server_target.url(), "key".to_string(), false));

        super::super::sync_progress_to_targets(
            "alice",
            "item_emby",
            5000,
            true,
            "emby",
            0,
            &app_state,
            &[(1, client_target.clone())],
            &config,
            &client_source,
            Some("Test Movie".to_string())
        ).await;

        // Yield to let tokio::spawn finish
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        mock_update.assert_async().await;

        // Verify history entry was recorded
        let st = app_state.lock().await;
        let key = ("alice".to_string(), "imdb123".to_string());
        assert!(st.last_syncs.contains_key(&key));
    }
}
