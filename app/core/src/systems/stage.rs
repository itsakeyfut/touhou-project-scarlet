use bevy::prelude::*;

use crate::{
    components::enemy::Enemy,
    events::BossSpawnEvent,
    resources::{EnemySpawner, StageData},
    states::AppState,
};

/// Advances the stage timeline and manages boss-spawn and stage-clear transitions.
///
/// Every frame this system:
///
/// 1. Increments [`StageData::elapsed_time`] by `delta_secs`.
/// 2. Checks whether the spawner script is exhausted **and** no non-boss enemies
///    remain on screen. When both conditions hold for the first time,
///    [`StageData::boss_active`] is set to `true` and a [`BossSpawnEvent`] is
///    emitted so Phase 08 systems can spawn the stage boss.
/// 3. When the boss has been activated and [`StageData::boss_defeated`] is
///    `true`, transitions to [`AppState::StageClear`].
///
/// Registered in [`crate::GameSystemSet::StageControl`].
pub fn stage_control_system(
    mut stage_data: ResMut<StageData>,
    spawner: Res<EnemySpawner>,
    enemies: Query<&Enemy>,
    mut boss_spawn_events: MessageWriter<BossSpawnEvent>,
    mut next_state: ResMut<NextState<AppState>>,
    time: Res<Time>,
) {
    // 1. Advance elapsed time.
    stage_data.elapsed_time += time.delta_secs();

    // 2. Boss-spawn trigger: script exhausted + no active non-boss enemies.
    if !stage_data.boss_active {
        let script_done = spawner.is_finished();
        let no_normal_enemies = enemies.iter().all(|e| e.is_boss);

        if script_done && no_normal_enemies {
            stage_data.boss_active = true;
            boss_spawn_events.write(BossSpawnEvent {
                stage_number: stage_data.stage_number,
            });
        }
    }

    // 3. Stage-clear trigger: boss has been activated and then defeated.
    if stage_data.boss_active && stage_data.boss_defeated {
        next_state.set(AppState::StageClear);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh StageData has elapsed_time == 0.
    #[test]
    fn stage_data_starts_at_zero() {
        let sd = StageData::default();
        assert_eq!(sd.elapsed_time, 0.0);
        assert!(!sd.boss_active);
        assert!(!sd.boss_defeated);
    }

    /// boss_active flag must be set independently of boss_defeated.
    #[test]
    fn boss_active_and_defeated_are_independent() {
        let mut sd = StageData::default();
        sd.boss_active = true;
        assert!(!sd.boss_defeated);
        sd.boss_defeated = true;
        assert!(sd.boss_active && sd.boss_defeated);
    }
}
