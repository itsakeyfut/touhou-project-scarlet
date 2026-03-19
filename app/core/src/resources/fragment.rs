use bevy::prelude::*;

/// Number of [`crate::components::ItemKind::LifeFragment`] items needed to
/// earn one extra life.
pub const LIFE_EXTEND_FRAGMENTS: u8 = 5;

/// Number of [`crate::components::ItemKind::BombFragment`] items needed to
/// earn one extra bomb stock.
pub const BOMB_EXTEND_FRAGMENTS: u8 = 5;

/// Tracks collected life and bomb fragments across the current run.
///
/// Inserted as a [`Resource`] at game-start by [`crate::ScarletCorePlugin`].
/// Updated by [`crate::systems::item::item_collection_system`] when the
/// player picks up [`crate::components::ItemKind::LifeFragment`] or
/// [`crate::components::ItemKind::BombFragment`] items.
///
/// When a fragment counter reaches its threshold constant
/// ([`LIFE_EXTEND_FRAGMENTS`] / [`BOMB_EXTEND_FRAGMENTS`]) the counter is
/// reset to zero and the corresponding stock (`lives` / `bombs`) is incremented
/// in [`crate::resources::GameData`].
#[derive(Resource, Default, Debug, PartialEq, Eq)]
pub struct FragmentTracker {
    /// Collected life fragments since the last extend (resets at threshold).
    pub life_fragments: u8,
    /// Collected bomb fragments since the last extend (resets at threshold).
    pub bomb_fragments: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_fragment_tracker_is_zero() {
        let tracker = FragmentTracker::default();
        assert_eq!(tracker.life_fragments, 0);
        assert_eq!(tracker.bomb_fragments, 0);
    }

    #[test]
    fn life_extend_threshold_is_five() {
        assert_eq!(LIFE_EXTEND_FRAGMENTS, 5);
    }

    #[test]
    fn bomb_extend_threshold_is_five() {
        assert_eq!(BOMB_EXTEND_FRAGMENTS, 5);
    }

    /// Simulates collecting fragments one at a time and verifies the extend
    /// logic that the item system will apply.
    #[test]
    fn five_life_fragments_triggers_extend() {
        let mut tracker = FragmentTracker::default();
        let mut lives: u8 = 2;

        for _ in 0..LIFE_EXTEND_FRAGMENTS {
            tracker.life_fragments += 1;
            if tracker.life_fragments >= LIFE_EXTEND_FRAGMENTS {
                tracker.life_fragments = 0;
                lives += 1;
            }
        }

        assert_eq!(tracker.life_fragments, 0, "counter must reset after extend");
        assert_eq!(lives, 3, "player must gain one life");
    }

    #[test]
    fn four_life_fragments_do_not_trigger_extend() {
        let mut tracker = FragmentTracker::default();
        let mut lives: u8 = 2;

        for _ in 0..(LIFE_EXTEND_FRAGMENTS - 1) {
            tracker.life_fragments += 1;
            if tracker.life_fragments >= LIFE_EXTEND_FRAGMENTS {
                tracker.life_fragments = 0;
                lives += 1;
            }
        }

        assert_eq!(lives, 2, "no extend before reaching threshold");
        assert_eq!(tracker.life_fragments, LIFE_EXTEND_FRAGMENTS - 1);
    }
}
