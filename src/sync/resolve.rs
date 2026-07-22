use crate::client::{MediaClient, ProviderIds};
use crate::config::Config;
use crate::state::AppState;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

pub async fn resolve_item_providers(
    source_index: usize,
    source_item_id: &str,
    source_client: &Arc<MediaClient>,
    user_lower: &str,
    state_lock: &Arc<Mutex<AppState>>,
    source_name: &str,
) -> Option<ProviderIds> {
    let state = state_lock.lock().await;
    if source_index >= state.caches.len() {
        return None;
    }

    if let Some(provs) = state.caches[source_index]
        .id_to_providers
        .get(source_item_id)
    {
        Some(provs.clone())
    } else {
        drop(state);
        let src_user_id = {
            let state_read = state_lock.lock().await;
            state_read.caches[source_index]
                .users
                .get(user_lower)
                .cloned()
        };

        if let Some(uid) = src_user_id {
            debug!(
                "Cache miss on '{}' for item {}. Resolving details dynamically...",
                source_name, source_item_id
            );
            if let Ok(provs) = source_client.get_item_providers(&uid, source_item_id).await {
                let mut state_write = state_lock.lock().await;
                state_write.caches[source_index]
                    .index_item(source_item_id.to_string(), provs.clone());
                Some(provs)
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub async fn resolve_target_user(
    target_index: usize,
    user_lower: &str,
    client_target: &Arc<MediaClient>,
    config: &Config,
    state_lock: &Arc<Mutex<AppState>>,
) -> Option<String> {
    let mut state = state_lock.lock().await;
    let mut target_user_id = crate::state::find_mapped_user_id(
        user_lower,
        &state.caches[target_index].users,
        &config.user_mappings,
    );
    if target_user_id.is_none() {
        drop(state);
        if let Ok(new_users) = client_target.get_users().await {
            let mut state_write = state_lock.lock().await;
            if target_index < state_write.caches.len() {
                state_write.caches[target_index].merge_users(new_users);
            }
        }
        state = state_lock.lock().await;
        target_user_id = crate::state::find_mapped_user_id(
            user_lower,
            &state.caches[target_index].users,
            &config.user_mappings,
        );
    }
    target_user_id
}

/// Resolve target library item: RAM cache first, then at most one HTTP search per
/// provider id (Imdb → Tmdb → Tvdb), marking misses so we never re-query the same id.
pub async fn resolve_target_item(
    target_index: usize,
    providers: &ProviderIds,
    target_name: &str,
    target_user_id: Option<&str>,
    client_target: &Arc<MediaClient>,
    state_lock: &Arc<Mutex<AppState>>,
) -> Option<String> {
    if providers.is_empty() {
        return None;
    }
    let t_uid = target_user_id?;

    // Up to 3 provider ids, one HTTP each, stop on first hit.
    for _ in 0..3 {
        let next = {
            let state = state_lock.lock().await;
            if target_index >= state.caches.len() {
                return None;
            }
            let cache = &state.caches[target_index];
            if let Some(id) = cache.lookup_item_id(providers) {
                return Some(id);
            }
            if cache.is_negative_cached(providers) {
                return None;
            }
            cache
                .next_http_search(providers)
                .map(|(t, id)| (t, id.to_string()))
        };

        let Some((ptype, pid)) = next else {
            return None;
        };

        debug!(
            "HTTP search on '{}' for {}={}",
            target_name, ptype, pid
        );

        match client_target
            .find_item_one_provider(t_uid, ptype, &pid)
            .await
        {
            Ok(Some((id, found))) => {
                let mut state = state_lock.lock().await;
                let mut merged = providers.clone();
                if merged.imdb.is_empty() {
                    merged.imdb = found.imdb;
                }
                if merged.tmdb.is_empty() {
                    merged.tmdb = found.tmdb;
                }
                if merged.tvdb.is_empty() {
                    merged.tvdb = found.tvdb;
                }
                state.caches[target_index].index_item(id.clone(), merged);
                return Some(id);
            }
            Ok(None) => {
                let mut state = state_lock.lock().await;
                state.caches[target_index].index_one_not_found(ptype, &pid);
                // Loop: try next provider id if any.
            }
            Err(e) => {
                // Network error — do not poison cache; bail this item.
                tracing::warn!(
                    "Target '{}' lookup error for {}={} (will not poison cache): {}",
                    target_name,
                    ptype,
                    pid,
                    e
                );
                return None;
            }
        }
    }
    None
}


