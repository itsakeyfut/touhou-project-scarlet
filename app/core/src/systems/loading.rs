//! Loading-state systems: config readiness gate.

use bevy::prelude::*;

use crate::{
    config::{
        EnemyBulletConfigParams, FodderEnemyConfigParams, GameRulesConfigParams,
        PlayerBulletConfigParams, PlayerConfigParams,
    },
    states::AppState,
};

/// Interval in seconds between "still waiting" warn! messages.
const WARN_INTERVAL_SECS: f32 = 5.0;

/// Blocks the [`AppState::Loading`] → [`AppState::Playing`] transition until
/// every RON config file has been fully loaded by the asset server.
///
/// Runs every frame during [`AppState::Loading`]. Once all five
/// [`SystemParam`] getters return `Some`, the state is advanced to
/// [`AppState::Playing`], triggering [`crate::systems::player::spawn_player`]
/// and the stage-1 script loader.
///
/// # Stuck-loading detection
///
/// If loading takes longer than [`WARN_INTERVAL_SECS`] a `warn!` message is
/// emitted listing which configs are still missing. This repeats every
/// [`WARN_INTERVAL_SECS`] until all configs are ready, making it easy to
/// diagnose missing or malformed RON files during development.
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
    time: Res<Time>,
    mut elapsed: Local<f32>,
) {
    if player_cfg.get().is_some()
        && game_rules_cfg.get().is_some()
        && fodder_cfg.get().is_some()
        && player_bullet_cfg.get().is_some()
        && enemy_bullet_cfg.get().is_some()
    {
        info!("All configs loaded — advancing to Playing");
        next_state.set(AppState::Playing);
        return;
    }

    *elapsed += time.delta_secs();
    if *elapsed >= WARN_INTERVAL_SECS {
        warn!(
            "Still waiting for configs after {:.1}s — \
             player={} game_rules={} fodder={} player_bullet={} enemy_bullet={}. \
             Check that all RON files exist and are valid.",
            *elapsed,
            player_cfg.get().is_some(),
            game_rules_cfg.get().is_some(),
            fodder_cfg.get().is_some(),
            player_bullet_cfg.get().is_some(),
            enemy_bullet_cfg.get().is_some(),
        );
        *elapsed = 0.0;
    }
}
