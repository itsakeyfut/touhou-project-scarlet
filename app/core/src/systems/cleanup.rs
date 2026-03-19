//! Session-scope cleanup: despawns all gameplay entities on `OnExit(Playing)`.

use bevy::prelude::*;

use crate::components::GameSessionEntity;

/// Despawns every entity carrying [`GameSessionEntity`] when the game leaves
/// [`crate::states::AppState::Playing`].
///
/// This single system replaces the need for per-system `DespawnOnExit` guards
/// or manual entity tracking. Add [`GameSessionEntity`] to every entity spawned
/// during a gameplay session (player, enemies, bullets, items) and this system
/// will clean them all up atomically when transitioning to a menu, game-over, or
/// stage-clear screen.
///
/// Registered via [`bevy::app::OnExit`]`(`[`crate::states::AppState::Playing`]`)`
/// in [`crate::ScarletCorePlugin`].
pub fn cleanup_game_session(
    mut commands: Commands,
    session_entities: Query<Entity, With<GameSessionEntity>>,
) {
    for entity in &session_entities {
        commands.entity(entity).despawn();
    }
}
