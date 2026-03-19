use bevy::prelude::*;

use crate::{
    config::GameRulesConfigParams,
    events::{ExtendEvent, ExtendKind},
    resources::{FragmentTracker, GameData},
};

/// Checks whether any fragment counter has reached its extend threshold and,
/// if so, awards the corresponding stock and emits an [`ExtendEvent`].
///
/// This system is the single source of truth for fragment-based extends.
/// Fragment counters are incremented by
/// [`crate::systems::item::item_collection_system`] (via `apply_item`); this
/// system runs afterward in the same [`crate::GameSystemSet::GameLogic`] set
/// and performs the extend check each frame.
///
/// # Extend rules
///
/// Thresholds and caps are read from [`crate::config::GameRulesConfig`]
/// (loaded from `assets/config/game_rules.ron`).  Default values:
///
/// | Counter | Threshold | Effect |
/// |---|---|---|
/// | `life_fragments` | 5 | `lives += 1`, counter reset to 0 |
/// | `bomb_fragments` | 5 | `bombs = (bombs+1).min(max_bombs)`, counter reset to 0 |
///
/// Multiple extends in a single frame (e.g., collecting the 5th life fragment
/// and the 5th bomb fragment simultaneously) each emit a separate
/// [`ExtendEvent`].
///
/// Registered in [`crate::GameSystemSet::GameLogic`].
pub fn check_extend_system(
    mut tracker: ResMut<FragmentTracker>,
    mut game_data: ResMut<GameData>,
    mut extend_events: MessageWriter<ExtendEvent>,
    rules: GameRulesConfigParams,
) {
    let cfg = rules.get_or_default();
    let life_threshold = cfg.life_extend_fragments;
    let bomb_threshold = cfg.bomb_extend_fragments;
    let max_bombs = cfg.max_bombs;

    if tracker.life_fragments >= life_threshold {
        tracker.life_fragments = 0;
        game_data.lives = game_data.lives.saturating_add(1);
        extend_events.write(ExtendEvent {
            kind: ExtendKind::Life,
        });
    }

    if tracker.bomb_fragments >= bomb_threshold {
        tracker.bomb_fragments = 0;
        game_data.bombs = game_data.bombs.saturating_add(1).min(max_bombs);
        extend_events.write(ExtendEvent {
            kind: ExtendKind::Bomb,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::game_rules::{
        DEFAULT_BOMB_EXTEND_FRAGMENTS, DEFAULT_LIFE_EXTEND_FRAGMENTS, DEFAULT_MAX_BOMBS,
    };

    fn make_resources() -> (FragmentTracker, GameData) {
        (
            FragmentTracker::default(),
            GameData {
                score: 0,
                hi_score: 0,
                lives: 2,
                bombs: 3,
                power: 0,
                graze: 0,
            },
        )
    }

    /// Helper that runs the extend logic inline (without a Bevy App),
    /// using the same default thresholds as `check_extend_system`.
    fn run_extend(tracker: &mut FragmentTracker, game_data: &mut GameData) -> Vec<ExtendKind> {
        let mut events = Vec::new();
        if tracker.life_fragments >= DEFAULT_LIFE_EXTEND_FRAGMENTS {
            tracker.life_fragments = 0;
            game_data.lives = game_data.lives.saturating_add(1);
            events.push(ExtendKind::Life);
        }
        if tracker.bomb_fragments >= DEFAULT_BOMB_EXTEND_FRAGMENTS {
            tracker.bomb_fragments = 0;
            game_data.bombs = game_data.bombs.saturating_add(1).min(DEFAULT_MAX_BOMBS);
            events.push(ExtendKind::Bomb);
        }
        events
    }

    /// Below threshold — no extend, no counter reset.
    #[test]
    fn no_extend_below_threshold() {
        let (mut tracker, mut gd) = make_resources();
        tracker.life_fragments = DEFAULT_LIFE_EXTEND_FRAGMENTS - 1;
        let events = run_extend(&mut tracker, &mut gd);
        assert!(events.is_empty());
        assert_eq!(gd.lives, 2);
        assert_eq!(tracker.life_fragments, DEFAULT_LIFE_EXTEND_FRAGMENTS - 1);
    }

    /// Exactly at threshold — one life extend, counter reset.
    #[test]
    fn life_extend_at_threshold() {
        let (mut tracker, mut gd) = make_resources();
        tracker.life_fragments = DEFAULT_LIFE_EXTEND_FRAGMENTS;
        let events = run_extend(&mut tracker, &mut gd);
        assert_eq!(events, [ExtendKind::Life]);
        assert_eq!(gd.lives, 3);
        assert_eq!(tracker.life_fragments, 0);
    }

    /// Exactly at threshold — one bomb extend, counter reset.
    #[test]
    fn bomb_extend_at_threshold() {
        let (mut tracker, mut gd) = make_resources();
        gd.bombs = 2;
        tracker.bomb_fragments = DEFAULT_BOMB_EXTEND_FRAGMENTS;
        let events = run_extend(&mut tracker, &mut gd);
        assert_eq!(events, [ExtendKind::Bomb]);
        assert_eq!(gd.bombs, 3);
        assert_eq!(tracker.bomb_fragments, 0);
    }

    /// Both counters at threshold simultaneously — two events emitted.
    #[test]
    fn both_extend_simultaneously() {
        let (mut tracker, mut gd) = make_resources();
        gd.bombs = 2;
        tracker.life_fragments = DEFAULT_LIFE_EXTEND_FRAGMENTS;
        tracker.bomb_fragments = DEFAULT_BOMB_EXTEND_FRAGMENTS;
        let events = run_extend(&mut tracker, &mut gd);
        assert_eq!(events.len(), 2);
        assert!(events.contains(&ExtendKind::Life));
        assert!(events.contains(&ExtendKind::Bomb));
        assert_eq!(tracker.life_fragments, 0);
        assert_eq!(tracker.bomb_fragments, 0);
    }

    /// Bomb count must not exceed max_bombs even when already at maximum.
    #[test]
    fn bomb_extend_caps_at_3() {
        let (mut tracker, mut gd) = make_resources();
        gd.bombs = DEFAULT_MAX_BOMBS; // already at maximum
        tracker.bomb_fragments = DEFAULT_BOMB_EXTEND_FRAGMENTS;
        let events = run_extend(&mut tracker, &mut gd);
        assert_eq!(events, [ExtendKind::Bomb]);
        assert_eq!(gd.bombs, DEFAULT_MAX_BOMBS, "bombs must not exceed max");
        assert_eq!(tracker.bomb_fragments, 0, "counter must reset even at cap");
    }
}
