use bevy::prelude::*;

/// Marker component for player-fired bullets.
///
/// Attached to every bullet the player shoots. Used to distinguish
/// player bullets from enemy bullets in queries.
#[derive(Component)]
pub struct PlayerBullet {
    /// Damage dealt to an enemy on contact.
    pub damage: f32,
}

impl Default for PlayerBullet {
    fn default() -> Self {
        Self { damage: 12.0 }
    }
}

/// Velocity of a bullet in pixels per second.
///
/// Applied every frame by [`crate::systems::bullet::bullet_movement_system`].
/// Used by both player bullets and enemy bullets.
#[derive(Component)]
pub struct BulletVelocity(pub Vec2);

/// Repeating timer that controls the player's fire rate.
///
/// Attached to the [`crate::components::Player`] entity.
/// [`crate::systems::player::shoot_input_system`] ticks this timer and
/// emits [`crate::events::ShootEvent`] only when it fires.
#[derive(Component)]
pub struct ShootTimer {
    pub timer: Timer,
}

impl Default for ShootTimer {
    fn default() -> Self {
        Self {
            // 10 shots per second.
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
        }
    }
}

/// Marker component — entity is despawned when it leaves the play area.
///
/// Checked every frame by
/// [`crate::systems::bullet::despawn_out_of_bounds_system`].
/// Attach to any entity that should be cleaned up automatically
/// (player bullets, enemy bullets, items, etc.).
#[derive(Component)]
pub struct DespawnOutOfBounds;
