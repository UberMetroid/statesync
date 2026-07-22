//! Property + boundary tests for force skip-if-equal (played / position).

use proptest::prelude::*;
use statesync::client::types::UserDataEntry;
use statesync::sync_force::force_equal::played_state_already_equal;

const POS_EQ_TICKS: i64 = 50_000_000; // 5s in ticks (must match implementation)

fn entry(played: bool, pos: Option<i64>) -> UserDataEntry {
    UserDataEntry {
        item_id: "x".into(),
        played,
        playback_position_ticks: pos,
        is_favorite: None,
    }
}

// --- Unit: positive / negative / boundary ---

#[test]
fn equal_when_both_played_and_positions_within_window() {
    let tgt = entry(true, Some(100));
    assert!(played_state_already_equal(true, true, 100, &tgt));
    assert!(played_state_already_equal(
        true,
        true,
        100 + POS_EQ_TICKS,
        &tgt
    ));
}

#[test]
fn not_equal_when_target_unplayed_and_force_played() {
    let tgt = entry(false, Some(0));
    assert!(!played_state_already_equal(true, false, 0, &tgt));
}

#[test]
fn equal_when_force_played_off_even_if_target_unplayed() {
    let tgt = entry(false, Some(0));
    assert!(played_state_already_equal(false, false, 0, &tgt));
}

#[test]
fn position_boundary_just_outside_window_needs_write() {
    let tgt = entry(true, Some(0));
    assert!(!played_state_already_equal(
        false,
        true,
        POS_EQ_TICKS + 1,
        &tgt
    ));
}

#[test]
fn both_zero_positions_considered_equal() {
    let tgt = entry(true, Some(0));
    assert!(played_state_already_equal(true, true, 0, &tgt));
    let tgt_none = entry(true, None);
    assert!(played_state_already_equal(true, true, 0, &tgt_none));
}

// --- Property ---

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn within_window_and_played_is_equal(
        base in 0i64..1_000_000_000,
        delta in 0i64..=POS_EQ_TICKS,
    ) {
        let tgt = entry(true, Some(base));
        let src = base.saturating_add(delta);
        prop_assert!(played_state_already_equal(true, true, src, &tgt));
    }

    #[test]
    fn outside_window_needs_write_when_nonzero(
        base in 1i64..500_000_000,
    ) {
        let tgt = entry(true, Some(base));
        let src = base.saturating_add(POS_EQ_TICKS + 1);
        prop_assert!(!played_state_already_equal(false, true, src, &tgt));
    }

    #[test]
    fn unplayed_target_never_equal_if_force_played(
        pos in 0i64..100_000_000,
        src in 0i64..100_000_000,
    ) {
        let tgt = entry(false, Some(pos));
        prop_assert!(!played_state_already_equal(true, false, src, &tgt));
    }
}
