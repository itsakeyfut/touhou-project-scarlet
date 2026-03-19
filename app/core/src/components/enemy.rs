use bevy::prelude::*;

/// Core component for any entity that the player can defeat.
///
/// Attach to enemy sprites, bosses, and other destructible entities.
/// HP and collision radius are stored together to keep the hot collision
/// query (`player_bullet_hit_enemy`) to a single component access.
#[derive(Component)]
pub struct Enemy {
    /// Current hit points. The entity is despawned when this reaches ≤ 0.
    pub hp: f32,
    /// Radius of the circle used for bullet-collision detection (px).
    pub collision_radius: f32,
}

impl Enemy {
    /// Creates a new enemy with the given HP and collision radius.
    pub fn new(hp: f32, collision_radius: f32) -> Self {
        Self {
            hp,
            collision_radius,
        }
    }
}
