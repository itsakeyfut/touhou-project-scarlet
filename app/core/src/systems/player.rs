use bevy::prelude::*;

use crate::{
    components::{
        GameSessionEntity,
        bullet::ShootTimer,
        player::{InvincibilityTimer, Player, PlayerStats},
    },
    config::PlayerConfigParams,
    constants::{PLAY_AREA_HALF_H, PLAY_AREA_HALF_W},
    events::ShootEvent,
    resources::GameData,
    systems::bullet::shoot_interval_from_power,
};

/// Spawns the player entity at the bottom-center of the play area.
///
/// Stats and fire rate are read from [`PlayerConfigParams`] (loaded from
/// `assets/config/player.ron`).  Falls back to [`PlayerStats::default`] and
/// [`ShootTimer::default`] if the config has not finished loading yet.
///
/// Uses a placeholder colored rectangle until real sprites are added in Phase 19.
/// [`ShootTimer`] is attached here so the firing rate is tracked per-player.
pub fn spawn_player(mut commands: Commands, player_cfg: PlayerConfigParams) {
    commands.spawn((
        Player,
        PlayerStats {
            speed: player_cfg.speed(),
            slow_speed: player_cfg.slow_speed(),
            hitbox_radius: player_cfg.hitbox_radius(),
            graze_radius: player_cfg.graze_radius(),
            pickup_radius: player_cfg.pickup_radius(),
        },
        ShootTimer {
            // Initial interval matches power 0; update_shoot_timer_system
            // adjusts it dynamically as power changes during play.
            timer: Timer::from_seconds(shoot_interval_from_power(0), TimerMode::Repeating),
        },
        Sprite {
            color: Color::srgb(1.0, 0.3, 0.3),
            custom_size: Some(Vec2::splat(16.0)),
            ..default()
        },
        Transform::from_xyz(0.0, -PLAY_AREA_HALF_H + 60.0, 1.0),
        GameSessionEntity,
    ));
}

/// Reads Z-key input and emits [`ShootEvent`] at the rate set by [`ShootTimer`].
///
/// Registered in [`crate::GameSystemSet::Input`].
/// The event carries the player's current world position so that
/// [`crate::systems::bullet::bullet_spawn_system`] can place bullets correctly
/// without needing to query the player again.
pub fn shoot_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Transform, &mut ShootTimer), With<Player>>,
    mut events: MessageWriter<ShootEvent>,
    time: Res<Time>,
) {
    let Ok((transform, mut timer)) = query.single_mut() else {
        return;
    };

    timer.timer.tick(time.delta());

    if keys.pressed(KeyCode::KeyZ) && timer.timer.just_finished() {
        events.write(ShootEvent {
            origin: transform.translation.truncate(),
        });
    }
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

/// Ticks the [`InvincibilityTimer`] and removes it once it expires.
///
/// While the timer is running the player sprite flickers at 10 Hz to signal
/// the invincibility window visually. When the timer finishes the sprite is
/// restored to full opacity and the component is removed.
///
/// Registered in [`crate::GameSystemSet::Effects`].
pub fn update_invincibility(
    mut commands: Commands,
    mut query: Query<(Entity, &mut InvincibilityTimer, &mut Sprite), With<Player>>,
    time: Res<Time>,
) {
    let Ok((entity, mut invincibility, mut sprite)) = query.single_mut() else {
        return;
    };

    invincibility.timer.tick(time.delta());

    if invincibility.timer.is_finished() {
        sprite.color = sprite.color.with_alpha(1.0);
        commands.entity(entity).remove::<InvincibilityTimer>();
    } else {
        // Flicker at 10 Hz: alternate between full opacity and 20% opacity.
        let visible = ((invincibility.timer.elapsed_secs() * 10.0) as u32).is_multiple_of(2);
        sprite.color = sprite.color.with_alpha(if visible { 1.0 } else { 0.2 });
    }
}

/// Updates [`ShootTimer`] duration whenever the player's power level changes.
///
/// Uses a [`Local`] to track the last-known power value. When `game_data.power`
/// differs, `ShootTimer.timer` is set to the new interval from
/// [`shoot_interval_from_power`] without resetting elapsed time, so the next
/// shot fires at the correct cadence with minimal disruption.
///
/// Both power-up and power-down paths are handled: power increases when items
/// are collected; power decreases by 16 when the player is hit.
///
/// Registered in [`crate::GameSystemSet::PlayerLogic`].
pub fn update_shoot_timer_system(
    mut player: Query<&mut ShootTimer, With<Player>>,
    game_data: Res<GameData>,
    mut prev_power: Local<u8>,
) {
    let power = game_data.power;
    if power == *prev_power {
        return;
    }
    *prev_power = power;

    let Ok(mut timer) = player.single_mut() else {
        return;
    };

    let interval = shoot_interval_from_power(power);
    timer
        .timer
        .set_duration(std::time::Duration::from_secs_f32(interval));
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
