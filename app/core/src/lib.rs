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
    BulletEmitter, BulletPattern, BulletTrail, BulletVelocity, DespawnOutOfBounds, Enemy,
    EnemyBullet, EnemyBulletKind, GrazeVisual, InvincibilityTimer, ItemKind, ItemPhysics, Player,
    PlayerBullet, PlayerStats, ShootTimer,
};
pub use constants::{PLAY_AREA_HALF_H, PLAY_AREA_HALF_W, PLAY_AREA_HEIGHT, PLAY_AREA_WIDTH};
pub use events::{EnemyDefeatedEvent, GrazeEvent, PlayerHitEvent, ShootEvent};
pub use game_set::GameSystemSet;
pub use resources::{BOMB_EXTEND_FRAGMENTS, FragmentTracker, GameData, LIFE_EXTEND_FRAGMENTS};
pub use shaders::{GrazeMaterial, ScarletShadersPlugin};
pub use states::AppState;
pub use systems::collision::check_circle_collision;
pub use systems::item::calc_point_item_value;

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
        app.add_message::<PlayerHitEvent>();
        app.add_message::<GrazeEvent>();
        app.add_message::<EnemyDefeatedEvent>();

        // Resources — inserted with game-start values.
        app.insert_resource(GameData::new_game());
        app.insert_resource(FragmentTracker::default());

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

        // Collision systems.
        app.add_systems(
            Update,
            (
                systems::collision::player_bullet_hit_enemy,
                systems::collision::player_hit_detection,
                systems::collision::graze_detection_system,
            )
                .in_set(GameSystemSet::Collision),
        );

        // GameLogic systems — run after Collision so events are visible.
        app.add_systems(
            Update,
            (
                systems::collision::handle_player_hit,
                systems::item::on_enemy_defeated,
            )
                .in_set(GameSystemSet::GameLogic),
        );

        app.add_systems(
            Update,
            systems::player::update_invincibility.in_set(GameSystemSet::Effects),
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
