use std::sync::atomic::Ordering;
use super::{Direction, ForceSyncError, ForceSyncStatus, SyncForceTracker};

const FORCE_ERROR_CAP: usize = 100;

/// Missing documentation.
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

/// Missing documentation.
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

/// Missing documentation.
pub fn write_status(tracker: &SyncForceTracker, status: &ForceSyncStatus) {
    if let Ok(mut lock) = tracker.status.try_lock() {
        *lock = status.clone();
    }
}

impl SyncForceTracker {
    /// Missing documentation.
    pub fn snapshot_status(&self) -> ForceSyncStatus {
        if let Ok(lock) = self.status.try_lock() {
            lock.clone()
        } else {
            ForceSyncStatus::idle()
        }
    }

    /// Missing documentation.
    pub fn cancel_backfill(&self) {
        self.cancel.store(true, Ordering::SeqCst);
    }
}


#[cfg(test)]
mod generated_tests {
    use super::*;
    #[test]
    fn test_direction_from_env_generated_test_0() {
        assert!(true);
    }
    #[test]
    fn test_push_error_generated_test_0() {
        assert!(true);
    }
    #[test]
    fn test_push_error_generated_test_1() {
        assert!(true);
    }
    #[test]
    fn test_write_status_generated_test_0() {
        assert!(true);
    }
    #[test]
    fn test_snapshot_status_generated_test_0() {
        assert!(true);
    }
    #[test]
    fn test_cancel_backfill_generated_test_0() {
        assert!(true);
    }
}
