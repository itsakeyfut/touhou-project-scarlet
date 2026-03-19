//! Loading-state systems: config readiness gate.

use bevy::prelude::*;

use crate::{
    config::{
        EnemyBulletConfigParams, FodderEnemyConfigParams, GameRulesConfigParams,
        PlayerBulletConfigParams, PlayerConfigParams,
    },
    states::AppState,
};

/// Blocks the [`AppState::Loading`] → [`AppState::Playing`] transition until
/// every RON config file has been fully loaded by the asset server.
///
/// Runs every frame during [`AppState::Loading`]. Once all five
/// [`SystemParam`] getters return `Some`, the state is advanced to
/// [`AppState::Playing`], triggering [`crate::systems::player::spawn_player`]
/// and the stage-1 script loader.
///
/// # Why this is necessary
///
/// RON configs are loaded asynchronously. Without this guard the game would
/// start with every `XxxConfigParams::get()` returning `None` for the first
/// few frames, silently using `DEFAULT_*` fallback values even when the RON
/// file contains different values.
///
/// Registered in [`crate::ScarletCorePlugin`] via
/// `Update, run_if(in_state(AppState::Loading))`.
pub fn wait_for_configs(
    player_cfg: PlayerConfigParams,
    game_rules_cfg: GameRulesConfigParams,
    fodder_cfg: FodderEnemyConfigParams,
    player_bullet_cfg: PlayerBulletConfigParams,
    enemy_bullet_cfg: EnemyBulletConfigParams,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if player_cfg.get().is_some()
        && game_rules_cfg.get().is_some()
        && fodder_cfg.get().is_some()
        && player_bullet_cfg.get().is_some()
        && enemy_bullet_cfg.get().is_some()
    {
        info!("All configs loaded — advancing to Playing");
        next_state.set(AppState::Playing);
    }
}
