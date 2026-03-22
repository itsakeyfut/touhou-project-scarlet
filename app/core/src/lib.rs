use bevy::prelude::*;

pub mod components;
pub mod config;
pub mod constants;
pub mod debug;
pub mod events;
pub mod game_set;
pub mod resources;
pub mod shaders;
pub mod stages;
pub mod states;
pub mod systems;

pub use components::{
    Boss, BossMovement, BossPhaseData, BossType, BulletEmitter, BulletPattern, BulletTrail,
    BulletVelocity, DespawnOutOfBounds, Enemy, EnemyBullet, EnemyBulletKind, EnemyKind,
    EnemyMovement, GameSessionEntity, GrazeVisual, InvincibilityTimer, ItemKind, ItemPhysics,
    Player, PlayerBullet, PlayerStats, ShootTimer,
};
pub use config::{
    EnemyBulletConfig, EnemyBulletConfigHandle, EnemyBulletConfigParams, FodderEnemyConfig,
    FodderEnemyConfigHandle, FodderEnemyConfigParams, GameRulesConfig, GameRulesConfigHandle,
    GameRulesConfigParams, PlayerBulletConfig, PlayerBulletConfigHandle, PlayerBulletConfigParams,
    PlayerConfig, PlayerConfigHandle, PlayerConfigParams, ScarletConfigPlugin,
};
pub use constants::{PLAY_AREA_HALF_H, PLAY_AREA_HALF_W, PLAY_AREA_HEIGHT, PLAY_AREA_WIDTH};
pub use events::{
    BombUsedEvent, BossHitEvent, BossPhaseChangedEvent, BossSpawnEvent, EnemyDefeatedEvent,
    ExtendEvent, ExtendKind, GrazeEvent, PlayerHitEvent, ShootEvent,
};
pub use game_set::GameSystemSet;
pub use resources::{
    BOMB_DURATION_SECS, BOMB_EXTEND_FRAGMENTS, BOMB_INVINCIBLE_SECS, BombState,
    COUNTER_BOMB_WINDOW_SECS, EnemySpawner, FragmentTracker, GameData, LIFE_EXTEND_FRAGMENTS,
    SpawnEntry, StageData,
};
pub use shaders::{
    GrazeMaterial, HitFlashMaterial, ScarletShadersPlugin, SpellCardBackground, SpellCardBgMaterial,
};
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
        // Config plugin must be registered first so asset loaders and handles
        // are available before any system tries to access them.
        app.add_plugins(config::ScarletConfigPlugin);

        app.add_plugins(shaders::ScarletShadersPlugin);

        app.init_state::<AppState>();

        // Events.
        app.add_message::<ShootEvent>();
        app.add_message::<PlayerHitEvent>();
        app.add_message::<GrazeEvent>();
        app.add_message::<EnemyDefeatedEvent>();
        app.add_message::<ExtendEvent>();
        app.add_message::<BossSpawnEvent>();
        app.add_message::<BossPhaseChangedEvent>();
        app.add_message::<BossHitEvent>();
        app.add_message::<BombUsedEvent>();

        // Resources — inserted with game-start values.
        app.insert_resource(GameData::new_game());
        app.insert_resource(FragmentTracker::default());
        app.insert_resource(StageData::default());
        app.insert_resource(EnemySpawner::default());
        app.insert_resource(BombState::default());

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

        // Per-run reset: restore all session resources to initial values before
        // the stage loader runs so the spawner script is fresh when populated.
        app.add_systems(
            OnEnter(AppState::Playing),
            systems::reset::reset_per_run_resources.before(stages::stage1::load_stage1_system),
        );

        app.add_systems(
            OnEnter(AppState::Playing),
            stages::stage1::load_stage1_system,
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

        // Bomb systems.
        // bomb_input_system runs in Input so counter-bomb window set by
        // handle_player_hit (GameLogic, previous frame) is available.
        // bomb_effect_system runs in GameLogic after Collision so graze is
        // finalised before bullets are cleared.
        // bomb_active_system runs in Effects to tick timers last.
        app.add_systems(
            Update,
            systems::bomb::bomb_input_system.in_set(GameSystemSet::Input),
        )
        .add_systems(
            Update,
            systems::bomb::bomb_effect_system.in_set(GameSystemSet::GameLogic),
        )
        .add_systems(
            Update,
            systems::bomb::bomb_active_system.in_set(GameSystemSet::Effects),
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
            systems::item::item_movement_system.in_set(GameSystemSet::Movement),
        )
        .add_systems(
            Update,
            systems::enemy::movement::enemy_movement_system.in_set(GameSystemSet::Movement),
        )
        .add_systems(
            Update,
            systems::boss::movement::boss_movement_system.in_set(GameSystemSet::Movement),
        )
        .add_systems(
            Update,
            systems::bullet::despawn_out_of_bounds_system.in_set(GameSystemSet::Cleanup),
        )
        .add_systems(
            Update,
            systems::enemy::cull::enemy_cull_system.in_set(GameSystemSet::Cleanup),
        );

        // Collision systems.
        //
        // player_bullet_hit_enemy and player_bullet_hit_boss are chained with
        // apply_deferred between them: both query bullets immutably and mutate
        // different components (Enemy vs Boss), so Bevy would otherwise run them
        // in parallel. A bullet despawned by the first system must be flushed
        // before the second system runs, preventing the same bullet from
        // registering a hit against both a regular enemy and a boss.
        app.add_systems(
            Update,
            (
                systems::collision::player_bullet_hit_enemy,
                ApplyDeferred,
                systems::collision::player_bullet_hit_boss,
            )
                .chain()
                .in_set(GameSystemSet::Collision),
        );
        app.add_systems(
            Update,
            (
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
                (
                    systems::item::item_collection_system,
                    systems::score::check_extend_system,
                )
                    .chain(),
            )
                .in_set(GameSystemSet::GameLogic),
        );

        // Boss phase transition systems — chained so that emitter-swap and
        // spell-card-background systems see the BossPhaseChangedEvent written
        // by boss_phase_system in the same frame.
        app.add_systems(
            Update,
            (
                systems::boss::phase::boss_phase_system,
                (
                    systems::boss::phase::update_boss_emitter_on_phase_change,
                    systems::boss::phase::on_spell_card_start,
                ),
            )
                .chain()
                .in_set(GameSystemSet::GameLogic),
        );

        // Spawn the spell-card background when a boss's initial phase is already
        // a spell card (BossPhaseChangedEvent is not emitted for phase 0).
        app.add_systems(
            Update,
            systems::boss::phase::spawn_initial_spell_card_bg.in_set(GameSystemSet::GameLogic),
        );

        // StageControl systems — stage_control runs first to update elapsed_time,
        // then enemy_spawner uses the fresh time to process the script.
        // on_boss_spawn_stage1 runs last to react to the BossSpawnEvent emitted
        // by stage_control_system in the same frame.
        app.add_systems(
            Update,
            (
                systems::stage::stage_control_system,
                systems::enemy::spawner::enemy_spawner_system,
                systems::boss::bosses::rumia::on_boss_spawn_stage1,
            )
                .chain()
                .in_set(GameSystemSet::StageControl),
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

        // Config readiness gate: advances Loading → Playing once all RON files
        // have been fully loaded by the asset server.
        app.add_systems(
            Update,
            systems::loading::wait_for_configs.run_if(in_state(AppState::Loading)),
        );

        // Session cleanup: despawns all GameSessionEntity entities when
        // leaving Playing so re-entering starts from a clean world.
        app.add_systems(
            OnExit(AppState::Playing),
            systems::cleanup::cleanup_game_session,
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
