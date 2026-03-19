use bevy::prelude::*;

use crate::{
    components::{bullet::BulletPattern, enemy::Enemy},
    config::FodderEnemyConfigParams,
    resources::{EnemySpawner, SpawnEntry, StageData},
    systems::danmaku::emitter::SpiralState,
};

/// Spawns enemies from the [`EnemySpawner`] script when their scheduled time arrives.
///
/// Each frame this system walks the [`EnemySpawner::script`] starting at
/// [`EnemySpawner::index`] and spawns every [`SpawnEntry`] whose
/// [`SpawnEntry::time`] is ≤ [`StageData::elapsed_time`]. The index is
/// advanced past each processed entry; entries are never re-processed.
///
/// # Spawned components
///
/// Every enemy entity receives:
/// - [`Enemy`] — HP, collision radius, and score value derived from [`EnemyKind`].
/// - [`EnemyKind`] — stored as a component for queries and drop-table logic.
/// - [`EnemyMovement`] — cloned from the script entry.
/// - [`Sprite`] — placeholder coloured square sized to the collision diameter.
/// - [`Transform`] — positioned at [`SpawnEntry::position`], Z = 1.0.
///
/// Optionally:
/// - [`BulletEmitter`] — if [`SpawnEntry::emitter`] is `Some`.
/// - [`SpiralState`] — additionally inserted when the emitter pattern is
///   [`BulletPattern::Spiral`], as required by
///   [`crate::systems::danmaku::emitter::update_spiral_emitters`].
///
/// # Ordering
///
/// Must run **after** [`crate::systems::stage::stage_control_system`] so that
/// [`StageData::elapsed_time`] has already been updated for the current frame.
/// Both systems are in [`crate::GameSystemSet::StageControl`]; use `.chain()`
/// when registering them (see [`crate::ScarletCorePlugin`]).
///
/// Registered in [`crate::GameSystemSet::StageControl`].
pub fn enemy_spawner_system(
    mut commands: Commands,
    mut spawner: ResMut<EnemySpawner>,
    stage_data: Res<StageData>,
    fodder_cfg: FodderEnemyConfigParams,
) {
    let elapsed = stage_data.elapsed_time;

    while let Some(entry) = spawner.script.get(spawner.index) {
        if entry.time > elapsed {
            break;
        }

        spawn_enemy(&mut commands, entry, &fodder_cfg);
        spawner.index += 1;
    }
}

/// Spawns a single enemy entity from a [`SpawnEntry`].
fn spawn_enemy(commands: &mut Commands, entry: &SpawnEntry, fodder_cfg: &FodderEnemyConfigParams) {
    let kind = entry.kind;
    let hp = fodder_cfg.hp_for(kind);
    let radius = fodder_cfg.radius_for(kind);
    let score = fodder_cfg.score_for(kind);

    let mut entity = commands.spawn((
        Enemy {
            hp,
            hp_max: hp,
            collision_radius: radius,
            score_value: score,
            is_boss: false,
        },
        kind,
        entry.movement.clone(),
        Sprite {
            color: kind.color(),
            custom_size: Some(Vec2::splat(radius * 2.0)),
            ..default()
        },
        Transform::from_translation(entry.position.extend(1.0)),
    ));

    if let Some(emitter) = entry.emitter.clone() {
        let is_spiral = matches!(emitter.pattern, BulletPattern::Spiral { .. });
        entity.insert(emitter);
        if is_spiral {
            entity.insert(SpiralState::default());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        components::{EnemyKind, EnemyMovement},
        resources::SpawnEntry,
    };

    fn make_entry(time: f32) -> SpawnEntry {
        SpawnEntry {
            time,
            kind: EnemyKind::Fairy,
            position: Vec2::ZERO,
            movement: EnemyMovement::FallDown { speed: 80.0 },
            emitter: None,
        }
    }

    /// Spawner with no entries is already finished.
    #[test]
    fn empty_script_is_finished() {
        let s = EnemySpawner::default();
        assert!(s.is_finished());
    }

    /// Index advances only past entries whose time has been reached.
    #[test]
    fn index_advances_up_to_elapsed_time() {
        let mut spawner = EnemySpawner {
            script: vec![make_entry(1.0), make_entry(2.0), make_entry(5.0)],
            index: 0,
        };
        let elapsed = 2.5_f32;

        // Simulate the while loop from enemy_spawner_system.
        while let Some(entry) = spawner.script.get(spawner.index) {
            if entry.time > elapsed {
                break;
            }
            spawner.index += 1;
        }

        // Entries at t=1.0 and t=2.0 should have been processed; t=5.0 not yet.
        assert_eq!(spawner.index, 2);
        assert!(!spawner.is_finished());
    }

    /// After all entries are processed the spawner is finished.
    #[test]
    fn all_entries_processed_when_elapsed_exceeds_all() {
        let mut spawner = EnemySpawner {
            script: vec![make_entry(0.5), make_entry(1.0)],
            index: 0,
        };
        let elapsed = 99.0_f32;

        while let Some(entry) = spawner.script.get(spawner.index) {
            if entry.time > elapsed {
                break;
            }
            spawner.index += 1;
        }

        assert!(spawner.is_finished());
    }

    /// An entry whose time equals elapsed_time must be spawned (boundary case).
    #[test]
    fn entry_at_exact_elapsed_time_is_spawned() {
        let mut spawner = EnemySpawner {
            script: vec![make_entry(3.0)],
            index: 0,
        };
        let elapsed = 3.0_f32;

        while let Some(entry) = spawner.script.get(spawner.index) {
            if entry.time > elapsed {
                break;
            }
            spawner.index += 1;
        }

        assert_eq!(spawner.index, 1);
    }
}
