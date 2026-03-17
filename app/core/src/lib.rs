use bevy::prelude::*;

pub mod components;
pub mod constants;
pub mod debug;
pub mod game_set;
pub mod states;
pub mod systems;

pub use components::{InvincibilityTimer, Player, PlayerStats};
pub use constants::{PLAY_AREA_HALF_H, PLAY_AREA_HALF_W, PLAY_AREA_HEIGHT, PLAY_AREA_WIDTH};
pub use game_set::GameSystemSet;
pub use states::AppState;

/// Core game plugin.
///
/// Registers the [`AppState`] state machine, [`GameSystemSet`] ordering,
/// and all game-logic systems. Subsequent phases will add more systems here.
pub struct ScarletCorePlugin;

impl Plugin for ScarletCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>();

        // System set ordering — all sets run only while Playing.
        app.configure_sets(
            Update,
            (
                GameSystemSet::Input,
                GameSystemSet::PlayerLogic,
                GameSystemSet::BulletEmit,
                GameSystemSet::Movement,
                GameSystemSet::Collision,
                GameSystemSet::GameLogic,
                GameSystemSet::StageControl,
                GameSystemSet::Effects,
                GameSystemSet::Cleanup,
            )
                .chain()
                .run_if(in_state(AppState::Playing)),
        );

        // Player systems.
        app.add_systems(OnEnter(AppState::Playing), systems::player::spawn_player)
            .add_systems(
                Update,
                systems::player::player_movement_system.in_set(GameSystemSet::Input),
            );

        #[cfg(feature = "debug-hitbox")]
        app.add_systems(Startup, debug::debug_skip_to_playing)
            .add_systems(Update, debug::debug_play_area_system);
    }
}
