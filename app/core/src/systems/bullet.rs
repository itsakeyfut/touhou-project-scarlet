use bevy::prelude::*;

use crate::{
    components::bullet::{BulletVelocity, DespawnOutOfBounds, PlayerBullet},
    config::PlayerBulletConfigParams,
    constants::{PLAY_AREA_HALF_H, PLAY_AREA_HALF_W},
    events::ShootEvent,
    resources::GameData,
};

/// Extra margin (px) beyond the play-area boundary before bullets are despawned.
///
/// Prevents visible pop-in/out at the exact edge of the play area.
const DESPAWN_MARGIN: f32 = 32.0;

// ---------------------------------------------------------------------------
// Public systems
// ---------------------------------------------------------------------------

/// Moves every entity that has a [`BulletVelocity`] component.
///
/// Registered in [`crate::GameSystemSet::Movement`].
/// Applies to both player bullets and enemy bullets — attach
/// [`BulletVelocity`] to any entity that should move autonomously.
pub fn bullet_movement_system(
    mut query: Query<(&mut Transform, &BulletVelocity)>,
    time: Res<Time>,
) {
    for (mut transform, vel) in &mut query {
        transform.translation += (vel.0 * time.delta_secs()).extend(0.0);
    }
}

/// Despawns entities with [`DespawnOutOfBounds`] that have left the play area.
///
/// A small [`DESPAWN_MARGIN`] is added beyond the play-area boundary so that
/// bullets that are partially visible at the edge are not removed prematurely.
///
/// Registered in [`crate::GameSystemSet::Cleanup`].
pub fn despawn_out_of_bounds_system(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<DespawnOutOfBounds>>,
) {
    let half_w = PLAY_AREA_HALF_W + DESPAWN_MARGIN;
    let half_h = PLAY_AREA_HALF_H + DESPAWN_MARGIN;

    for (entity, transform) in &query {
        let pos = transform.translation.truncate();
        if pos.x.abs() > half_w || pos.y.abs() > half_h {
            commands.entity(entity).despawn();
        }
    }
}

/// Spawns player bullets in response to [`ShootEvent`].
///
/// The number of bullets and their horizontal spread are determined by the
/// current power level in [`GameData`]:
///
/// | Power   | Bullets |
/// |---------|---------|
/// | 0–31    | 1       |
/// | 32–63   | 3       |
/// | 64–95   | 4       |
/// | 96–128  | 5       |
///
/// Registered in [`crate::GameSystemSet::BulletEmit`].
pub fn bullet_spawn_system(
    mut commands: Commands,
    mut events: MessageReader<ShootEvent>,
    game_data: Res<GameData>,
    bullet_cfg: PlayerBulletConfigParams,
) {
    for event in events.read() {
        let count = bullet_count_from_power(game_data.power);
        spawn_bullet_stream(&mut commands, event.origin, count, &bullet_cfg);
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Returns the number of bullet streams for the given power level.
fn bullet_count_from_power(power: u8) -> u8 {
    match power {
        0..=31 => 1,
        32..=63 => 3,
        64..=95 => 4,
        _ => 5,
    }
}

/// Spawns `count` bullets in a horizontal fan centred on `origin`.
fn spawn_bullet_stream(
    commands: &mut Commands,
    origin: Vec2,
    count: u8,
    cfg: &PlayerBulletConfigParams,
) {
    let spread = cfg.spread();
    let speed = cfg.speed();
    let spread_speed_scale = cfg.spread_speed_scale();
    let origin_y_offset = cfg.origin_y_offset();
    let sprite_w = cfg.sprite_width();
    let sprite_h = cfg.sprite_height();
    let damage = cfg.damage();

    let count = count.max(1);
    for i in 0..count {
        let x_offset = if count > 1 {
            let t = i as f32 / (count - 1) as f32; // 0.0 … 1.0
            -spread / 2.0 + spread * t
        } else {
            0.0
        };

        commands.spawn((
            PlayerBullet { damage },
            BulletVelocity(Vec2::new(x_offset * spread_speed_scale, speed)),
            Sprite {
                color: Color::srgb(1.0, 0.8, 0.0),
                custom_size: Some(Vec2::new(sprite_w, sprite_h)),
                ..default()
            },
            Transform::from_translation(
                (origin + Vec2::new(x_offset, origin_y_offset)).extend(2.0),
            ),
            DespawnOutOfBounds,
        ));
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bullet_count_from_power_ranges() {
        assert_eq!(bullet_count_from_power(0), 1);
        assert_eq!(bullet_count_from_power(31), 1);
        assert_eq!(bullet_count_from_power(32), 3);
        assert_eq!(bullet_count_from_power(63), 3);
        assert_eq!(bullet_count_from_power(64), 4);
        assert_eq!(bullet_count_from_power(95), 4);
        assert_eq!(bullet_count_from_power(96), 5);
        assert_eq!(bullet_count_from_power(128), 5);
    }

    #[test]
    fn despawn_margin_is_positive() {
        assert!(DESPAWN_MARGIN > 0.0);
    }

    #[test]
    fn bullet_count_max_is_five() {
        // Power is capped at 128; ensure we never exceed 5 bullets.
        assert_eq!(bullet_count_from_power(128), 5);
    }
}
