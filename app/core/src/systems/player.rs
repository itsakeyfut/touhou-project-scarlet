use bevy::prelude::*;

use crate::{
    components::player::{Player, PlayerStats},
    constants::{PLAY_AREA_HALF_H, PLAY_AREA_HALF_W},
};

/// Spawns the player entity at the bottom-center of the play area.
///
/// Uses a placeholder colored rectangle until real sprites are added in Phase 19.
pub fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player,
        PlayerStats::default(),
        Sprite {
            color: Color::srgb(1.0, 0.3, 0.3),
            custom_size: Some(Vec2::splat(16.0)),
            ..default()
        },
        Transform::from_xyz(0.0, -PLAY_AREA_HALF_H + 60.0, 1.0),
    ));
}

/// Moves the player based on keyboard input.
///
/// - Arrow keys or WASD for 8-directional movement.
/// - Shift held: focus (slow) mode — uses [`PlayerStats::slow_speed`].
/// - Diagonal input is normalised so diagonal speed equals cardinal speed.
/// - Position is clamped to the play area defined by [`PLAY_AREA_HALF_W`] / [`PLAY_AREA_HALF_H`].
pub fn player_movement_system(
    mut query: Query<(&PlayerStats, &mut Transform), With<Player>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let Ok((stats, mut transform)) = query.single_mut() else {
        return;
    };

    let slow = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    let speed = if slow { stats.slow_speed } else { stats.speed };

    let mut dir = Vec2::ZERO;
    if keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA) {
        dir.x -= 1.0;
    }
    if keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD) {
        dir.x += 1.0;
    }
    if keys.pressed(KeyCode::ArrowUp) || keys.pressed(KeyCode::KeyW) {
        dir.y += 1.0;
    }
    if keys.pressed(KeyCode::ArrowDown) || keys.pressed(KeyCode::KeyS) {
        dir.y -= 1.0;
    }

    // Normalise so diagonal movement is not faster than cardinal movement.
    if dir != Vec2::ZERO {
        dir = dir.normalize();
    }

    let delta = dir * speed * time.delta_secs();
    transform.translation.x =
        (transform.translation.x + delta.x).clamp(-PLAY_AREA_HALF_W, PLAY_AREA_HALF_W);
    transform.translation.y =
        (transform.translation.y + delta.y).clamp(-PLAY_AREA_HALF_H, PLAY_AREA_HALF_H);
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- helpers --------------------------------------------------------

    /// Simulate one movement tick: returns the new position after applying
    /// `dir` (pre-normalised if non-zero), `speed`, and `dt`.
    fn apply_movement(pos: Vec2, dir: Vec2, speed: f32, dt: f32) -> Vec2 {
        let dir = if dir != Vec2::ZERO {
            dir.normalize()
        } else {
            dir
        };
        let delta = dir * speed * dt;
        Vec2::new(
            (pos.x + delta.x).clamp(-PLAY_AREA_HALF_W, PLAY_AREA_HALF_W),
            (pos.y + delta.y).clamp(-PLAY_AREA_HALF_H, PLAY_AREA_HALF_H),
        )
    }

    // ---- direction tests ------------------------------------------------

    /// Pressing right must increase x; y must stay the same.
    #[test]
    fn move_right_increases_x() {
        let start = Vec2::ZERO;
        let result = apply_movement(start, Vec2::new(1.0, 0.0), 200.0, 0.016);
        assert!(result.x > start.x, "x should increase when moving right");
        assert_eq!(result.y, start.y);
    }

    /// Pressing left must decrease x; y must stay the same.
    #[test]
    fn move_left_decreases_x() {
        let start = Vec2::ZERO;
        let result = apply_movement(start, Vec2::new(-1.0, 0.0), 200.0, 0.016);
        assert!(result.x < start.x, "x should decrease when moving left");
        assert_eq!(result.y, start.y);
    }

    /// Pressing up must increase y; x must stay the same.
    #[test]
    fn move_up_increases_y() {
        let start = Vec2::ZERO;
        let result = apply_movement(start, Vec2::new(0.0, 1.0), 200.0, 0.016);
        assert!(result.y > start.y, "y should increase when moving up");
        assert_eq!(result.x, start.x);
    }

    /// Pressing down must decrease y; x must stay the same.
    #[test]
    fn move_down_decreases_y() {
        let start = Vec2::ZERO;
        let result = apply_movement(start, Vec2::new(0.0, -1.0), 200.0, 0.016);
        assert!(result.y < start.y, "y should decrease when moving down");
        assert_eq!(result.x, start.x);
    }

    // ---- boundary clamp tests -------------------------------------------

    /// Player moving right past the right wall must be clamped to PLAY_AREA_HALF_W.
    #[test]
    fn clamp_right_boundary() {
        let result = apply_movement(
            Vec2::new(PLAY_AREA_HALF_W, 0.0),
            Vec2::new(1.0, 0.0),
            200.0,
            1.0,
        );
        assert_eq!(result.x, PLAY_AREA_HALF_W);
    }

    /// Player moving left past the left wall must be clamped to -PLAY_AREA_HALF_W.
    #[test]
    fn clamp_left_boundary() {
        let result = apply_movement(
            Vec2::new(-PLAY_AREA_HALF_W, 0.0),
            Vec2::new(-1.0, 0.0),
            200.0,
            1.0,
        );
        assert_eq!(result.x, -PLAY_AREA_HALF_W);
    }

    /// Player moving up past the top wall must be clamped to PLAY_AREA_HALF_H.
    #[test]
    fn clamp_top_boundary() {
        let result = apply_movement(
            Vec2::new(0.0, PLAY_AREA_HALF_H),
            Vec2::new(0.0, 1.0),
            200.0,
            1.0,
        );
        assert_eq!(result.y, PLAY_AREA_HALF_H);
    }

    /// Player moving down past the bottom wall must be clamped to -PLAY_AREA_HALF_H.
    #[test]
    fn clamp_bottom_boundary() {
        let result = apply_movement(
            Vec2::new(0.0, -PLAY_AREA_HALF_H),
            Vec2::new(0.0, -1.0),
            200.0,
            1.0,
        );
        assert_eq!(result.y, -PLAY_AREA_HALF_H);
    }

    // ---- diagonal speed tests -------------------------------------------

    /// Diagonal input direction must have length 1.0 after normalisation.
    #[test]
    fn diagonal_direction_is_normalised() {
        let raw = Vec2::new(1.0, 1.0);
        let normalised = raw.normalize();
        assert!(
            (normalised.length() - 1.0).abs() < f32::EPSILON,
            "diagonal direction length should be 1.0, got {}",
            normalised.length()
        );
    }

    /// Diagonal displacement magnitude must equal cardinal displacement magnitude.
    ///
    /// Without normalisation a diagonal move would be √2 times faster than a
    /// cardinal move at the same speed and dt.
    #[test]
    fn diagonal_speed_equals_cardinal_speed() {
        let speed = 200.0_f32;
        let dt = 0.016_f32;

        let cardinal_delta = Vec2::new(1.0, 0.0) * speed * dt;
        let diagonal_dir = Vec2::new(1.0, 1.0).normalize();
        let diagonal_delta = diagonal_dir * speed * dt;

        assert!(
            (diagonal_delta.length() - cardinal_delta.length()).abs() < 1e-4,
            "diagonal displacement ({}) should equal cardinal displacement ({})",
            diagonal_delta.length(),
            cardinal_delta.length(),
        );
    }

    // ---- speed stat tests -----------------------------------------------

    /// Normal speed and slow speed must differ.
    #[test]
    fn slow_speed_is_less_than_normal_speed() {
        let stats = PlayerStats::default();
        assert!(stats.slow_speed < stats.speed);
    }
}
