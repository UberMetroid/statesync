#[cfg(test)]
mod tests {
    use crate::sync_force::{ForceSyncState, ForceSyncStatus, SyncForceTracker, ForceSyncError, helpers};
    use crate::sync_force::runner::rate_from_env;

    #[test]
    fn idle_status_is_clean() {
        let status = ForceSyncStatus::idle();
        assert_eq!(status.state, ForceSyncState::Idle);
        assert!(status.started_at.is_none());
        assert!(status.finished_at.is_none());
        assert!(status.errors.is_empty());
    }

    #[test]
    fn idle_status_has_no_finished_at() {
        let status = ForceSyncStatus::idle();
        assert!(status.finished_at.is_none());
    }

    #[test]
    fn running_status_carries_progress_counts() {
        let status = ForceSyncStatus {
            state: ForceSyncState::Running,
            started_at: Some("now".to_string()),
            finished_at: None,
            direction: None,
            total_pairs: 5,
            processed: 2,
            succeeded: 2,
            skipped: 0,
            failed: 0,
            current_user: None,
            last_error: None,
            errors: Vec::new(),
        };
        assert_eq!(status.processed, 2);
        assert_eq!(status.total_pairs, 5);
    }

    #[test]
    fn rate_clamped_to_range() {
        unsafe {
            std::env::set_var("STATESYNC_FORCE_RATE", "100");
        }
        assert_eq!(rate_from_env(), 50);

        unsafe {
            std::env::set_var("STATESYNC_FORCE_RATE", "0");
        }
        assert_eq!(rate_from_env(), 1);

        unsafe {
            std::env::set_var("STATESYNC_FORCE_RATE", "25");
        }
        assert_eq!(rate_from_env(), 25);

        unsafe {
            std::env::remove_var("STATESYNC_FORCE_RATE");
        }
        assert_eq!(rate_from_env(), 5);
    }

    #[test]
    fn cancel_backfill_sets_flag() {
        let tracker = SyncForceTracker::default();
        assert!(!tracker.cancel.load(std::sync::atomic::Ordering::SeqCst));
        tracker.cancel_backfill();
        assert!(tracker.cancel.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[test]
    fn errors_capped_at_limit() {
        let mut status = ForceSyncStatus::idle();
        let mut errors = Vec::new();

        for i in 0..150 {
            helpers::push_error(
                &mut errors,
                &mut status,
                ForceSyncError {
                    user: "u".to_string(),
                    server: "s".to_string(),
                    item_id: None,
                    provider: None,
                    message: format!("err {}", i),
                },
            );
        }

        assert_eq!(errors.len(), 100);
        assert_eq!(errors[0].message, "err 50");
        assert_eq!(errors[99].message, "err 149");
    }
}
