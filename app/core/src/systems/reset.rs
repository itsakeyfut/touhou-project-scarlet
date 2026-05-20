//! Per-run resource reset: restores gameplay resources to initial values on
//! `OnEnter(AppState::Playing)`.

use bevy::prelude::*;

use crate::resources::{BombState, EnemySpawner, FragmentTracker, GameData, StageData};

/// Resets all per-run gameplay resources to their starting values.
///
/// Called on [`bevy::app::OnEnter`]`(`[`crate::states::AppState::Playing`]`)`
/// **before** [`crate::stages::setup_stage`] so that the spawner script is
/// fresh when the stage loader populates it.
///
/// Resources reset:
///
/// | Resource | Reset to |
/// |---|---|
/// | [`GameData`] | [`GameData::new_game`] (score 0, 2 lives, 3 bombs, power 0); `hi_score` preserved |
/// | [`EnemySpawner`] | [`EnemySpawner::default`] (empty script, index 0) |
/// | [`StageData`] | [`StageData::default`] (stage 1, elapsed 0 s); `stage_number` preserved |
/// | [`FragmentTracker`] | [`FragmentTracker::default`] (fragments 0) |
/// | [`BombState`] | [`BombState::default`] (inactive, timers elapsed) |
///
/// `hi_score` is carried over so the session-best score survives GameOver →
/// restart. `stage_number` is carried over so stage transitions from
/// [`crate::systems::stage_clear::on_stage_clear`] are not undone on
/// re-entering `Playing`.
pub fn reset_per_run_resources(
    mut game_data: ResMut<GameData>,
    mut spawner: ResMut<EnemySpawner>,
    mut stage_data: ResMut<StageData>,
    mut tracker: ResMut<FragmentTracker>,
    mut bomb_state: ResMut<BombState>,
) {
    let hi_score = game_data.hi_score;
    let stage_number = stage_data.stage_number;
    *game_data = GameData::new_game();
    game_data.hi_score = hi_score;
    *spawner = EnemySpawner::default();
    *stage_data = StageData::default();
    stage_data.stage_number = stage_number;
    *tracker = FragmentTracker::default();
    *bomb_state = BombState::default();
}
