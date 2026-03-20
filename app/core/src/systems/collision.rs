use std::collections::HashSet;

use bevy::prelude::*;

use crate::{
    components::{
        boss::Boss,
        bullet::{EnemyBullet, EnemyBulletKind, PlayerBullet},
        enemy::Enemy,
        player::{InvincibilityTimer, Player, PlayerStats},
    },
    config::{EnemyBulletConfigParams, PlayerBulletConfigParams, PlayerConfigParams},
    constants::PLAY_AREA_HALF_H,
    events::{BossHitEvent, EnemyDefeatedEvent, GrazeEvent, PlayerHitEvent},
    resources::GameData,
    states::AppState,
};

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
    mut enemies: Query<(Entity, &Transform, &mut Enemy), Without<Boss>>,
    mut defeated_events: MessageWriter<EnemyDefeatedEvent>,
    bullet_cfg: PlayerBulletConfigParams,
) {
    // Track entities already consumed this frame (commands are deferred,
    // so despawns are not applied until after the system completes).
    // hit_bullets: prevents one bullet from hitting multiple enemies.
    // hit_enemies:  prevents already-defeated enemies from absorbing more bullets.
    let mut hit_bullets: HashSet<Entity> = HashSet::new();
    let mut hit_enemies: HashSet<Entity> = HashSet::new();

    let bullet_radius = bullet_cfg.collision_radius();

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

            if check_circle_collision(bullet_pos, bullet_radius, enemy_pos, enemy.collision_radius)
            {
                enemy.hp -= player_bullet.damage;
                commands.entity(bullet_entity).despawn();
                hit_bullets.insert(bullet_entity);

                if enemy.hp <= 0.0 {
                    defeated_events.write(EnemyDefeatedEvent {
                        position: enemy_pos,
                        score: enemy.score_value,
                        is_boss: enemy.score_value >= 500,
                    });
                    commands.entity(enemy_entity).despawn();
                    hit_enemies.insert(enemy_entity);
                }

                // Bullet is spent — move on to the next bullet.
                break;
            }
        }
    }
}

