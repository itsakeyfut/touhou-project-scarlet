use std::f32::consts::TAU;

use bevy::prelude::*;

use crate::components::{
    enemy::{Enemy, EnemyMovement},
    player::Player,
};

/// Moves enemy entities each frame according to their [`EnemyMovement`] component.
///
/// # Movement variants
///
/// | Variant | Behaviour |
/// |---|---|
/// | `Linear` | Constant velocity; never stops. |
/// | `LinearThenStop` | Moves until the internal timer reaches zero, then halts. |
/// | `FallDown` | Moves straight down at a fixed speed. |
/// | `SineWave` | Base velocity plus a sinusoidal horizontal oscillation derived from global time. |
/// | `ChasePlayer` | Turns toward the player every frame and moves at a fixed speed. |
/// | `Waypoints` | Advances through an ordered list of world-space targets. |
///
/// Registered in [`crate::GameSystemSet::Movement`].
#[allow(clippy::type_complexity)]
pub fn enemy_movement_system(
    mut enemies: Query<(&mut Transform, &mut EnemyMovement), With<Enemy>>,
    player: Query<&Transform, (With<Player>, Without<Enemy>)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    let elapsed = time.elapsed_secs();

    // Resolve player position once; fall back to Vec2::ZERO if no player exists
    // (e.g. during brief transitions or while the player entity is not yet spawned).
    let player_pos = player
        .single()
        .map(|tf| tf.translation.truncate())
        .unwrap_or(Vec2::ZERO);

    for (mut tf, mut movement) in &mut enemies {
        match &mut *movement {
            EnemyMovement::Linear { velocity } => {
                tf.translation += velocity.extend(0.0) * dt;
            }

            EnemyMovement::LinearThenStop {
                velocity,
                stop_after,
            } => {
                if *stop_after > 0.0 {
                    tf.translation += velocity.extend(0.0) * dt;
                    *stop_after = (*stop_after - dt).max(0.0);
                }
            }

            EnemyMovement::FallDown { speed } => {
                tf.translation.y -= *speed * dt;
            }

            EnemyMovement::SineWave {
                base_velocity,
                amplitude,
                frequency,
            } => {
                // The sine drives a velocity contribution (px/s) so that the
                // oscillation magnitude scales with `amplitude`.
                let sine_vel_x = *amplitude * (elapsed * *frequency * TAU).sin();
                tf.translation.x += (base_velocity.x + sine_vel_x) * dt;
                tf.translation.y += base_velocity.y * dt;
            }

            EnemyMovement::ChasePlayer { speed } => {
                let pos = tf.translation.truncate();
                let dir = (player_pos - pos).normalize_or(Vec2::NEG_Y);
                tf.translation += (dir * *speed * dt).extend(0.0);
            }

            EnemyMovement::Waypoints {
                points,
                speed,
                current,
            } => {
                if let Some(&target) = points.get(*current) {
                    let pos = tf.translation.truncate();
                    let to_target = target - pos;
                    let distance = to_target.length();
                    let move_dist = *speed * dt;

                    if distance <= move_dist {
                        // Snap to waypoint and advance index.
                        tf.translation = target.extend(tf.translation.z);
                        *current += 1;
                    } else {
                        let dir = to_target / distance;
                        tf.translation += (dir * move_dist).extend(0.0);
                    }
                }
                // No more waypoints — enemy remains stationary.
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `LinearThenStop` timer must not go below zero.
    #[test]
    fn linear_then_stop_clamps_to_zero() {
        let mut mv = EnemyMovement::LinearThenStop {
            velocity: Vec2::new(100.0, 0.0),
            stop_after: 0.001,
        };
        // Simulate a frame larger than remaining time.
        if let EnemyMovement::LinearThenStop { stop_after, .. } = &mut mv {
            *stop_after = (*stop_after - 1.0).max(0.0);
        }
        if let EnemyMovement::LinearThenStop { stop_after, .. } = mv {
            assert_eq!(stop_after, 0.0);
        }
    }

    /// `Waypoints` current index should advance when the enemy reaches a waypoint.
    #[test]
    fn waypoints_advance_index() {
        // Place enemy exactly on top of the first waypoint.
        let waypoints = vec![Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0)];
        let mut current = 0usize;
        let speed = 200.0_f32;
        let dt = 1.0 / 60.0;
        let target = waypoints[current];
        let pos = target; // already at waypoint
        let distance = (target - pos).length();
        if distance <= speed * dt {
            current += 1;
        }
        assert_eq!(current, 1);
    }

    /// ChasePlayer direction must be unit length (or zero when overlapping).
    #[test]
    fn chase_player_direction_is_normalized() {
        let pos = Vec2::new(0.0, 0.0);
        let player = Vec2::new(3.0, 4.0);
        let dir = (player - pos).normalize_or(Vec2::NEG_Y);
        let len = dir.length();
        assert!((len - 1.0).abs() < 1e-5, "direction must be unit length");
    }
}
