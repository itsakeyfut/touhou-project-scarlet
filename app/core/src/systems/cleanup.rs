//! Session-scope cleanup: despawns all gameplay entities on `OnExit(Playing)`.

use bevy::prelude::*;

use crate::{components::GameSessionEntity, states::AppState};

/// Despawns every entity carrying [`GameSessionEntity`] when the game leaves
/// [`AppState::Playing`] for a terminal transition (e.g. `GameOver`, `StageClear`).
///
/// **Pause guard**: when transitioning to [`AppState::Paused`] the session entities
/// are deliberately preserved so gameplay can resume seamlessly. The system reads
/// [`NextState<AppState>`] — which still holds the pending target during
/// `OnExit(Playing)` — and returns early without despawning.
///
/// This single system replaces the need for per-system `DespawnOnExit` guards or
/// manual entity tracking. Add [`GameSessionEntity`] to every entity spawned
/// during a gameplay session (player, enemies, bullets, items) and this system
/// will clean them all up atomically on any non-pause exit from `Playing`.
///
/// Registered via [`bevy::app::OnExit`]`(`[`AppState::Playing`]`)`
/// in [`crate::ScarletCorePlugin`].
pub fn cleanup_game_session(
    mut commands: Commands,
    session_entities: Query<Entity, With<GameSessionEntity>>,
    next_state: Res<NextState<AppState>>,
) {
    // Preserve entities when pausing — the player will resume from the same world.
    if matches!(next_state.as_ref(), NextState::Pending(AppState::Paused)) {
        return;
    }

    for entity in &session_entities {
        commands.entity(entity).despawn();
    }
}
