use bevy::prelude::*;

use crate::{
    components::{
        GameSessionEntity,
        bullet::{BulletVelocity, DespawnOutOfBounds, PlayerBullet},
    },
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
// Power stage table
// ---------------------------------------------------------------------------

/// Parameters for one power stage of the Reimu A shot pattern.
///
/// | Field    | Description |
/// |----------|-------------|
/// | `count`  | Number of bullets per shot |
/// | `spread` | Total horizontal spread across the fan (px) |
/// | `damage` | Damage per bullet on contact |
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PowerStageParams {
    /// Number of bullets fired per shot.
    pub count: u8,
    /// Total horizontal span of the bullet fan in pixels.
    ///
    /// When `count == 1` this is ignored and the single bullet fires straight up.
    pub spread: f32,
    /// Damage dealt to an enemy per bullet contact.
    pub damage: f32,
}

/// Returns the [`PowerStageParams`] for the Reimu A shot at the given power level.
///
/// Eight stages are defined, each spanning 16 power units:
///
/// | Stage | Power   | Bullets | Spread (px) | Damage |
/// |-------|---------|---------|-------------|--------|
/// | 0     | 0–15    | 1       | 0           | 10.0   |
/// | 1     | 16–31   | 2       | 8           | 11.0   |
/// | 2     | 32–47   | 3       | 10          | 12.0   |
/// | 3     | 48–63   | 3       | 10          | 13.0   |
/// | 4     | 64–79   | 4       | 12          | 14.0   |
/// | 5     | 80–95   | 4       | 12          | 15.0   |
/// | 6     | 96–111  | 5       | 14          | 16.0   |
/// | 7     | 112–127 | 5       | 14          | 17.0   |
/// | 8     | 128     | 5       | 14          | 18.0   |
pub fn power_stage_params(power: u8) -> PowerStageParams {
    match power {
        0..=15 => PowerStageParams {
            count: 1,
            spread: 0.0,
            damage: 10.0,
        },
        16..=31 => PowerStageParams {
            count: 2,
            spread: 8.0,
            damage: 11.0,
        },
        32..=47 => PowerStageParams {
            count: 3,
            spread: 10.0,
            damage: 12.0,
        },
        48..=63 => PowerStageParams {
            count: 3,
            spread: 10.0,
            damage: 13.0,
        },
        64..=79 => PowerStageParams {
            count: 4,
            spread: 12.0,
            damage: 14.0,
        },
        80..=95 => PowerStageParams {
            count: 4,
            spread: 12.0,
            damage: 15.0,
        },
        96..=111 => PowerStageParams {
            count: 5,
            spread: 14.0,
            damage: 16.0,
        },
        112..=127 => PowerStageParams {
            count: 5,
            spread: 14.0,
            damage: 17.0,
        },
        _ => PowerStageParams {
            // Power 128 (MAX).
            count: 5,
            spread: 14.0,
            damage: 18.0,
        },
    }
}

/// Returns the fire interval in seconds for the given power level.
///
/// Higher power stages shoot faster:
///
/// | Power    | Interval (s) | Shots/s (approx) |
/// |----------|--------------|------------------|
/// | 0–15     | 0.200        | 5                |
/// | 16–31    | 0.160        | 6.25             |
/// | 32–47    | 0.140        | ~7.1             |
/// | 48–63    | 0.120        | ~8.3             |
/// | 64–95    | 0.100        | 10               |
/// | 96–127   | 0.083        | ~12              |
/// | 128      | 0.070        | ~14.3            |
pub fn shoot_interval_from_power(power: u8) -> f32 {
    match power {
        0..=15 => 0.200,
        16..=31 => 0.160,
        32..=47 => 0.140,
        48..=63 => 0.120,
        64..=95 => 0.100,
        96..=127 => 0.083,
        _ => 0.070, // Power 128 (MAX).
    }
}

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
/// Bullet count, spread, and damage are determined by the current power level
/// via the 8-stage table in [`power_stage_params`]. Forward speed and sprite
/// dimensions are read from [`PlayerBulletConfigParams`].
///
/// | Power   | Bullets | Spread (px) | Damage |
/// |---------|---------|-------------|--------|
/// | 0–15    | 1       | 0           | 10     |
/// | 16–31   | 2       | 8           | 11     |
/// | 32–47   | 3       | 10          | 12     |
/// | 48–63   | 3       | 10          | 13     |
/// | 64–79   | 4       | 12          | 14     |
/// | 80–95   | 4       | 12          | 15     |
/// | 96–111  | 5       | 14          | 16     |
/// | 112–127 | 5       | 14          | 17     |
/// | 128     | 5       | 14          | 18     |
///
/// Registered in [`crate::GameSystemSet::BulletEmit`].
pub fn bullet_spawn_system(
    mut commands: Commands,
    mut events: MessageReader<ShootEvent>,
    game_data: Res<GameData>,
    bullet_cfg: PlayerBulletConfigParams,
) {
    for event in events.read() {
        let stage = power_stage_params(game_data.power);
        spawn_bullet_stream(&mut commands, event.origin, stage, &bullet_cfg);
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Spawns bullets in a horizontal fan centred on `origin` using the given
/// power-stage parameters.
///
/// Speed, horizontal-velocity scale, spawn y-offset, and sprite dimensions
/// come from [`PlayerBulletConfigParams`] to remain configurable without a
/// code change.
fn spawn_bullet_stream(
    commands: &mut Commands,
    origin: Vec2,
    stage: PowerStageParams,
    cfg: &PlayerBulletConfigParams,
) {
    let speed = cfg.speed();
    let spread_speed_scale = cfg.spread_speed_scale();
    let origin_y_offset = cfg.origin_y_offset();
    let sprite_w = cfg.sprite_width();
    let sprite_h = cfg.sprite_height();

    let count = stage.count.max(1);
    for i in 0..count {
        let x_offset = if count > 1 {
            let t = i as f32 / (count - 1) as f32; // 0.0 … 1.0
            -stage.spread / 2.0 + stage.spread * t
        } else {
            0.0
        };

        commands.spawn((
            PlayerBullet {
                damage: stage.damage,
            },
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
            GameSessionEntity,
        ));
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- power_stage_params ------------------------------------------------

    /// Stage boundaries: every 16-unit boundary must flip to the correct stage.
    #[test]
    fn power_stage_boundaries() {
        // Stage 0 (0–15)
        assert_eq!(power_stage_params(0).count, 1);
        assert_eq!(power_stage_params(15).count, 1);
        // Stage 1 (16–31)
        assert_eq!(power_stage_params(16).count, 2);
        assert_eq!(power_stage_params(31).count, 2);
        // Stage 2 (32–47)
        assert_eq!(power_stage_params(32).count, 3);
        assert_eq!(power_stage_params(47).count, 3);
        // Stage 3 (48–63): same count as stage 2, higher damage
        assert_eq!(power_stage_params(48).count, 3);
        assert_eq!(power_stage_params(63).count, 3);
        // Stage 4 (64–79)
        assert_eq!(power_stage_params(64).count, 4);
        assert_eq!(power_stage_params(79).count, 4);
        // Stage 5 (80–95): same count as stage 4, higher damage
        assert_eq!(power_stage_params(80).count, 4);
        assert_eq!(power_stage_params(95).count, 4);
        // Stage 6 (96–111)
        assert_eq!(power_stage_params(96).count, 5);
        assert_eq!(power_stage_params(111).count, 5);
        // Stage 7 (112–127): same count as stage 6, higher damage
        assert_eq!(power_stage_params(112).count, 5);
        assert_eq!(power_stage_params(127).count, 5);
        // Stage 8 (128 MAX)
        assert_eq!(power_stage_params(128).count, 5);
    }

    /// Damage must be strictly non-decreasing as power increases.
    #[test]
    fn damage_non_decreasing() {
        let checkpoints: &[u8] = &[0, 16, 32, 48, 64, 80, 96, 112, 128];
        let damages: Vec<f32> = checkpoints
            .iter()
            .map(|&p| power_stage_params(p).damage)
            .collect();
        for window in damages.windows(2) {
            assert!(
                window[1] >= window[0],
                "damage must not decrease: {} → {}",
                window[0],
                window[1]
            );
        }
    }

    /// Stage 0 must have spread == 0 so the single bullet goes straight up.
    #[test]
    fn stage_0_has_zero_spread() {
        let stage = power_stage_params(0);
        assert_eq!(stage.spread, 0.0, "single-bullet stage must have no spread");
    }

    /// All stages must have spread >= 0 and damage > 0.
    #[test]
    fn all_stages_valid() {
        for power in [0u8, 16, 32, 48, 64, 80, 96, 112, 128] {
            let p = power_stage_params(power);
            assert!(p.count >= 1, "power {power}: count must be >= 1");
            assert!(p.spread >= 0.0, "power {power}: spread must be >= 0");
            assert!(p.damage > 0.0, "power {power}: damage must be > 0");
        }
    }

    // ---- shoot_interval_from_power ----------------------------------------

    /// Interval must be strictly non-increasing as power rises.
    #[test]
    fn interval_non_increasing() {
        let checkpoints: &[u8] = &[0, 16, 32, 48, 64, 96, 128];
        let intervals: Vec<f32> = checkpoints
            .iter()
            .map(|&p| shoot_interval_from_power(p))
            .collect();
        for window in intervals.windows(2) {
            assert!(
                window[1] <= window[0],
                "interval must not increase: {} → {}",
                window[0],
                window[1]
            );
        }
    }

    /// All intervals must be positive.
    #[test]
    fn all_intervals_positive() {
        for power in [0u8, 15, 16, 31, 32, 63, 64, 95, 96, 127, 128] {
            assert!(
                shoot_interval_from_power(power) > 0.0,
                "power {power}: interval must be positive"
            );
        }
    }

    /// Power 128 (MAX) must have the shortest interval.
    #[test]
    fn max_power_has_shortest_interval() {
        assert!(
            shoot_interval_from_power(128) < shoot_interval_from_power(0),
            "max power must fire faster than min power"
        );
    }

    // ---- legacy tests (renamed) -------------------------------------------

    #[test]
    fn despawn_margin_is_positive() {
        assert!(DESPAWN_MARGIN > 0.0);
    }
}