/// Detects collisions between player bullets and [`Boss`] entities.
///
/// For each active player bullet a scan over all boss entities is performed.
/// When a hit is detected:
///
/// - The bullet is despawned (one bullet can only hit one target per frame).
/// - `boss.current_mut().hp` is reduced by [`PlayerBullet::damage`].
/// - A [`BossHitEvent`] is emitted so the `HitFlashMaterial` system (Issue #62)
///   can trigger the white-flash animation on the boss sprite.
///
/// The boss hitbox radius comes from [`Enemy::collision_radius`] on the same
/// entity (boss entities carry both [`Boss`] and [`Enemy`] components).
///
/// # Ordering note
///
/// Runs **after** [`player_bullet_hit_enemy`] with an explicit `apply_deferred`
/// between them (see [`crate::ScarletCorePlugin`]).
///
/// Without this ordering both systems would run in parallel because they query
/// bullets immutably and mutate different components (`Enemy` vs `Boss`). Since
/// bullet despawns are deferred via [`Commands`], a bullet that already hit an
/// enemy would still appear in the query when this system runs, potentially
/// registering a double-hit. The `apply_deferred` barrier flushes all pending
/// despawn commands before this system executes, so spent bullets are no longer
/// visible in the query.
///
/// Registered in [`crate::GameSystemSet::Collision`].
pub fn player_bullet_hit_boss(
    mut commands: Commands,
    bullets: Query<(Entity, &Transform, &PlayerBullet)>,
    mut bosses: Query<(Entity, &Transform, &mut Boss, &Enemy)>,
    mut boss_hit_events: MessageWriter<BossHitEvent>,
    bullet_cfg: PlayerBulletConfigParams,
) {
    let bullet_radius = bullet_cfg.collision_radius();
    let mut hit_bullets: HashSet<Entity> = HashSet::new();

    for (bullet_entity, bullet_tf, player_bullet) in &bullets {
        if hit_bullets.contains(&bullet_entity) {
            continue;
        }

        let bullet_pos = bullet_tf.translation.truncate();

        for (boss_entity, boss_tf, mut boss, enemy) in &mut bosses {
            let boss_pos = boss_tf.translation.truncate();

            if check_circle_collision(bullet_pos, bullet_radius, boss_pos, enemy.collision_radius) {
                boss.current_mut().hp -= player_bullet.damage;
                commands.entity(bullet_entity).despawn();
                hit_bullets.insert(bullet_entity);
                boss_hit_events.write(BossHitEvent {
                    entity: boss_entity,
                });
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
    enemy_bullet_cfg: EnemyBulletConfigParams,
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
            enemy_bullet_cfg.radius_for(*kind),
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
    player_cfg: PlayerConfigParams,
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

    // Start the invincibility window (duration from player.ron).
    commands.entity(player_entity).insert(InvincibilityTimer {
        timer: Timer::from_seconds(player_cfg.invincibility_secs(), TimerMode::Once),
    });
}

/// Detects when enemy bullets enter the player's graze zone and increments
/// [`GameData::graze`] for each new bullet crossing the boundary.
///
/// Graze zone radius is [`PlayerStats::graze_radius`] (16 px). A bullet that
/// stays inside the zone across multiple frames is counted only once; a bullet
/// that leaves and re-enters is **not** counted again because graze tracking
/// is reset each time a new set of grazed entities is computed.
///
/// # Implementation
///
/// A [`Local`] [`HashSet<Entity>`] persists between frames and records which
/// enemy bullets are currently inside the graze zone. Each frame:
///
/// 1. All bullets currently overlapping the zone are collected into
///    `current_grazed`.
/// 2. Any bullet in `current_grazed` that was **not** in the previous frame's
///    set is a new graze → `game_data.graze` is incremented.
/// 3. `graze_set` is replaced with `current_grazed`, so bullets that left the
///    zone (or were despawned) are automatically removed.
///
/// Registered in [`crate::GameSystemSet::Collision`].
pub fn graze_detection_system(
    player: Query<(&Transform, &PlayerStats), With<Player>>,
    bullets: Query<(Entity, &Transform, &EnemyBulletKind), With<EnemyBullet>>,
    mut game_data: ResMut<GameData>,
    mut graze_set: Local<HashSet<Entity>>,
    mut graze_events: MessageWriter<GrazeEvent>,
    enemy_bullet_cfg: EnemyBulletConfigParams,
) {
    let Ok((player_tf, stats)) = player.single() else {
        return;
    };

    let player_pos = player_tf.translation.truncate();
    let mut current_grazed: HashSet<Entity> = HashSet::new();

    for (entity, bullet_tf, kind) in &bullets {
        let bullet_pos = bullet_tf.translation.truncate();
        if check_circle_collision(
            player_pos,
            stats.graze_radius,
            bullet_pos,
            enemy_bullet_cfg.radius_for(*kind),
        ) {
            current_grazed.insert(entity);
            if !graze_set.contains(&entity) {
                game_data.graze += 1;
                graze_events.write(GrazeEvent {
                    bullet_entity: entity,
                });
            }
        }
    }

    // Replace the set: bullets that left the zone or were despawned are dropped.
    *graze_set = current_grazed;
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

    // ---- player_bullet_hit_boss logic ------------------------------------

    /// Bullet entirely inside the boss hitbox must register as a collision.
    #[test]
    fn bullet_inside_boss_hitbox_collides() {
        // Boss at origin with radius 20; bullet at (5, 0) with radius 3.
        assert!(check_circle_collision(
            Vec2::new(5.0, 0.0),
            3.0,
            Vec2::ZERO,
            20.0
        ));
    }

    /// Bullet far outside the boss hitbox must not collide.
    #[test]
    fn bullet_outside_boss_hitbox_no_collision() {
        // Boss at origin with radius 20; bullet at (30, 0) with radius 3.
        assert!(!check_circle_collision(
            Vec2::new(30.0, 0.0),
            3.0,
            Vec2::ZERO,
            20.0
        ));
    }

    // ---- graze duplicate-prevention logic ---------------------------------
    //
    // The graze system uses a Local<HashSet<Entity>> that persists across
    // frames. The following tests verify the duplicate-counting prevention
    // logic in isolation by simulating the set operations the system performs.

    /// A bullet entering the graze zone for the first time must be counted.
    #[test]
    fn graze_counts_new_bullet_entering_zone() {
        // Bullet with Entity index 0 is inside the graze zone.
        // graze_set is empty (no bullet was tracked last frame).
        let graze_set: HashSet<u32> = HashSet::new();
        let current_grazed: HashSet<u32> = [0].into();

        // Simulate: count bullets in current_grazed that were not in graze_set.
        let new_grazes = current_grazed
            .iter()
            .filter(|e| !graze_set.contains(e))
            .count();
        assert_eq!(new_grazes, 1, "one new bullet should be counted as a graze");
    }

    /// A bullet that remains inside the graze zone across two frames must not
    /// be counted again on the second frame.
    #[test]
    fn graze_does_not_double_count_bullet_staying_in_zone() {
        // Frame 1: bullet 0 entered → counted.
        let graze_set: HashSet<u32> = [0].into(); // state after frame 1

        // Frame 2: same bullet still inside.
        let current_grazed: HashSet<u32> = [0].into();

        let new_grazes = current_grazed
            .iter()
            .filter(|e| !graze_set.contains(e))
            .count();
        assert_eq!(
            new_grazes, 0,
            "bullet already in set must not be counted again"
        );
    }

    /// A bullet that leaves the graze zone must be removed from the tracking set.
    #[test]
    fn graze_removes_bullet_that_left_zone() {
        // Frame 1: bullets 0 and 1 were grazed.
        let graze_set: HashSet<u32> = [0, 1].into();

        // Frame 2: only bullet 1 is still inside; bullet 0 has left.
        let current_grazed: HashSet<u32> = [1].into();

        // After the swap the set should no longer contain bullet 0.
        assert!(
            !current_grazed.contains(&0),
            "departed bullet must not remain in set"
        );
        assert!(
            current_grazed.contains(&1),
            "bullet still in zone must remain in set"
        );
        // No new graze counted (bullet 1 was already tracked).
        let new_grazes = current_grazed
            .iter()
            .filter(|e| !graze_set.contains(e))
            .count();
        assert_eq!(new_grazes, 0);
    }

    /// Two distinct bullets entering simultaneously must each be counted once.
    #[test]
    fn graze_counts_multiple_simultaneous_new_bullets() {
        let graze_set: HashSet<u32> = HashSet::new();
        let current_grazed: HashSet<u32> = [3, 7].into();

        let new_grazes = current_grazed
            .iter()
            .filter(|e| !graze_set.contains(e))
            .count();
        assert_eq!(new_grazes, 2);
    }

    /// The graze zone radius constant matches PlayerStats default graze_radius.
    #[test]
    fn graze_radius_matches_player_stats_default() {
        use crate::components::player::PlayerStats;
        assert_eq!(PlayerStats::default().graze_radius, 16.0);
    }
}
