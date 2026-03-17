use bevy::prelude::*;

/// UI plugin.
///
/// Will manage camera setup, all game screens (title, character select,
/// difficulty select, loading, pause, game over, stage clear, ending),
/// and the in-game HUD. Systems are added in Phase 11–12.
pub struct ScarletUiPlugin;

impl Plugin for ScarletUiPlugin {
    fn build(&self, _app: &mut App) {
        // TODO(phase-11): add screen systems
        // TODO(phase-12): add HUD systems
    }
}
