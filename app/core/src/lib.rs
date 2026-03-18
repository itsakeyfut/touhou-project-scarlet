use bevy::prelude::*;

pub mod components;
pub mod constants;
pub mod debug;
pub mod events;
pub mod game_set;
pub mod resources;
pub mod shaders;
pub mod states;
pub mod systems;

pub use components::{
    BulletEmitter, BulletPattern, BulletTrail, BulletVelocity, DespawnOutOfBounds, EnemyBullet,
    EnemyBulletKind, InvincibilityTimer, Player, PlayerBullet, PlayerStats, ShootTimer,
};
pub use constants::{PLAY_AREA_HALF_H, PLAY_AREA_HALF_W, PLAY_AREA_HEIGHT, PLAY_AREA_WIDTH};
pub use events::ShootEvent;
pub use game_set::GameSystemSet;
pub use resources::GameData;
pub use shaders::ScarletShadersPlugin;
pub use states::AppState;

/// Core game plugin.
///
/// Registers the [`AppState`] state machine, [`GameSystemSet`] ordering,
/// and all game-logic systems. Subsequent phases will add more systems here.
pub struct ScarletCorePlugin;

impl Plugin for ScarletCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(shaders::ScarletShadersPlugin);

        app.init_state::<AppState>();

        // Events.
        app.add_message::<ShootEvent>();

        // Resources — inserted with game-start values.
        app.insert_resource(GameData::new_game());

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
                (
                    systems::player::player_movement_system,
                    systems::player::shoot_input_system,
                )
                    .in_set(GameSystemSet::Input),
            );

        // Bullet systems.
        app.add_systems(
            Update,
            systems::bullet::bullet_spawn_system.in_set(GameSystemSet::BulletEmit),
        )
        .add_systems(
            Update,
            systems::bullet::bullet_movement_system.in_set(GameSystemSet::Movement),
        )
        .add_systems(
            Update,
            systems::bullet::despawn_out_of_bounds_system.in_set(GameSystemSet::Cleanup),
        );

        // Danmaku emitter systems.
        app.add_systems(
            Update,
            (
                systems::danmaku::emitter::bullet_emitter_system,
                systems::danmaku::emitter::update_spiral_emitters,
            )
                .in_set(GameSystemSet::BulletEmit),
        );

        #[cfg(feature = "debug-hitbox")]
        app.add_systems(Startup, debug::debug_skip_to_playing)
            .add_systems(OnEnter(AppState::Playing), debug::spawn_debug_enemies)
            .add_systems(
                Update,
                (debug::debug_play_area_system, debug::debug_bullet_hitbox),
            );
    }
}
