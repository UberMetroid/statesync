use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::Mutex;
use crate::client::MediaClient;
use crate::config::Config;
use crate::state::AppState;

pub mod helpers;
pub mod runner;
pub mod sync_loop;
#[cfg(test)]
pub mod tests;

pub use helpers::{direction_from_env, push_error, write_status};
pub use runner::run_force_sync;

pub async fn snapshot_status(tracker: &SyncForceTracker) -> ForceSyncStatus {
    tracker.status.lock().await.clone()
}

pub async fn cancel_backfill(tracker: &SyncForceTracker) {
    tracker.cancel.store(true, std::sync::atomic::Ordering::SeqCst);
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ForceSyncOptions {
    pub direction: Direction,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum Direction {
    Both,
    EmbyToJellyfin,
    JellyfinToEmby,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum ForceSyncState {
    Idle,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ForceSyncError {
    pub user: String,
    pub server: String,
    pub item_id: Option<String>,
    pub provider: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ForceSyncStatus {
    pub state: ForceSyncState,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub direction: Option<Direction>,
    pub total_pairs: u64,
    pub processed: u64,
    pub succeeded: u64,
    pub skipped: u64,
    pub failed: u64,
    pub current_user: Option<String>,
    pub last_error: Option<String>,
    pub errors: Vec<ForceSyncError>,
}

impl ForceSyncStatus {
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

pub struct SyncForceTracker {
    pub force_sync_in_progress: AtomicBool,
    pub running: Mutex<bool>,
    pub cancel: AtomicBool,
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

pub struct ForceContext {
    pub direction: Direction,
    pub config: Config,
    pub clients: Vec<Arc<MediaClient>>,
    pub state: Arc<Mutex<AppState>>,
    pub tracker: Arc<SyncForceTracker>,
}
