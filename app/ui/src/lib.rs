use bevy::prelude::*;

pub mod camera;

/// UI plugin.
///
/// Manages camera setup, all game screens (title, character select,
/// difficulty select, loading, pause, game over, stage clear, ending),
/// and the in-game HUD.
pub struct ScarletUiPlugin;

impl Plugin for ScarletUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, camera::setup_camera);

        // TODO(phase-11): add screen systems
        // TODO(phase-12): add HUD systems
    }
}
