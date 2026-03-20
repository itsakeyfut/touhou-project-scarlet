//! Per-run resource reset: restores gameplay resources to initial values on
//! `OnEnter(AppState::Playing)`.

use bevy::prelude::*;

use crate::resources::{EnemySpawner, FragmentTracker, GameData, StageData};

/// Resets all per-run gameplay resources to their starting values.
///
/// Called on [`bevy::app::OnEnter`]`(`[`crate::states::AppState::Playing`]`)`
/// **before** `load_stage1_system` so that the spawner script is fresh when
/// the stage loader populates it.
///
/// Resources reset:
///
/// | Resource | Reset to |
/// |---|---|
/// | [`GameData`] | [`GameData::new_game`] (score 0, 2 lives, 3 bombs, power 0) |
/// | [`EnemySpawner`] | [`EnemySpawner::default`] (empty script, index 0) |
/// | [`StageData`] | [`StageData::default`] (stage 1, elapsed 0 s) |
/// | [`FragmentTracker`] | [`FragmentTracker::default`] (fragments 0) |
///
/// Note: `hi_score` inside [`GameData`] is intentionally reset to `0` for now.
/// Persistent high-score saving will be addressed in a future issue once a
/// save/load system is in place; `GameData::new_game` will then preserve it.
pub fn reset_per_run_resources(
    mut game_data: ResMut<GameData>,
    mut spawner: ResMut<EnemySpawner>,
    mut stage_data: ResMut<StageData>,
    mut tracker: ResMut<FragmentTracker>,
) {
    *game_data = GameData::new_game();
    *spawner = EnemySpawner::default();
    *stage_data = StageData::default();
    *tracker = FragmentTracker::default();
}
