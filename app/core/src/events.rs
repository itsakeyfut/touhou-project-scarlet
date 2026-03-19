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
