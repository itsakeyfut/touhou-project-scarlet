use bevy::prelude::*;

pub mod constants;
pub mod debug;
pub mod states;

pub use constants::{PLAY_AREA_HALF_H, PLAY_AREA_HALF_W, PLAY_AREA_HEIGHT, PLAY_AREA_WIDTH};
pub use states::AppState;

/// Core game plugin.
///
/// Registers the [`AppState`] state machine. Subsequent phases will add
/// resources, events, system sets, and game-logic systems here.
pub struct ScarletCorePlugin;

impl Plugin for ScarletCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>();

        #[cfg(feature = "debug-hitbox")]
        app.add_systems(Update, debug::debug_play_area_system);
    }
}
