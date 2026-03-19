use bevy::prelude::*;

use crate::{
    components::enemy::Enemy,
    constants::{PLAY_AREA_HALF_H, PLAY_AREA_HALF_W},
};

/// Extra margin (px) outside the play area before an enemy is culled.
///
/// Gives enemies spawned just off-screen a chance to enter the visible area
/// before being removed, and prevents visible pop-in at the edges.
const CULL_MARGIN: f32 = 80.0;

const CULL_HALF_W: f32 = PLAY_AREA_HALF_W + CULL_MARGIN;
const CULL_HALF_H: f32 = PLAY_AREA_HALF_H + CULL_MARGIN;

/// Despawns non-boss enemy entities that have moved beyond the cull boundary.
///
/// The cull boundary extends [`CULL_MARGIN`] (80 px) beyond each edge of the
/// play area. Enemies with [`Enemy::is_boss`] set to `true` are exempt and are
/// never culled by this system (bosses are managed by the boss-phase system).
///
/// Registered in [`crate::GameSystemSet::Cleanup`].
pub fn enemy_cull_system(mut commands: Commands, enemies: Query<(Entity, &Transform, &Enemy)>) {
    for (entity, tf, enemy) in &enemies {
        if enemy.is_boss {
            continue;
        }

        let pos = tf.translation.truncate();
        let out_of_bounds =
            pos.x.abs() > CULL_HALF_W || pos.y < -CULL_HALF_H || pos.y > CULL_HALF_H;

        if out_of_bounds {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_culled(x: f32, y: f32) -> bool {
        let pos = Vec2::new(x, y);
        pos.x.abs() > CULL_HALF_W || pos.y < -CULL_HALF_H || pos.y > CULL_HALF_H
    }

    /// An enemy inside the play area must not be culled.
    #[test]
    fn inside_play_area_not_culled() {
        assert!(!is_culled(0.0, 0.0));
        assert!(!is_culled(180.0, 200.0));
    }

    /// An enemy just outside the margin must be culled.
    #[test]
    fn outside_margin_is_culled() {
        assert!(is_culled(CULL_HALF_W + 1.0, 0.0));
        assert!(is_culled(-(CULL_HALF_W + 1.0), 0.0));
        assert!(is_culled(0.0, -(CULL_HALF_H + 1.0)));
        assert!(is_culled(0.0, CULL_HALF_H + 1.0));
    }

    /// An enemy exactly at the margin boundary must not yet be culled.
    #[test]
    fn at_margin_boundary_not_culled() {
        assert!(!is_culled(CULL_HALF_W, 0.0));
        assert!(!is_culled(0.0, CULL_HALF_H));
    }

    /// Cull margin must be positive so off-screen spawns have room to enter.
    #[test]
    fn cull_margin_is_positive() {
        assert!(CULL_MARGIN > 0.0);
    }
}
