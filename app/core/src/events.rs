use bevy::prelude::*;

/// Fired by [`crate::systems::player::shoot_input_system`] when the player
/// presses Z and the [`crate::components::ShootTimer`] cooldown elapses.
///
/// [`crate::systems::bullet::bullet_spawn_system`] consumes this event and
/// spawns one or more [`crate::components::PlayerBullet`] entities.
#[derive(Event, Message)]
pub struct ShootEvent {
    /// World-space position of the player at the moment of firing.
    ///
    /// The spawn system offsets each bullet relative to this origin so that
    /// bullets appear to come from the player's current location.
    pub origin: Vec2,
}

/// Fired when an enemy bullet makes contact with the player's hitbox.
///
/// Consumed by `handle_player_hit` (Phase 05) to decrement lives and
/// start the invincibility window. Emitted by
/// [`crate::systems::collision::player_hit_detection`] (Phase 05).
#[derive(Event, Message)]
pub struct PlayerHitEvent {
    /// Damage dealt by the colliding bullet.
    pub bullet_damage: u8,
}

/// Fired when an [`crate::components::Enemy`] entity's HP drops to ≤ 0.
///
/// Emitted by [`crate::systems::collision::player_bullet_hit_enemy`] at the
/// moment the killing hit is detected (within the same frame, before the
/// entity is actually despawned). Consumed by
/// [`crate::systems::item::on_enemy_defeated`] to drop items and award score.
#[derive(Event, Message)]
pub struct EnemyDefeatedEvent {
    /// World-space position of the enemy at the moment of defeat.
    ///
    /// Used as the spawn origin for dropped items. Captured before the despawn
    /// command is issued so that it is available even though the entity still
    /// exists in queries until the end of the frame.
    pub position: Vec2,
    /// Score value awarded to the player.
    pub score: u32,
    /// `true` for boss-tier enemies (score ≥ 500); `false` for normal enemies.
    ///
    /// Used by the item system to select an appropriate drop table.
    pub is_boss: bool,
}

/// The stock type gained when an extend is triggered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtendKind {
    /// One extra life granted by collecting five [`crate::components::ItemKind::LifeFragment`]s.
    Life,
    /// One extra bomb stock granted by collecting five [`crate::components::ItemKind::BombFragment`]s.
    Bomb,
}

/// Fired when the player earns an extra life or bomb stock through fragment collection.
///
/// Emitted by [`crate::systems::score::check_extend_system`] when a
/// [`crate::resources::FragmentTracker`] counter reaches its threshold
/// ([`crate::resources::LIFE_EXTEND_FRAGMENTS`] or
/// [`crate::resources::BOMB_EXTEND_FRAGMENTS`]).
///
/// Consumers (future audio / UI systems) can read this event to play the
/// extend jingle or display a notification.
#[derive(Event, Message)]
pub struct ExtendEvent {
    /// Which stock type was extended.
    pub kind: ExtendKind,
}

/// Fired when all normal enemies are defeated and the spawner script is
/// exhausted, signalling that it is time to spawn the stage boss.
///
/// Emitted by [`crate::systems::stage::stage_control_system`] and consumed
/// by the boss-spawn system added in Phase 08.
#[derive(Event, Message)]
pub struct BossSpawnEvent {
    /// The stage number for which the boss should be spawned.
    pub stage_number: u8,
}

/// Fired when the active boss phase changes (HP depleted or time expired).
///
/// Emitted by `boss_phase_system` (Issue #58) at the moment the transition
/// is detected. Consumed by:
/// - `update_boss_emitter_on_phase_change` (Issue #59) to swap the
///   [`crate::components::BulletEmitter`] pattern.
/// - The spell-card background system (Issue #62) to start/stop the
///   `SpellCardBgMaterial` overlay.
/// - The UI system to update the spell-card name display.
#[derive(Event, Message)]
pub struct BossPhaseChangedEvent {
    /// The boss entity whose phase changed.
    pub entity: Entity,
    /// Index of the **new** active phase inside [`crate::components::Boss::phases`].
    pub phase: usize,
}

/// Fired when a player bullet lands a hit on a boss entity.
///
/// Emitted by [`crate::systems::collision::player_bullet_hit_boss`] once per
/// bullet that connects. Consumed by:
/// - The `HitFlashMaterial` system (Issue #62) to trigger the white-flash
///   animation on the boss sprite.
/// - Future audio systems to play the hit sound effect.
#[derive(Event, Message)]
pub struct BossHitEvent {
    /// The boss entity that was hit.
    pub entity: Entity,
}

/// Fired when the player activates a bomb (X key with bombs > 0).
///
/// Emitted by [`crate::systems::bomb::bomb_input_system`] in the same frame
/// the X key is pressed. Consumed by:
/// - [`crate::systems::bomb::bomb_effect_system`] to clear all enemy bullets
///   and award bonus score.
/// - Future audio / visual systems to play the bomb animation.
#[derive(Event, Message)]
pub struct BombUsedEvent {
    /// `true` when the bomb cancelled an in-progress hit within the
    /// [`crate::resources::COUNTER_BOMB_WINDOW_SECS`] window.
    pub is_counter_bomb: bool,
}

/// Fired when an enemy bullet newly enters the player's graze zone (16 px).
///
/// Consumed by [`crate::shaders::plugin::update_graze_material`] to trigger
/// the electric spark animation on the graze-field ring. Emitted by
/// [`crate::systems::collision::graze_detection_system`].
#[derive(Event, Message)]
pub struct GrazeEvent {
    /// The enemy bullet entity that caused the graze.
    pub bullet_entity: Entity,
}
