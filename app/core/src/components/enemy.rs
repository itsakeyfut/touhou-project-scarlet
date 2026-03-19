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
    /// Score awarded to the player when this enemy is defeated.
    ///
    /// Added to [`crate::resources::GameData::score`] via
    /// [`crate::events::EnemyDefeatedEvent`] in
    /// [`crate::systems::item::on_enemy_defeated`].
    pub score_value: u32,
}

impl Enemy {
    /// Creates a new enemy with the given HP and collision radius.
    ///
    /// `score_value` defaults to `0`; use [`Enemy::with_score`] when the enemy
    /// should award points on defeat.
    pub fn new(hp: f32, collision_radius: f32) -> Self {
        Self {
            hp,
            collision_radius,
            score_value: 0,
        }
    }

    /// Creates a new enemy with an explicit score value.
    pub fn with_score(hp: f32, collision_radius: f32, score_value: u32) -> Self {
        Self {
            hp,
            collision_radius,
            score_value,
        }
    }
}
