use super::helpers::record_force_error;
use super::{ForceContext, ForceSyncError, ForceSyncStatus, write_status};
use crate::client::PlayedItem;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use super::force_constants::{FORCE_ITEM_CAP, FORCE_PAGE_TIMEOUT};

pub async fn force_sync_favorites_pair(
    src_idx: usize,
    tgt_idx: usize,
    src_username: &str,
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
    if !ctx.config.sync.force_favorites {
        return false;
    }
    let source_client = ctx.clients[src_idx].clone();
    let page_size: usize = 500;
    let mut last_status_write = Instant::now() - Duration::from_secs(1);

    let mut page: usize = 0;
    let mut cancelled = false;
    loop {
        if ctx.tracker.cancel.load(Ordering::SeqCst) {
            cancelled = true;
            break;
        }
        if page * page_size >= FORCE_ITEM_CAP {
            break;
        }
        let items_res = tokio::time::timeout(
            FORCE_PAGE_TIMEOUT,
            source_client.get_user_favorite_items(src_user_id, page * page_size, page_size),
        )
        .await;
        let items: Vec<PlayedItem> = match items_res {
            Ok(Ok(items)) => items,
            Ok(Err(e)) => {
                record_force_error(
                    ctx,
                    errors,
                    status,
                    ForceSyncError {
                        user: src_username.to_string(),
                        server: ctx.config.servers[src_idx].name.clone(),
                        item_id: None,
                        provider: None,
                        message: format!("could not list favorites: {e}"),
                    },
                )
                .await;
                *failed_total += 1;
                status.by_field.favorite.fail += 1;
                write_status(&ctx.tracker, status);
                break;
            }
            Err(_) => {
                record_force_error(
                    ctx,
                    errors,
                    status,
                    ForceSyncError {
                        user: src_username.to_string(),
                        server: ctx.config.servers[src_idx].name.clone(),
                        item_id: None,
                        provider: None,
                        message: format!(
                            "timed out listing favorites after {:?}",
                            FORCE_PAGE_TIMEOUT
                        ),
                    },
                )
                .await;
                *failed_total += 1;
                status.by_field.favorite.fail += 1;
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
            let permit = semaphore.acquire().await;
            if let Ok(permit) = permit {
                super::force_favorites_item::process_favorite_item(
                    item,
                    src_idx,
                    tgt_idx,
                    src_username,
                    tgt_user_id,
                    ctx,
                    status,
                    processed_total,
                    succeeded_total,
                    skipped_total,
                    failed_total,
                    errors,
                    permit,
                    &mut last_status_write,
                    min_interval,
                )
                .await;
            }
        }
        if cancelled {
            break;
        }
        page += 1;
    }
    write_status(&ctx.tracker, status);
    cancelled
}
