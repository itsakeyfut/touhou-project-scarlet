use bevy::prelude::*;

use crate::components::bullet::{BulletVelocity, DespawnOutOfBounds, EnemyBullet, EnemyBulletKind};

// ---------------------------------------------------------------------------
// Public emit helpers (called by emitter systems)
// ---------------------------------------------------------------------------

/// Fires `count` bullets equally spaced around a full 360° circle.
pub fn emit_ring(
    commands: &mut Commands,
    origin: Vec2,
    count: u8,
    speed: f32,
    kind: EnemyBulletKind,
) {
    let step = 360.0 / count.max(1) as f32;
    for i in 0..count {
        let angle = (step * i as f32).to_radians();
        let dir = Vec2::from_angle(angle);
        spawn_enemy_bullet(commands, origin, dir * speed, kind);
    }
}

/// Fires `count` bullets in a fan aimed at `player_pos`.
///
/// When `count` is 1 the single bullet points directly at the player.
/// When `spread_deg` is 0 all bullets travel in the same direction.
pub fn emit_aimed(
    commands: &mut Commands,
    origin: Vec2,
    player_pos: Vec2,
    count: u8,
    spread_deg: f32,
    speed: f32,
    kind: EnemyBulletKind,
) {
    let base_dir = (player_pos - origin).normalize_or(Vec2::NEG_Y);
    let base_angle = base_dir.to_angle();
    let half = spread_deg.to_radians() / 2.0;
    let step = if count > 1 {
        spread_deg.to_radians() / (count - 1) as f32
    } else {
        0.0
    };

    for i in 0..count {
        let angle = base_angle - half + step * i as f32;
        let dir = Vec2::from_angle(angle);
        spawn_enemy_bullet(commands, origin, dir * speed, kind);
    }
}

/// Fires `count` bullets in a fixed fan at `angle_offset` degrees from straight down.
pub fn emit_spread(
    commands: &mut Commands,
    origin: Vec2,
    count: u8,
    spread_deg: f32,
    speed: f32,
    angle_offset: f32,
    kind: EnemyBulletKind,
) {
    let count = count.max(1);
    let step = if count > 1 {
        spread_deg / (count - 1) as f32
    } else {
        0.0
    };

    for i in 0..count {
        // Straight down = -PI/2; positive angle_offset rotates clockwise.
        let angle = (-spread_deg / 2.0 + step * i as f32 + angle_offset).to_radians()
            - std::f32::consts::FRAC_PI_2;
        let dir = Vec2::from_angle(angle);
        spawn_enemy_bullet(commands, origin, dir * speed, kind);
    }
}

/// Fires `arms` bullets at `current_angle` + equally spaced arm offsets.
///
/// Called every frame by the spiral emitter system; the angle is advanced
/// externally by [`super::emitter::SpiralState`].
pub fn emit_spiral_frame(
    commands: &mut Commands,
    origin: Vec2,
    arms: u8,
    speed: f32,
    current_angle: f32,
    kind: EnemyBulletKind,
) {
    let arm_gap = 360.0 / arms.max(1) as f32;
    for arm in 0..arms {
        let angle = (current_angle + arm_gap * arm as f32).to_radians();
        let dir = Vec2::from_angle(angle);
        spawn_enemy_bullet(commands, origin, dir * speed, kind);
    }
}

// ---------------------------------------------------------------------------
// Internal helper
// ---------------------------------------------------------------------------

pub(super) fn spawn_enemy_bullet(
    commands: &mut Commands,
    origin: Vec2,
    velocity: Vec2,
    kind: EnemyBulletKind,
) {
    commands.spawn((
        EnemyBullet { damage: 1 },
        kind,
        BulletVelocity(velocity),
        Sprite {
            color: kind.color(),
            custom_size: Some(kind.sprite_size()),
            ..default()
        },
        // TODO(phase-04, #30): replace Sprite with Mesh2d + BulletGlowMaterial
        Transform::from_translation(origin.extend(1.5)),
        DespawnOutOfBounds,
    ));
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Ring with N bullets must produce N equally-spaced angles summing to 360°.
    #[test]
    fn ring_angles_cover_full_circle() {
        let count = 8u8;
        let step = 360.0 / count as f32;
        let angles: Vec<f32> = (0..count).map(|i| step * i as f32).collect();
        // Last angle must be exactly 7 steps from 0, not 360 itself.
        assert!((angles.last().unwrap() - 315.0_f32).abs() < 1e-3);
    }

    /// Aimed at a target directly below: base angle should be -90° (NEG_Y).
    #[test]
    fn aimed_at_below_points_neg_y() {
        let origin = Vec2::ZERO;
        let target = Vec2::new(0.0, -100.0);
        let base_dir = (target - origin).normalize_or(Vec2::NEG_Y);
        let angle_deg = base_dir.to_angle().to_degrees();
        assert!(
            (angle_deg - (-90.0_f32)).abs() < 1e-3,
            "angle = {angle_deg}"
        );
    }

    /// Spread with count=1 should use step=0 (single centre bullet).
    #[test]
    fn spread_single_bullet_has_zero_step() {
        let count = 1u8;
        let step = if count > 1 {
            45.0 / (count - 1) as f32
        } else {
            0.0
        };
        assert_eq!(step, 0.0);
    }

    /// collision_radius values must be positive for all EnemyBulletKind variants.
    #[test]
    fn all_enemy_bullet_kinds_have_positive_radius() {
        use EnemyBulletKind::*;
        for kind in [
            SmallRound,
            MediumRound,
            LargeRound,
            Rice,
            Knife,
            Star,
            Bubble,
        ] {
            assert!(kind.collision_radius() > 0.0, "{kind:?} radius must be > 0");
        }
    }
}
