use super::live_sync::{make_cache, make_config, make_server_config};
use crate::client::ProviderIds;
use crate::state::AppState;

#[tokio::test]
async fn test_sync_progress_to_targets_success() {
    let mut server_target = mockito::Server::new_async().await;
    let mock_update = server_target
        .mock("POST", "/Users/u2/Items/item_jf/UserData")
        .with_status(200)
        .with_body(r#"{"Played": true, "PlaybackPositionTicks": 5000}"#)
        .create_async()
        .await;

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
            c.index_item(
                "item_emby".to_string(),
                ProviderIds::from_parts("imdb123", "", ""),
            );
            c
        },
        {
            let mut c = make_cache("jellyfin", vec![("alice", "u2")]);
            c.imdb_to_id
                .insert("imdb123".to_string(), "item_jf".to_string());
            c
        },
    ];
    let app_state = std::sync::Arc::new(tokio::sync::Mutex::new(AppState::new(caches)));
    let client_source = std::sync::Arc::new(crate::client::MediaClient::new(
        "http://source".to_string(),
        "key".to_string(),
        true,
    ));
    let client_target = std::sync::Arc::new(crate::client::MediaClient::new(
        server_target.url(),
        "key".to_string(),
        false,
    ));

    crate::sync::sync_progress_to_targets(
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
        Some("Test Movie".to_string()),
    )
    .await;

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    mock_update.assert_async().await;

    let st = app_state.lock().await;
    let key = ("alice".to_string(), "imdb:imdb123".to_string());
    assert!(st.last_syncs.contains_key(&key));
}
