use std::sync::atomic::Ordering;
use super::{Direction, ForceSyncError, ForceSyncStatus, SyncForceTracker};

const FORCE_ERROR_CAP: usize = 100;

pub fn direction_from_env() -> Direction {
    match std::env::var("STATESYNC_FORCE_DIRECTION")
        .unwrap_or_default()
        .to_lowercase()
        .as_str()
    {
        "embytojellyfin" | "emby_to_jellyfin" => Direction::EmbyToJellyfin,
        "jellyfintoemby" | "jellyfin_to_emby" => Direction::JellyfinToEmby,
        _ => Direction::Both,
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

pub fn write_status(tracker: &SyncForceTracker, status: &ForceSyncStatus) {
    if let Ok(mut lock) = tracker.status.try_lock() {
        *lock = status.clone();
    }
}

impl SyncForceTracker {
    pub fn snapshot_status(&self) -> ForceSyncStatus {
        if let Ok(lock) = self.status.try_lock() {
            lock.clone()
        } else {
            ForceSyncStatus::idle()
        }
    }

    pub fn cancel_backfill(&self) {
        self.cancel.store(true, Ordering::SeqCst);
    }
}
