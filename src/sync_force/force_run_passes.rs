use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::sync::Semaphore;

use super::force_story::{apply_story, story_favorites, story_played};
use super::sync_loop::{force_sync_favorites_pair, force_sync_pair};
use super::{ForceContext, ForcePair, ForceSyncError, ForceSyncStatus, write_status};

pub async fn run_played_pass(
    ctx: &ForceContext,
    pairs: &[ForcePair],
    pair_n: u64,
    status: &mut ForceSyncStatus,
    processed_total: &mut u64,
    succeeded_total: &mut u64,
    skipped_total: &mut u64,
    failed_total: &mut u64,
    errors: &mut Vec<ForceSyncError>,
    semaphore: &Semaphore,
    min_interval: Duration,
) -> bool {
    let mut cancelled = false;
    {
        let mut st = ctx.state.lock().await;
        st.log_event(
            "info",
            "Force sync: starting watched-history pass (all linked routes)",
        );
    }
    for (i, (src_idx, tgt_idx, src_username, src_user_id, tgt_user_id)) in pairs.iter().enumerate()
    {
        if ctx.tracker.cancel.load(Ordering::SeqCst) {
            cancelled = true;
            break;
        }
        let src_name = ctx.config.servers[*src_idx].name.as_str();
        let tgt_name = ctx.config.servers[*tgt_idx].name.as_str();
        let pair_i = (i as u64) + 1;
        let (h, d) = story_played(
            src_username,
            src_name,
            tgt_name,
            pair_i,
            pair_n.max(1),
            ctx.dry_run,
        );
        apply_story(
            status,
            "played",
            h.clone(),
            d.clone(),
            Some(src_username),
            Some(src_name),
            Some(tgt_name),
            pair_i,
            pair_n,
        );
        write_status(&ctx.tracker, status);
        {
            let mut st = ctx.state.lock().await;
            st.log_event_detail("info", &h, Some(d));
        }

        cancelled = force_sync_pair(
            *src_idx,
            *tgt_idx,
            src_username,
            src_user_id,
            tgt_user_id,
            ctx,
            status,
            processed_total,
            succeeded_total,
            skipped_total,
            failed_total,
            errors,
            semaphore,
            min_interval,
        )
        .await;

        if cancelled {
            break;
        }
    }
    cancelled
}

pub async fn run_favorites_pass(
    ctx: &ForceContext,
    pairs: &[ForcePair],
    pair_n: u64,
    status: &mut ForceSyncStatus,
    processed_total: &mut u64,
    succeeded_total: &mut u64,
    skipped_total: &mut u64,
    failed_total: &mut u64,
    errors: &mut Vec<ForceSyncError>,
    semaphore: &Semaphore,
    min_interval: Duration,
) -> bool {
    let mut cancelled = false;
    {
        let mut st = ctx.state.lock().await;
        st.log_event(
            "info",
            "Force sync: starting favorites pass (all linked routes)",
        );
    }
    for (i, (src_idx, tgt_idx, src_username, src_user_id, tgt_user_id)) in pairs.iter().enumerate()
    {
        if ctx.tracker.cancel.load(Ordering::SeqCst) {
            cancelled = true;
            break;
        }
        let src_name = ctx.config.servers[*src_idx].name.as_str();
        let tgt_name = ctx.config.servers[*tgt_idx].name.as_str();
        let pair_i = (i as u64) + 1;
        let (h, d) = story_favorites(
            src_username,
            src_name,
            tgt_name,
            pair_i,
            pair_n.max(1),
            ctx.dry_run,
        );
        apply_story(
            status,
            "favorites",
            h.clone(),
            d.clone(),
            Some(src_username),
            Some(src_name),
            Some(tgt_name),
            pair_i,
            pair_n,
        );
        write_status(&ctx.tracker, status);
        {
            let mut st = ctx.state.lock().await;
            st.log_event_detail("info", &h, Some(d));
        }

        cancelled = force_sync_favorites_pair(
            *src_idx,
            *tgt_idx,
            src_username,
            src_user_id,
            tgt_user_id,
            ctx,
            status,
            processed_total,
            succeeded_total,
            skipped_total,
            failed_total,
            errors,
            semaphore,
            min_interval,
        )
        .await;

        if cancelled {
            break;
        }
    }
    cancelled
}
