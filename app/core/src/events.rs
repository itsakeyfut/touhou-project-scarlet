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
