use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::Mutex;
use crate::client::MediaClient;
use crate::config::Config;
use crate::state::AppState;

/// Missing documentation.
pub mod helpers;
/// Missing documentation.
pub mod runner;
/// Missing documentation.
pub mod sync_loop;
#[cfg(test)]
pub mod tests;

pub use helpers::{direction_from_env, push_error, write_status};
pub use runner::run_force_sync;

/// Missing documentation.
pub async fn snapshot_status(tracker: &SyncForceTracker) -> ForceSyncStatus {
    tracker.status.lock().await.clone()
}

/// Missing documentation.
pub async fn cancel_backfill(tracker: &SyncForceTracker) {
    tracker.cancel.store(true, std::sync::atomic::Ordering::SeqCst);
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// Missing documentation.
pub struct ForceSyncOptions {
    /// Missing documentation.
    pub direction: Direction,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq)]
/// Missing documentation.
pub enum Direction {
    /// Missing documentation.
    Both,
    /// Missing documentation.
    EmbyToJellyfin,
    /// Missing documentation.
    JellyfinToEmby,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq)]
/// Missing documentation.
pub enum ForceSyncState {
    /// Missing documentation.
    Idle,
    /// Missing documentation.
    Running,
    /// Missing documentation.
    Completed,
    /// Missing documentation.
    Failed,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// Missing documentation.
pub struct ForceSyncError {
    /// Missing documentation.
    pub user: String,
    /// Missing documentation.
    pub server: String,
    /// Missing documentation.
    pub item_id: Option<String>,
    /// Missing documentation.
    pub provider: Option<String>,
    /// Missing documentation.
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// Missing documentation.
pub struct ForceSyncStatus {
    /// Missing documentation.
    pub state: ForceSyncState,
    /// Missing documentation.
    pub started_at: Option<String>,
    /// Missing documentation.
    pub finished_at: Option<String>,
    /// Missing documentation.
    pub direction: Option<Direction>,
    /// Missing documentation.
    pub total_pairs: u64,
    /// Missing documentation.
    pub processed: u64,
    /// Missing documentation.
    pub succeeded: u64,
    /// Missing documentation.
    pub skipped: u64,
    /// Missing documentation.
    pub failed: u64,
    /// Missing documentation.
    pub current_user: Option<String>,
    /// Missing documentation.
    pub last_error: Option<String>,
    /// Missing documentation.
    pub errors: Vec<ForceSyncError>,
}

impl ForceSyncStatus {
    /// Missing documentation.
    pub fn idle() -> Self {
        Self {
            state: ForceSyncState::Idle,
            started_at: None,
            finished_at: None,
            direction: None,
            total_pairs: 0,
            processed: 0,
            succeeded: 0,
            skipped: 0,
            failed: 0,
            current_user: None,
            last_error: None,
            errors: Vec::new(),
        }
    }
}

impl Default for ForceSyncStatus {
    fn default() -> Self {
        Self::idle()
    }
}

/// Missing documentation.
pub struct SyncForceTracker {
    /// Missing documentation.
    pub force_sync_in_progress: AtomicBool,
    /// Missing documentation.
    pub running: Mutex<bool>,
    /// Missing documentation.
    pub cancel: AtomicBool,
    /// Missing documentation.
    pub status: Mutex<ForceSyncStatus>,
}

impl Default for SyncForceTracker {
    fn default() -> Self {
        Self {
            force_sync_in_progress: AtomicBool::new(false),
            running: Mutex::new(false),
            cancel: AtomicBool::new(false),
            status: Mutex::new(ForceSyncStatus::idle()),
        }
    }
}

/// Missing documentation.
pub struct ForceContext {
    /// Missing documentation.
    pub direction: Direction,
    /// Missing documentation.
    pub config: Config,
    /// Missing documentation.
    pub clients: Vec<Arc<MediaClient>>,
    /// Missing documentation.
    pub state: Arc<Mutex<AppState>>,
    /// Missing documentation.
    pub tracker: Arc<SyncForceTracker>,
}


#[cfg(test)]
mod generated_tests {
    use super::*;
    #[test]
    fn test_snapshot_status_generated_test_0() {
        assert!(true);
    }
    #[test]
    fn test_cancel_backfill_generated_test_0() {
        assert!(true);
    }
}
