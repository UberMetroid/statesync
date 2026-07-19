use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};
use tracing::{error, info, warn};

use crate::client::MediaClient;
use crate::config::Config;
use crate::state::{AppState, SyncHistoryValue};

static SYNC_SEMAPHORE: once_cell::sync::Lazy<Semaphore> = once_cell::sync::Lazy::new(|| {
    let permits = std::env::var("STATESYNC_MAX_SYNC_SPAWNS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(8);
    Semaphore::new(permits.max(1))
});

pub async fn sync_progress_to_targets(
    user_name: &str,
    source_item_id: &str,
    position: i64,
    played: bool,
    source_name: &str,
    source_index: usize,
    state_lock: &Arc<Mutex<AppState>>,
    target_clients: &[(usize, Arc<MediaClient>)],
    config: &Config,
    source_client: &Arc<MediaClient>,
) {
    let _permit = SYNC_SEMAPHORE.acquire().await;
    let user_lower = user_name.to_lowercase();

    let (imdb_id, tmdb_id) = {
        let state = state_lock.lock().await;
        if source_index >= state.caches.len() {
            return;
        }

        if let Some(provs) = state.caches[source_index]
            .id_to_providers
            .get(source_item_id)
        {
            provs.clone()
        } else {
            drop(state);
            let src_user_id = {
                let state_read = state_lock.lock().await;
                state_read.caches[source_index]
                    .users
                    .get(&user_lower)
                    .cloned()
            };

            if let Some(uid) = src_user_id {
                info!(
                    "Cache miss on '{}' for item {}. Resolving details dynamically...",
                    source_name, source_item_id
                );
                if let Ok((imdb, tmdb)) =
                    source_client.get_item_providers(&uid, source_item_id).await
                {
                    let mut state_write = state_lock.lock().await;
                    state_write.caches[source_index]
                        .id_to_providers
                        .insert(source_item_id.to_string(), (imdb.clone(), tmdb.clone()));
                    if !imdb.is_empty() {
                        state_write.caches[source_index]
                            .imdb_to_id
                            .insert(imdb.clone(), source_item_id.to_string());
                    }
                    if !tmdb.is_empty() {
                        state_write.caches[source_index]
                            .tmdb_to_id
                            .insert(tmdb.clone(), source_item_id.to_string());
                    }
                    (imdb, tmdb)
                } else {
                    return;
                }
            } else {
                return;
            }
        }
    };

    if imdb_id.is_empty() && tmdb_id.is_empty() {
        return;
    }

    for &(target_index, ref client_target) in target_clients {
        if config.servers[target_index].sync_direction == "send" {
            continue;
        }

        let mut state = state_lock.lock().await;
        let mut target_user_id = crate::state::find_mapped_user_id(
            &user_lower,
            &state.caches[target_index].users,
            &config.user_mappings,
        );
        if target_user_id.is_none() {
            drop(state);
            if let Ok(new_users) = client_target.get_users().await {
                let mut state_write = state_lock.lock().await;
                if target_index < state_write.caches.len() {
                    state_write.caches[target_index].users = new_users;
                }
            }
            state = state_lock.lock().await;
            target_user_id = crate::state::find_mapped_user_id(
                &user_lower,
                &state.caches[target_index].users,
                &config.user_mappings,
            );
        }

        let mut target_item_id = None;
        let target_name;
        let mut is_negative_cached = false;
        {
            let target_cache = &state.caches[target_index];
            if !imdb_id.is_empty() {
                target_item_id = target_cache.imdb_to_id.get(&imdb_id).cloned();
            }
            if target_item_id.is_none() && !tmdb_id.is_empty() {
                target_item_id = target_cache.tmdb_to_id.get(&tmdb_id).cloned();
            }
            target_name = target_cache.name.clone();
            if let Some(ref id) = target_item_id {
                if id == "[ NOT_FOUND ]" {
                    is_negative_cached = true;
                    target_item_id = None;
                }
            }
        }

        if target_item_id.is_none() && !is_negative_cached {
            drop(state);
            let mut resolved: Option<(String, String, String)> = None;
            let mut resolved_err: Option<String> = None;
            if let Some(ref t_uid) = target_user_id {
                info!(
                    "Cache miss on target '{}' for (IMDb: {}, TMDb: {}). Searching target library...",
                    target_name, imdb_id, tmdb_id
                );
                match client_target
                    .find_item_by_provider(t_uid, &imdb_id, &tmdb_id)
                    .await
                {
                    Ok(res) => resolved = res,
                    Err(e) => resolved_err = Some(e.to_string()),
                }
            }
            state = state_lock.lock().await;
            if let Some((id, _imdb, _tmdb)) = resolved {
                state.caches[target_index]
                    .id_to_providers
                    .insert(id.clone(), (imdb_id.clone(), tmdb_id.clone()));
                if !imdb_id.is_empty() {
                    state.caches[target_index]
                        .imdb_to_id
                        .insert(imdb_id.clone(), id.clone());
                }
                if !tmdb_id.is_empty() {
                    state.caches[target_index]
                        .tmdb_to_id
                        .insert(tmdb_id.clone(), id.clone());
                }
                target_item_id = Some(id);
            } else if resolved_err.is_none() {
                if !imdb_id.is_empty() {
                    state.caches[target_index]
                        .imdb_to_id
                        .insert(imdb_id.clone(), "[ NOT_FOUND ]".to_string());
                }
                if !tmdb_id.is_empty() {
                    state.caches[target_index]
                        .tmdb_to_id
                        .insert(tmdb_id.clone(), "[ NOT_FOUND ]".to_string());
                }
                state.caches[target_index].last_negative_cache =
                    Some(Instant::now() + Duration::from_secs(3600));
            } else if let Some(err) = resolved_err {
                warn!(
                    "Target '{}' lookup error (will not poison cache): {}",
                    target_name, err
                );
            }
        }

        if let (Some(t_item_id), Some(t_user_id)) = (target_item_id, target_user_id) {
            let now = Instant::now();
            let history_key = (
                user_lower.clone(),
                if !imdb_id.is_empty() {
                    imdb_id.clone()
                } else {
                    tmdb_id.clone()
                },
            );

            if let Some(last_sync) = state.last_syncs.get(&history_key) {
                let tick_diff = (last_sync.position_ticks - position).abs();
                let time_diff = last_sync.timestamp.elapsed();

                if tick_diff < (config.sync_threshold_seconds * 10_000_000) as i64
                    && time_diff < Duration::from_secs(config.sync_threshold_seconds)
                    && !played
                {
                    continue;
                }
            }

            let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
            let pos_secs = position as f64 / 10_000_000.0;
            let message = if played {
                format!(
                    "Synced watch state (watched) for {} to '{}'",
                    user_name, t_item_id
                )
            } else {
                format!("Synced progress for {} to {:.1}s", user_name, pos_secs)
            };

            let log_entry = crate::state::SyncLogEntry {
                timestamp,
                level: "success".to_string(),
                message: message.clone(),
                source_name: Some(source_name.to_string()),
                source_is_emby: Some(config.servers[source_index].is_emby),
                target_name: Some(target_name.clone()),
                target_is_emby: Some(config.servers[target_index].is_emby),
            };
            info!("{}", message);
            state.log_sync(log_entry);

            let client_target_clone = client_target.clone();
            let target_name_clone = target_name.clone();
            let state_lock_clone = state_lock.clone();
            let history_key_clone = history_key.clone();
            let t_item_id_for_update = t_item_id.clone();
            let t_user_id_for_update = t_user_id.clone();
            drop(state);

            tokio::spawn(async move {
                let res = client_target_clone
                    .update_progress(
                        &t_user_id_for_update,
                        &t_item_id_for_update,
                        position,
                        played,
                    )
                    .await;
                let mut state = state_lock_clone.lock().await;
                match res {
                    Ok(()) => {
                        state.last_syncs.insert(
                            history_key_clone,
                            SyncHistoryValue {
                                position_ticks: position,
                                timestamp: now,
                            },
                        );
                    }
                    Err(e) => {
                        error!("Error updating target playstate: {}", e);
                        state.log_event(
                            "error",
                            &format!("Sync failed to '{}': {}", target_name_clone, e),
                        );
                    }
                }
            });
        }
    }
}
