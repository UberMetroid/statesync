use super::{Direction, ForceSyncError, ForceSyncStatus, SyncForceTracker};
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
