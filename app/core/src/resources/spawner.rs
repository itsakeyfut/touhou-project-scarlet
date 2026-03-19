use crate::components::{BulletEmitter, EnemyKind, EnemyMovement};

// ---------------------------------------------------------------------------
// SpawnEntry
// ---------------------------------------------------------------------------

/// Describes a single enemy appearance in a stage timeline.
///
/// Stage scripts (e.g. `stage1_script`) build a `Vec<SpawnEntry>` and hand it
/// to [`EnemySpawner`]. The spawner system compares each entry's [`time`] to
/// the stage's elapsed time and spawns the enemy when the time has passed.
///
/// [`time`]: SpawnEntry::time
#[derive(Clone)]
pub struct SpawnEntry {
    /// Stage elapsed time (seconds) at which this enemy should be spawned.
    pub time: f32,
    /// Enemy type; determines base HP, collision radius, and score value.
    pub kind: EnemyKind,
    /// World-space spawn position.
    pub position: bevy::math::Vec2,
    /// Movement pattern applied to the enemy on spawn.
    pub movement: EnemyMovement,
    /// Optional bullet emitter; `None` for enemies that do not shoot.
    pub emitter: Option<BulletEmitter>,
}

// ---------------------------------------------------------------------------
// EnemySpawner
// ---------------------------------------------------------------------------

/// Global resource that drives the enemy timeline for the current stage.
///
/// Populated by stage-script functions (e.g. `stage1_script`) and consumed
/// by [`crate::systems::enemy::spawner::enemy_spawner_system`].
///
/// The script must be sorted ascending by [`SpawnEntry::time`] so that the
/// spawner can advance `index` linearly without rescanning the entire list.
///
/// # Usage
///
/// ```rust,ignore
/// // In an OnEnter(AppState::Playing) system:
/// fn load_stage1(mut spawner: ResMut<EnemySpawner>) {
///     spawner.script = stage1_script();
///     spawner.index = 0;
/// }
/// ```
#[derive(bevy::prelude::Resource, Default)]
pub struct EnemySpawner {
    /// Ordered list of spawn events for this stage.
    ///
    /// Must be sorted by [`SpawnEntry::time`] in ascending order.
    pub script: Vec<SpawnEntry>,
    /// Index of the next entry to process.
    ///
    /// Advanced by the spawner system as entries are consumed; never decremented.
    pub index: usize,
}

impl EnemySpawner {
    /// Returns `true` when all entries in the script have been processed.
    pub fn is_finished(&self) -> bool {
        self.index >= self.script.len()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// A freshly-defaulted spawner has an empty script and index 0.
    #[test]
    fn default_spawner_is_empty() {
        let s = EnemySpawner::default();
        assert!(s.script.is_empty());
        assert_eq!(s.index, 0);
        assert!(s.is_finished());
    }

    /// is_finished returns false when index < script.len().
    #[test]
    fn is_finished_false_when_entries_remain() {
        use crate::components::EnemyMovement;
        let mut s = EnemySpawner::default();
        s.script.push(SpawnEntry {
            time: 1.0,
            kind: EnemyKind::Fairy,
            position: bevy::math::Vec2::ZERO,
            movement: EnemyMovement::FallDown { speed: 80.0 },
            emitter: None,
        });
        assert!(!s.is_finished());
        s.index = 1;
        assert!(s.is_finished());
    }
}
