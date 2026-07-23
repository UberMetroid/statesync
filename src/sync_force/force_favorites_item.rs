use std::time::{Duration, Instant};
use tokio::sync::SemaphorePermit;

use super::helpers::publish_counts;
use super::{ForceContext, ForceSyncError, ForceSyncStatus};
use crate::client::PlayedItem;
use crate::state::SyncHistoryValue;

pub async fn process_favorite_item(
    item: PlayedItem,
    _src_idx: usize,
    tgt_idx: usize,
    src_username: &str,
    tgt_user_id: &str,
    ctx: &ForceContext,
    status: &mut ForceSyncStatus,
    processed_total: &mut u64,
    succeeded_total: &mut u64,
    skipped_total: &mut u64,
    failed_total: &mut u64,
    errors: &mut Vec<ForceSyncError>,
    permit: SemaphorePermit<'_>,
    last_status_write: &mut Instant,
    min_interval: Duration,
) {
    let started_item = Instant::now();
    let providers = item.provider_ids();
    if providers.is_empty() {
        drop(permit);
        *skipped_total += 1;
        *processed_total += 1;
        status.by_field.favorite.skip += 1;
        status.skip_reasons.no_provider += 1;
        publish_counts(
            &ctx.tracker,
            status,
            *processed_total,
            *succeeded_total,
            *skipped_total,
            *failed_total,
            last_status_write,
            false,
        );
        return;
    }

    let target_client = ctx.clients[tgt_idx].clone();
    let target_name = ctx.config.servers[tgt_idx].name.clone();
    let target_item_id = crate::sync::resolve::resolve_target_item(
        tgt_idx,
        &providers,
        &target_name,
        Some(tgt_user_id),
        &target_client,
        &ctx.state,
    )
    .await;
    let target_item_id = match target_item_id {
        Some(id) => id,
        None => {
            drop(permit);
            *skipped_total += 1;
            *processed_total += 1;
            status.by_field.favorite.skip += 1;
            status.skip_reasons.no_match += 1;
            publish_counts(
                &ctx.tracker,
                status,
                *processed_total,
                *succeeded_total,
                *skipped_total,
                *failed_total,
                last_status_write,
                false,
            );
            return;
        }
    };

    if let Ok(tgt_ud) = target_client
        .get_item_user_data(tgt_user_id, &target_item_id)
        .await
    {
        if tgt_ud.is_favorite == Some(true) {
            drop(permit);
            *skipped_total += 1;
            *processed_total += 1;
            status.by_field.favorite.skip += 1;
            status.skip_reasons.already_equal += 1;
            publish_counts(
                &ctx.tracker,
                status,
                *processed_total,
                *succeeded_total,
                *skipped_total,
                *failed_total,
                last_status_write,
                false,
            );
            return;
        }
    }

    let update_res = if ctx.dry_run {
        Ok(Ok(()))
    } else {
        tokio::time::timeout(
            super::force_constants::FORCE_UPDATE_TIMEOUT,
            target_client.update_favorite(tgt_user_id, &target_item_id, true),
        )
        .await
    };

    match update_res {
        Ok(Ok(())) => {
            if !ctx.dry_run {
                if let Some(hk) = providers.history_key() {
                    let key = (src_username.to_lowercase(), hk);
                    let mut st = ctx.state.lock().await;
                    let prev = st.last_syncs.get(&key).cloned();
                    st.last_syncs.insert(
                        key,
                        SyncHistoryValue {
                            position_ticks: prev.as_ref().map(|p| p.position_ticks).unwrap_or(0),
                            timestamp: Instant::now(),
                            played: prev.as_ref().map(|p| p.played).unwrap_or(false),
                            favorite: Some(true),
                        },
                    );
                    drop(st);
                }
            }
            *succeeded_total += 1;
            *processed_total += 1;
            status.by_field.favorite.ok += 1;
        }
        Ok(Err(e)) => {
            super::helpers::record_force_error(
                ctx,
                errors,
                status,
                ForceSyncError {
                    user: src_username.to_string(),
                    server: ctx.config.servers[tgt_idx].name.clone(),
                    item_id: Some(target_item_id),
                    provider: providers.history_key(),
                    message: format!("could not write favorite: {e}"),
                },
            )
            .await;
            *failed_total += 1;
            *processed_total += 1;
            status.by_field.favorite.fail += 1;
        }
        Err(_) => {
            super::helpers::record_force_error(
                ctx,
                errors,
                status,
                ForceSyncError {
                    user: src_username.to_string(),
                    server: ctx.config.servers[tgt_idx].name.clone(),
                    item_id: Some(target_item_id),
                    provider: providers.history_key(),
                    message: format!(
                        "timed out writing favorite after {:?}",
                        super::force_constants::FORCE_UPDATE_TIMEOUT
                    ),
                },
            )
            .await;
            *failed_total += 1;
            *processed_total += 1;
            status.by_field.favorite.fail += 1;
        }
    }
    drop(permit);
    let elapsed = started_item.elapsed();
    if elapsed < min_interval {
        tokio::time::sleep(min_interval - elapsed).await;
    }
    publish_counts(
        &ctx.tracker,
        status,
        *processed_total,
        *succeeded_total,
        *skipped_total,
        *failed_total,
        last_status_write,
        false,
    );
}
