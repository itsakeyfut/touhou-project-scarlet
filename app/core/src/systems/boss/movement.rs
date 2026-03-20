use std::f32::consts::TAU;

use bevy::prelude::*;

use crate::{
    components::boss::{Boss, BossMovement},
    constants::{PLAY_AREA_HALF_H, PLAY_AREA_HALF_W},
};

/// Moves boss entities each frame according to the [`BossMovement`] variant
/// stored in the currently active [`crate::components::boss::BossPhaseData`].
///
/// # Movement variants
///
/// | Variant | Behaviour |
/// |---|---|
/// | `Static` | No movement; boss stays at its current position. |
/// | `Pendulum` | Sinusoidal horizontal sweep around `base_x`. |
/// | `Circle` | Constant-speed orbit around a fixed `center`. |
/// | `Teleport` | Waits at each waypoint for `wait_secs`, then snaps to the next. |
///
/// All resulting positions are clamped to the play area
/// (`±PLAY_AREA_HALF_W`, `±PLAY_AREA_HALF_H`) so the boss cannot leave the
/// visible field regardless of pattern parameters.
///
/// # Pendulum formula
///
/// ```text
/// x = base_x + amplitude × sin(elapsed × frequency × 2π)
/// ```
///
/// The Y coordinate is unchanged so the boss sweeps purely horizontally.
///
/// # Circle formula
///
/// ```text
/// x = center.x + radius × cos(elapsed × speed_deg_per_s × π/180)
/// y = center.y + radius × sin(elapsed × speed_deg_per_s × π/180)
/// ```
///
/// Positive `speed_deg` rotates counter-clockwise.
///
/// # Teleport
///
/// `BossMovement::Teleport` stores mutable per-phase state (`elapsed_secs`,
/// `current`) directly in the component so no extra resources are needed.
/// The waypoint list cycles: after the last waypoint, the boss returns to index 0.
///
/// Registered in [`crate::GameSystemSet::Movement`].
pub fn boss_movement_system(mut bosses: Query<(&mut Transform, &mut Boss)>, time: Res<Time>) {
    let dt = time.delta_secs();
    let elapsed = time.elapsed_secs();

    for (mut tf, mut boss) in &mut bosses {
        let idx = boss.current_phase;

        let Some(phase) = boss.phases.get_mut(idx) else {
            continue;
        };

        match &mut phase.movement {
            BossMovement::Static => {
                // No movement — position stays wherever it was spawned / last placed.
            }

            BossMovement::Pendulum {
                amplitude,
                frequency,
                base_x,
            } => {
                // Horizontal sine oscillation; Y is not modified.
                tf.translation.x = *base_x + *amplitude * (*frequency * elapsed * TAU).sin();
            }

            BossMovement::Circle {
                radius,
                speed_deg,
                center,
            } => {
                let angle = elapsed * speed_deg.to_radians();
                tf.translation.x = center.x + *radius * angle.cos();
                tf.translation.y = center.y + *radius * angle.sin();
            }

            BossMovement::Teleport {
                waypoints,
                wait_secs,
                elapsed_secs,
                current,
            } => {
                if !waypoints.is_empty() {
                    // Normalize index defensively in case phase data is malformed.
                    *current %= waypoints.len();

                    // Snap to current waypoint position immediately (handles first frame
                    // and ensures the boss is always at a valid waypoint).
                    let pos = waypoints[*current];
                    tf.translation.x = pos.x;
                    tf.translation.y = pos.y;

                    // Advance the wait timer; preserve leftover time so cadence stays
                    // accurate even on long frames.
                    *elapsed_secs += dt;
                    if *wait_secs > 0.0 && *elapsed_secs >= *wait_secs {
                        let steps = (*elapsed_secs / *wait_secs).floor() as usize;
                        *current = (*current + steps) % waypoints.len();
                        *elapsed_secs -= *wait_secs * steps as f32;
                    }
                }
            }
        }

        // Clamp to play area so no movement pattern can push the boss off-screen.
        tf.translation.x = tf.translation.x.clamp(-PLAY_AREA_HALF_W, PLAY_AREA_HALF_W);
        tf.translation.y = tf.translation.y.clamp(-PLAY_AREA_HALF_H, PLAY_AREA_HALF_H);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Helpers that reproduce the exact formula used in boss_movement_system
    // without needing a full Bevy App setup.

    fn pendulum_x(amplitude: f32, frequency: f32, base_x: f32, elapsed: f32) -> f32 {
        base_x + amplitude * (frequency * elapsed * TAU).sin()
    }

    fn circle_pos(radius: f32, speed_deg: f32, center: Vec2, elapsed: f32) -> Vec2 {
        let angle = elapsed * speed_deg.to_radians();
        Vec2::new(
            center.x + radius * angle.cos(),
            center.y + radius * angle.sin(),
        )
    }

    /// Pendulum at t=0 must be at base_x (sin(0) = 0).
    #[test]
    fn pendulum_at_zero_time_is_at_base_x() {
        let x = pendulum_x(80.0, 0.4, 0.0, 0.0);
        assert!(x.abs() < 1e-5, "expected 0.0, got {x}");
    }

    /// Pendulum peak magnitude must equal the amplitude.
    #[test]
    fn pendulum_peak_equals_amplitude() {
        // Peak at t = 0.25 / frequency (quarter cycle).
        let amplitude = 80.0_f32;
        let frequency = 0.4_f32;
        let base_x = 0.0_f32;
        let t_peak = 0.25 / frequency;
        let x = pendulum_x(amplitude, frequency, base_x, t_peak);
        assert!(
            (x - amplitude).abs() < 1e-3,
            "peak should be {amplitude}, got {x}"
        );
    }

    /// Pendulum must be symmetric: x at t and at t + half-period have opposite signs.
    #[test]
    fn pendulum_is_symmetric() {
        let amplitude = 60.0_f32;
        let frequency = 0.5_f32;
        let base_x = 10.0_f32;
        let t = 0.3_f32;
        let half_period = 1.0 / frequency / 2.0;
        let x1 = pendulum_x(amplitude, frequency, base_x, t);
        let x2 = pendulum_x(amplitude, frequency, base_x, t + half_period);
        // The offsets from base_x should be equal and opposite.
        assert!(
            ((x1 - base_x) + (x2 - base_x)).abs() < 1e-3,
            "offsets {x1} and {x2} should be symmetric about {base_x}"
        );
    }

    /// Circle at t=0 must be at (center.x + radius, center.y) (cos(0)=1, sin(0)=0).
    #[test]
    fn circle_at_zero_time_starts_at_rightmost_point() {
        let pos = circle_pos(60.0, 30.0, Vec2::new(0.0, 80.0), 0.0);
        assert!((pos.x - 60.0).abs() < 1e-4, "x should be 60, got {}", pos.x);
        assert!((pos.y - 80.0).abs() < 1e-4, "y should be 80, got {}", pos.y);
    }

    /// Circle position must always lie on the orbit circle (distance from center == radius).
    #[test]
    fn circle_position_is_always_on_orbit() {
        let radius = 60.0_f32;
        let center = Vec2::new(0.0, 80.0);
        for i in 0..8 {
            let t = i as f32 * 0.3;
            let pos = circle_pos(radius, 30.0, center, t);
            let dist = (pos - center).length();
            assert!(
                (dist - radius).abs() < 1e-3,
                "at t={t} distance from center should be {radius}, got {dist}"
            );
        }
    }

    // Helper that mirrors the production teleport logic (steps-based, remainder-preserving).
    fn teleport_tick(
        current: &mut usize,
        elapsed_secs: &mut f32,
        wait_secs: f32,
        dt: f32,
        len: usize,
    ) {
        *elapsed_secs += dt;
        if wait_secs > 0.0 && *elapsed_secs >= wait_secs {
            let steps = (*elapsed_secs / wait_secs).floor() as usize;
            *current = (*current + steps) % len;
            *elapsed_secs -= wait_secs * steps as f32;
        }
    }

    /// Teleport index must advance after wait_secs elapsed.
    #[test]
    fn teleport_advances_index_after_wait() {
        let wait_secs = 2.0_f32;
        let mut current: usize = 0;
        let mut elapsed_secs: f32 = 0.0;
        let len = 2;

        // Simulate several small ticks that add up to just over wait_secs.
        for _ in 0..25 {
            teleport_tick(&mut current, &mut elapsed_secs, wait_secs, 0.09, len);
            // 25 × 0.09 = 2.25 s — should advance once
        }
        assert_eq!(current, 1, "should have advanced to waypoint 1");
    }

    /// Teleport must cycle back to index 0 after the last waypoint.
    #[test]
    fn teleport_cycles_back_to_zero() {
        let wait_secs = 1.0_f32;
        let len = 3;
        let mut current = 0usize;
        let mut elapsed = 0.0_f32;

        // Advance through all three waypoints and back (4 advances → index 1).
        for _ in 0..4 {
            teleport_tick(&mut current, &mut elapsed, wait_secs, wait_secs + 0.01, len);
        }
        assert_eq!(current, 1, "after 4 advances modulo 3, current should be 1");
    }

    /// A single large dt that covers multiple wait intervals must skip the correct number of waypoints.
    #[test]
    fn teleport_large_dt_skips_multiple_waypoints() {
        let wait_secs = 1.0_f32;
        let len = 4;
        let mut current = 0usize;
        let mut elapsed = 0.0_f32;

        // One tick of 3.5 s — should advance 3 waypoints (indices 0→3) and leave 0.5 s remainder.
        teleport_tick(&mut current, &mut elapsed, wait_secs, 3.5, len);
        assert_eq!(current, 3, "3.5 s / 1.0 s = 3 steps");
        assert!((elapsed - 0.5).abs() < 1e-4, "remainder should be 0.5, got {elapsed}");
    }

    /// Play-area clamp: values outside bounds must be clamped.
    #[test]
    fn position_is_clamped_to_play_area() {
        // Simulate what the clamping step does.
        let x_raw = 999.0_f32;
        let y_raw = -999.0_f32;
        let x = x_raw.clamp(-PLAY_AREA_HALF_W, PLAY_AREA_HALF_W);
        let y = y_raw.clamp(-PLAY_AREA_HALF_H, PLAY_AREA_HALF_H);
        assert_eq!(x, PLAY_AREA_HALF_W);
        assert_eq!(y, -PLAY_AREA_HALF_H);
    }
}
