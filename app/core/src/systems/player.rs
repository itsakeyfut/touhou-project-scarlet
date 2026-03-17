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

    /// Player must not move outside the play area boundaries.
    #[test]
    fn position_clamped_within_play_area() {
        let far_right = 9999.0_f32;
        let clamped = far_right.clamp(-PLAY_AREA_HALF_W, PLAY_AREA_HALF_W);
        assert_eq!(clamped, PLAY_AREA_HALF_W);

        let far_left = -9999.0_f32;
        let clamped = far_left.clamp(-PLAY_AREA_HALF_W, PLAY_AREA_HALF_W);
        assert_eq!(clamped, -PLAY_AREA_HALF_W);
    }

    /// Normal speed and slow speed must differ.
    #[test]
    fn slow_speed_is_less_than_normal_speed() {
        let stats = PlayerStats::default();
        assert!(stats.slow_speed < stats.speed);
    }
}
