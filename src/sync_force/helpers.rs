use super::{Direction, ForceContext, ForceSyncError, ForceSyncStatus, SyncForceTracker};
use std::sync::atomic::Ordering;
use std::time::Instant;

const FORCE_ERROR_CAP: usize = 100;

/// Always Both — legacy STATESYNC_FORCE_DIRECTION type filters are ignored.
pub fn direction_from_env() -> Direction {
    let _ = std::env::var("STATESYNC_FORCE_DIRECTION");
    Direction::Both
}

/// Publish counters so the WUI always sees movement (including pure skips).
pub fn publish_counts(
    tracker: &SyncForceTracker,
    status: &mut ForceSyncStatus,
    processed: u64,
    succeeded: u64,
    skipped: u64,
    failed: u64,
    last_write: &mut Instant,
    force: bool,
) {
    status.processed = processed;
    status.succeeded = succeeded;
    status.skipped = skipped;
    status.failed = failed;
    write_status_throttled(tracker, status, last_write, force);
}

/// Throttled status publish so huge libraries don't thrash the status mutex.
pub fn write_status_throttled(
    tracker: &SyncForceTracker,
    status: &ForceSyncStatus,
    last_write: &mut Instant,
    force: bool,
) {
    let now = Instant::now();
    // ~4 UI updates/sec while force runs — snappy without hammering clones.
    if force || now.duration_since(*last_write).as_millis() >= 250 {
        write_status(tracker, status);
        *last_write = now;
    }
}

pub fn push_error(
    errors: &mut Vec<ForceSyncError>,
    status: &mut ForceSyncStatus,
    err: ForceSyncError,
) {
    status.last_error = Some(err.message.clone());
    errors.push(err);
    if errors.len() > FORCE_ERROR_CAP {
        errors.remove(0);
    }
    status.errors = errors.clone();
}

/// Record a force failure on status **and** the activity log (plain language).
pub async fn record_force_error(
    ctx: &ForceContext,
    errors: &mut Vec<ForceSyncError>,
    status: &mut ForceSyncStatus,
    mut err: ForceSyncError,
) {
    // Prefer "Name (host:port)" when we can resolve the config entry.
    if let Some(s) = ctx.config.servers.iter().find(|s| {
        s.name == err.server || crate::config::server_display_label(&s.name, &s.url) == err.server
    }) {
        err.server = crate::config::server_display_label(&s.name, &s.url);
    }
    let who = if err.user.trim().is_empty() {
        "—".to_string()
    } else {
        err.user.clone()
    };
    let mut detail = format!(
        "Who: {} · Where: {} · Why: {}",
        who, err.server, err.message
    );
    if let Some(ref id) = err.item_id {
        detail.push_str(&format!(" · library item id: {id}"));
    }
    if let Some(ref p) = err.provider {
        detail.push_str(&format!(" · catalog id: {p}"));
    }
    push_error(errors, status, err);
    write_status(&ctx.tracker, status);
    {
        let mut st = ctx.state.lock().await;
        st.log_event_detail(
            "error",
            "Force: could not update a library title",
            Some(detail),
        );
    }
}

pub fn write_status(tracker: &SyncForceTracker, status: &ForceSyncStatus) {
    match tracker.status.lock() {
        Ok(mut lock) => *lock = status.clone(),
        Err(poisoned) => *poisoned.into_inner() = status.clone(),
    }
}

impl SyncForceTracker {
    pub fn snapshot_status(&self) -> ForceSyncStatus {
        match self.status.lock() {
            Ok(lock) => lock.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }

    pub fn cancel_backfill(&self) {
        self.cancel.store(true, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod label_tests {
    use super::*;
    use crate::config::server_display_label;

    #[test]
    fn server_display_combines_name_and_host_port() {
        assert_eq!(
            server_display_label("Home Emby", "http://10.0.0.5:8096/"),
            "Home Emby (10.0.0.5:8096)"
        );
        assert_eq!(
            server_display_label("10.0.0.5:8096", "http://10.0.0.5:8096"),
            "10.0.0.5:8096"
        );
    }

    #[test]
    fn push_error_keeps_cap() {
        let mut status = ForceSyncStatus::idle();
        let mut errors = Vec::new();
        for i in 0..105 {
            push_error(
                &mut errors,
                &mut status,
                ForceSyncError {
                    user: "u".into(),
                    server: "s".into(),
                    item_id: None,
                    provider: None,
                    message: format!("e{i}"),
                },
            );
        }
        assert_eq!(errors.len(), 100);
        assert!(status.last_error.as_deref().unwrap().contains("e104"));
    }
}
