pub mod stage1;

use bevy::prelude::*;

use crate::resources::{Difficulty, EnemySpawner, StageData};

/// Loads the enemy script for the current stage and difficulty into [`EnemySpawner`].
///
/// Runs on [`OnEnter`]`(`[`crate::states::AppState::Playing`]`)` **after**
/// [`crate::systems::reset::reset_per_run_resources`] so the spawner is cleared
/// before the new script is written.
///
/// Stage-specific per-frame timing is reset here (`elapsed_time = 0`,
/// `boss_active = false`, `boss_defeated = false`) so the stage starts cleanly
/// regardless of what state the resources were in.
///
/// # Stage routing
///
/// | `stage_number` | Script loaded |
/// |---|---|
/// | 1 | [`stage1::stage1_script`] |
/// | 2–6 | empty (placeholder until implemented) |
/// | other | empty + `warn!` |
pub fn setup_stage(
    mut spawner: ResMut<EnemySpawner>,
    mut stage_data: ResMut<StageData>,
    difficulty: Res<Difficulty>,
) {
    stage_data.elapsed_time = 0.0;
    stage_data.boss_active = false;
    stage_data.boss_defeated = false;

    spawner.script = match stage_data.stage_number {
        1 => stage1::stage1_script(),
        2..=6 => vec![],
        n => {
            warn!("setup_stage: unknown stage {n}, defaulting to empty script");
            vec![]
        }
    };
    spawner.index = 0;

    info!(
        stage = stage_data.stage_number,
        difficulty = difficulty.label(),
        entries = spawner.script.len(),
        "Stage script loaded"
    );
}
