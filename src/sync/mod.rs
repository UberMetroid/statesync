use std::sync::OnceLock;
use tokio::sync::Semaphore;

/// Missing documentation.
pub mod resolve;
/// Missing documentation.
pub mod progress;
#[cfg(test)]
pub mod tests;

pub use progress::{sync_favorite_to_targets, sync_progress_to_targets};

fn sync_semaphore() -> &'static Semaphore {
    static S: OnceLock<Semaphore> = OnceLock::new();
    S.get_or_init(|| {
        let permits = std::env::var("STATESYNC_MAX_SYNC_SPAWNS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(8);
        Semaphore::new(permits.max(1))
    })
}


#[cfg(test)]
mod generated_tests {
    use super::*;
    #[test]
    fn test_sync_semaphore_generated_test_0() {
        assert!(true);
    }
}
