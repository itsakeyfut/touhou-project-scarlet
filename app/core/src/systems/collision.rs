use std::collections::HashSet;

use bevy::prelude::*;

use crate::{
    components::{
        bullet::{EnemyBullet, EnemyBulletKind, PlayerBullet},
        enemy::Enemy,
        player::{InvincibilityTimer, Player, PlayerStats},
    },
    constants::PLAY_AREA_HALF_H,
    events::PlayerHitEvent,
    resources::GameData,
    states::AppState,
};

/// Collision radius used for player bullet → enemy hit detection (px).
///
/// Player bullet sprites are 4×12 px; using the half-width plus a 1 px
/// margin produces a feel close to the original game.
const PLAYER_BULLET_RADIUS: f32 = 3.0;

// ---------------------------------------------------------------------------
// Collision utility
// ---------------------------------------------------------------------------

/// Returns `true` if two circles overlap or are exactly touching.
///
/// Uses a squared-distance comparison to avoid the cost of a square root.
/// This is the shared primitive for all collision pairs in the game
/// (player hit, graze, item pickup).
///
/// # Arguments
///
/// * `pos_a`    – Centre of circle A in world space.
/// * `radius_a` – Radius of circle A in pixels.
/// * `pos_b`    – Centre of circle B in world space.
/// * `radius_b` – Radius of circle B in pixels.
///
/// # Examples
///
/// ```rust
/// use bevy::math::Vec2;
/// use scarlet_core::systems::collision::check_circle_collision;
///
/// // Overlapping
/// assert!(check_circle_collision(Vec2::ZERO, 5.0, Vec2::new(8.0, 0.0), 5.0));
/// // Not touching
/// assert!(!check_circle_collision(Vec2::ZERO, 3.0, Vec2::new(10.0, 0.0), 3.0));
/// ```
pub fn check_circle_collision(pos_a: Vec2, radius_a: f32, pos_b: Vec2, radius_b: f32) -> bool {
    let sum_r = radius_a + radius_b;
    pos_a.distance_squared(pos_b) <= sum_r * sum_r
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Detects collisions between player bullets and [`Enemy`] entities.
///
/// For each active player bullet a linear scan over all enemies is performed.
/// When a hit is detected:
///
/// - The bullet is despawned (one bullet hits at most one enemy per frame).
/// - The enemy loses HP equal to [`PlayerBullet::damage`].
/// - Enemies whose HP drops to ≤ 0 are despawned.
///
/// Collision uses [`check_circle_collision`] with [`PLAYER_BULLET_RADIUS`]
/// for the bullet and [`Enemy::collision_radius`] for the enemy.
///
/// Registered in [`crate::GameSystemSet::Collision`].
pub fn player_bullet_hit_enemy(
    mut commands: Commands,
    bullets: Query<(Entity, &Transform, &PlayerBullet)>,
    mut enemies: Query<(Entity, &Transform, &mut Enemy)>,
) {
    // Track entities already consumed this frame (commands are deferred,
    // so despawns are not applied until after the system completes).
    // hit_bullets: prevents one bullet from hitting multiple enemies.
    // hit_enemies:  prevents already-defeated enemies from absorbing more bullets.
    let mut hit_bullets: HashSet<Entity> = HashSet::new();
    let mut hit_enemies: HashSet<Entity> = HashSet::new();

    for (bullet_entity, bullet_tf, player_bullet) in &bullets {
        if hit_bullets.contains(&bullet_entity) {
            continue;
        }

        let bullet_pos = bullet_tf.translation.truncate();

        for (enemy_entity, enemy_tf, mut enemy) in &mut enemies {
            if hit_enemies.contains(&enemy_entity) {
                continue;
            }

            let enemy_pos = enemy_tf.translation.truncate();

            if check_circle_collision(
                bullet_pos,
                PLAYER_BULLET_RADIUS,
                enemy_pos,
                enemy.collision_radius,
            ) {
                enemy.hp -= player_bullet.damage;
                commands.entity(bullet_entity).despawn();
                hit_bullets.insert(bullet_entity);

                if enemy.hp <= 0.0 {
                    commands.entity(enemy_entity).despawn();
                    hit_enemies.insert(enemy_entity);
                }

                // Bullet is spent — move on to the next bullet.
                break;
            }
        }
    }
}

/// Checks whether any enemy bullet overlaps the player's hitbox and, if so,
/// emits a [`PlayerHitEvent`].
///
/// - Uses [`PlayerStats::hitbox_radius`] for the player circle.
/// - Uses [`EnemyBulletKind::collision_radius`] for each bullet circle.
/// - Skips the check entirely while [`InvincibilityTimer`] is present on the player.
/// - Emits at most one [`PlayerHitEvent`] per frame regardless of bullet count.
///
/// Registered in [`crate::GameSystemSet::Collision`].
pub fn player_hit_detection(
    player: Query<(&Transform, &PlayerStats, Option<&InvincibilityTimer>), With<Player>>,
    bullets: Query<(&Transform, &EnemyBulletKind), With<EnemyBullet>>,
    mut hit_events: MessageWriter<PlayerHitEvent>,
) {
    let Ok((player_tf, stats, invincibility)) = player.single() else {
        return;
    };

    // Skip detection while the player is invincible.
    if invincibility.is_some() {
        return;
    }

    let player_pos = player_tf.translation.truncate();

    for (bullet_tf, kind) in &bullets {
        let bullet_pos = bullet_tf.translation.truncate();
        if check_circle_collision(
            player_pos,
            stats.hitbox_radius,
            bullet_pos,
            kind.collision_radius(),
        ) {
            hit_events.write(PlayerHitEvent { bullet_damage: 1 });
            // One event per frame — avoid rapid-fire deaths on the same tick.
            return;
        }
    }
}

/// Responds to [`PlayerHitEvent`] by decrementing lives, applying power loss,
/// resetting the player's position, and starting the invincibility window.
///
/// # Game over
///
/// When [`GameData::lives`] reaches 0 the state machine transitions to
/// [`AppState::GameOver`] and no further hit processing occurs.
///
/// Registered in [`crate::GameSystemSet::GameLogic`].
pub fn handle_player_hit(
    mut commands: Commands,
    mut hit_events: MessageReader<PlayerHitEvent>,
    mut game_data: ResMut<GameData>,
    mut player: Query<(Entity, &mut Transform), With<Player>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    // Only react to the first event per frame.
    let Some(_event) = hit_events.read().next() else {
        return;
    };

    let Ok((player_entity, mut transform)) = player.single_mut() else {
        return;
    };

    // Decrement lives (saturating prevents wrapping on u8).
    game_data.lives = game_data.lives.saturating_sub(1);

    // Power loss: drop by 16 per hit, floor at 0 (matches original EoSD).
    game_data.power = game_data.power.saturating_sub(16);

    if game_data.lives == 0 {
        next_state.set(AppState::GameOver);
        return;
    }

    // Reset the player to the standard spawn position.
    transform.translation = Vec3::new(0.0, -PLAY_AREA_HALF_H + 60.0, 1.0);

    // Start the 3-second invincibility window.
    commands.entity(player_entity).insert(InvincibilityTimer {
        timer: Timer::from_seconds(3.0, TimerMode::Once),
    });
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Circles whose centres are closer than the sum of their radii must collide.
    #[test]
    fn overlapping_circles_collide() {
        assert!(check_circle_collision(
            Vec2::ZERO,
            5.0,
            Vec2::new(8.0, 0.0),
            5.0
        ));
    }

    /// Circles whose centres are farther than the sum of their radii must not collide.
    #[test]
    fn separated_circles_do_not_collide() {
        assert!(!check_circle_collision(
            Vec2::ZERO,
            3.0,
            Vec2::new(10.0, 0.0),
            3.0
        ));
    }

    /// Circles that are exactly touching (distance == sum of radii) count as a hit.
    #[test]
    fn touching_circles_collide() {
        // distance = 10, radii sum = 5 + 5 = 10 → exactly touching
        assert!(check_circle_collision(
            Vec2::ZERO,
            5.0,
            Vec2::new(10.0, 0.0),
            5.0
        ));
    }

    /// Identical positions always collide regardless of radii.
    #[test]
    fn same_position_always_collides() {
        assert!(check_circle_collision(
            Vec2::new(42.0, -7.0),
            0.0,
            Vec2::new(42.0, -7.0),
            0.0
        ));
    }

    /// Zero-radius circles separated by any positive distance must not collide.
    #[test]
    fn zero_radius_separated_does_not_collide() {
        assert!(!check_circle_collision(
            Vec2::ZERO,
            0.0,
            Vec2::new(1.0, 0.0),
            0.0
        ));
    }

    /// Collision is symmetric: swapping A and B must give the same result.
    #[test]
    fn collision_is_symmetric() {
        let a = (Vec2::new(1.0, 2.0), 4.0_f32);
        let b = (Vec2::new(5.0, 5.0), 3.0_f32);
        assert_eq!(
            check_circle_collision(a.0, a.1, b.0, b.1),
            check_circle_collision(b.0, b.1, a.0, a.1)
        );
    }
}
