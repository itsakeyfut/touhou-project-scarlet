use bevy::prelude::*;

use crate::{
    components::{
        bullet::{BulletEmitter, BulletPattern},
        player::Player,
    },
    systems::danmaku::patterns::{emit_aimed, emit_ring, emit_spiral_frame, emit_spread},
};

// ---------------------------------------------------------------------------
// SpiralState component
// ---------------------------------------------------------------------------

/// Per-entity state for the [`BulletPattern::Spiral`] pattern.
///
/// The current angle is advanced every frame by
/// [`update_spiral_emitters`] based on `BulletPattern::rotation_speed_deg`.
/// Attach this component to any emitter entity whose pattern is
/// [`BulletPattern::Spiral`].
#[derive(Component)]
pub struct SpiralState {
    /// Accumulated rotation angle in degrees.
    pub current_angle: f32,
}

impl Default for SpiralState {
    fn default() -> Self {
        Self { current_angle: 0.0 }
    }
}

// ---------------------------------------------------------------------------
// Emitter systems
// ---------------------------------------------------------------------------

/// Ticks timer-based emitters and fires bullets when the timer elapses.
///
/// Handles `Ring`, `Aimed`, and `Spread` patterns. Entities with
/// [`SpiralState`] are excluded — those are handled by
/// [`update_spiral_emitters`].
///
/// Registered in [`crate::GameSystemSet::BulletEmit`].
pub fn bullet_emitter_system(
    mut commands: Commands,
    mut emitters: Query<(&Transform, &mut BulletEmitter), Without<SpiralState>>,
    player: Query<&Transform, (With<Player>, Without<BulletEmitter>)>,
    time: Res<Time>,
) {
    let player_pos = player
        .single()
        .map(|tf| tf.translation.truncate())
        .unwrap_or(Vec2::ZERO);

    for (transform, mut emitter) in &mut emitters {
        if !emitter.active {
            continue;
        }
        if !emitter.timer.tick(time.delta()).just_finished() {
            continue;
        }

        let origin = transform.translation.truncate();
        let kind = emitter.bullet_kind;

        match emitter.pattern.clone() {
            BulletPattern::Ring { count, speed } => {
                emit_ring(&mut commands, origin, count, speed, kind);
            }
            BulletPattern::Aimed {
                count,
                spread_deg,
                speed,
            } => {
                emit_aimed(
                    &mut commands,
                    origin,
                    player_pos,
                    count,
                    spread_deg,
                    speed,
                    kind,
                );
            }
            BulletPattern::Spread {
                count,
                spread_deg,
                speed,
                angle_offset,
            } => {
                emit_spread(
                    &mut commands,
                    origin,
                    count,
                    spread_deg,
                    speed,
                    angle_offset,
                    kind,
                );
            }
            BulletPattern::Spiral { .. } => {
                // Spiral entities must also have SpiralState and are handled
                // by update_spiral_emitters. This branch is unreachable in
                // normal gameplay but is kept for exhaustiveness.
            }
        }
    }
}

/// Advances the spiral angle and emits one wave of bullets every frame.
///
/// Unlike timer-based patterns the spiral fires continuously each frame
/// so that the rotation looks smooth. The emitter's `timer` field is
/// not used by this system.
///
/// Registered in [`crate::GameSystemSet::BulletEmit`].
pub fn update_spiral_emitters(
    mut commands: Commands,
    mut spirals: Query<(&Transform, &BulletEmitter, &mut SpiralState)>,
    time: Res<Time>,
) {
    for (transform, emitter, mut state) in &mut spirals {
        if !emitter.active {
            continue;
        }

        if let BulletPattern::Spiral {
            arms,
            speed,
            rotation_speed_deg,
        } = emitter.pattern
        {
            state.current_angle += rotation_speed_deg * time.delta_secs();
            let origin = transform.translation.truncate();
            emit_spiral_frame(
                &mut commands,
                origin,
                arms,
                speed,
                state.current_angle,
                emitter.bullet_kind,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spiral_state_default_angle_is_zero() {
        let state = SpiralState::default();
        assert_eq!(state.current_angle, 0.0);
    }

    #[test]
    fn spiral_angle_advances_with_time() {
        let mut state = SpiralState::default();
        let rotation_speed_deg = 90.0_f32;
        let dt = 1.0_f32; // 1 second
        state.current_angle += rotation_speed_deg * dt;
        assert!((state.current_angle - 90.0).abs() < 1e-4);
    }
}
