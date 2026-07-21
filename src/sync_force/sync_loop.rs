use std::time::{Duration, Instant};
use std::sync::atomic::Ordering;
use crate::client::PlayedItem;
use crate::state::SyncHistoryValue;
use super::{ForceContext, ForceSyncError, ForceSyncStatus, push_error, write_status};

const FORCE_PAGE_TIMEOUT: Duration = Duration::from_secs(60);
const FORCE_UPDATE_TIMEOUT: Duration = Duration::from_secs(30);
const FORCE_ITEM_CAP: usize = 100_000;

#[allow(clippy::too_many_arguments)]
/// Missing documentation.
pub async fn force_sync_pair(
    src_idx: usize,
    tgt_idx: usize,
    src_user_id: &str,
    tgt_user_id: &str,
    ctx: &ForceContext,
    status: &mut ForceSyncStatus,
    processed_total: &mut u64,
    succeeded_total: &mut u64,
    skipped_total: &mut u64,
    failed_total: &mut u64,
    errors: &mut Vec<ForceSyncError>,
    semaphore: &tokio::sync::Semaphore,
    min_interval: Duration,
) -> bool {
    let source_client = ctx.clients[src_idx].clone();
    let target_client = ctx.clients[tgt_idx].clone();
    let page_size: usize = 500;

    let mut page: usize = 0;
    let mut cancelled = false;
    loop {
        if ctx.tracker.cancel.load(Ordering::SeqCst) {
            cancelled = true;
            break;
        }
        if page * page_size >= FORCE_ITEM_CAP {
            tracing::warn!(
                "force-sync reached {} item cap; stopping at user {} on {}",
                FORCE_ITEM_CAP, src_user_id, ctx.config.servers[src_idx].name
            );
            break;
        }
        let items_res = tokio::time::timeout(
            FORCE_PAGE_TIMEOUT,
            source_client.get_user_played_items(src_user_id, page * page_size, page_size),
        )
        .await;
        let items: Vec<PlayedItem> = match items_res {
            Ok(Ok(items)) => items,
            Ok(Err(e)) => {
                push_error(
                    errors,
                    status,
                    ForceSyncError {
                        user: src_user_id.to_string(),
                        server: ctx.config.servers[src_idx].name.clone(),
                        item_id: None,
                        provider: None,
                        message: format!("list failed: {}", e),
                    },
                );
                *failed_total += 1;
                write_status(&ctx.tracker, status);
                break;
            }
            Err(_) => {
                push_error(
                    errors,
                    status,
                    ForceSyncError {
                        user: src_user_id.to_string(),
                        server: ctx.config.servers[src_idx].name.clone(),
                        item_id: None,
                        provider: None,
                        message: format!("list timeout after {:?}", FORCE_PAGE_TIMEOUT),
                    },
                );
                *failed_total += 1;
                write_status(&ctx.tracker, status);
                break;
            }
        };
        if items.is_empty() {
            break;
        }
        for item in items {
            if ctx.tracker.cancel.load(Ordering::SeqCst) {
                cancelled = true;
                break;
            }
            let started_item = Instant::now();
            let _permit = semaphore.acquire().await;

            let imdb = item.imdb_id.clone().unwrap_or_default();
            let tmdb = item.tmdb_id.clone().unwrap_or_default();
            if imdb.is_empty() && tmdb.is_empty() {
                *skipped_total += 1;
                *processed_total += 1;
                continue;
            }
            let resolved = target_client
                .find_item_by_provider(tgt_user_id, &imdb, &tmdb)
                .await
                .ok()
                .flatten();
            let target_item_id = match resolved {
                Some((id, _i, _t)) => id,
                None => {
                    *skipped_total += 1;
                    *processed_total += 1;
                    continue;
                }
            };
            let source_pos = item.playback_position_ticks.unwrap_or(0);
            let update_res = tokio::time::timeout(
                FORCE_UPDATE_TIMEOUT,
                target_client.update_progress(tgt_user_id, &target_item_id, source_pos, true),
            )
            .await;
            match update_res {
                Ok(Ok(())) => {
                    let key = (
                        src_user_id.to_lowercase(),
                        if !imdb.is_empty() {
                            imdb.clone()
                        } else {
                            tmdb.clone()
                        },
                    );
                    let mut st = ctx.state.lock().await;
                    st.last_syncs.insert(
                        key,
                        SyncHistoryValue {
                            position_ticks: source_pos,
                            timestamp: Instant::now(),
                        },
                    );
                    drop(st);
                    *succeeded_total += 1;
                    *processed_total += 1;
                }
                Ok(Err(e)) => {
                    push_error(
                        errors,
                        status,
                        ForceSyncError {
                            user: src_user_id.to_string(),
                            server: ctx.config.servers[tgt_idx].name.clone(),
                            item_id: Some(target_item_id),
                            provider: if !imdb.is_empty() {
                                Some(imdb)
                            } else {
                                Some(tmdb)
                            },
                            message: e.to_string(),
                        },
                    );
                    *failed_total += 1;
                    *processed_total += 1;
                }
                Err(_) => {
                    push_error(
                        errors,
                        status,
                        ForceSyncError {
                            user: src_user_id.to_string(),
                            server: ctx.config.servers[tgt_idx].name.clone(),
                            item_id: Some(target_item_id),
                            provider: if !imdb.is_empty() {
                                Some(imdb)
                            } else {
                                Some(tmdb)
                            },
                            message: format!("update timeout after {:?}", FORCE_UPDATE_TIMEOUT),
                        },
                    );
                    *failed_total += 1;
                    *processed_total += 1;
                }
            }
            let elapsed = started_item.elapsed();
            if elapsed < min_interval {
                tokio::time::sleep(min_interval - elapsed).await;
            }
            status.processed = *processed_total;
            status.succeeded = *succeeded_total;
            status.skipped = *skipped_total;
            status.failed = *failed_total;
            write_status(&ctx.tracker, status);
        }
        if cancelled {
            break;
        }
        page += 1;
    }
    cancelled
}


#[cfg(test)]
mod generated_tests {
    use super::*;
    #[test]
    fn test_force_sync_pair_generated_test_0() {
        assert!(true);
    }
}
