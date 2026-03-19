use bevy::prelude::*;

use crate::{
    components::{
        bullet::DespawnOutOfBounds,
        item::{ItemKind, ItemPhysics},
    },
    events::EnemyDefeatedEvent,
    resources::GameData,
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Y coordinate of the "score line" (px above play area centre).
///
/// When the player is at or above this height all items on screen are
/// automatically attracted toward the player, and point items yield their
/// maximum value.
const SCORE_LINE_Y: f32 = 192.0;

/// Maximum score awarded by a point item (player at or above the score line).
pub const POI_BASE_VALUE: u32 = 10_000;

/// Minimum score awarded by a point item (player at the bottom of the play area).
pub const POI_MIN_VALUE: u32 = 100;

// ---------------------------------------------------------------------------
// Item spawning
// ---------------------------------------------------------------------------

/// Spawns a single collectible item entity at `pos` with the given `kind`.
///
/// The entity is given:
/// - [`ItemKind`] — identifies the effect applied on pickup.
/// - [`ItemPhysics`] — initial falling physics state.
/// - [`Sprite`] — placeholder coloured square; replaced by sprite sheets in Phase 19.
/// - [`Transform`] — positioned at `pos` with a fixed Z of `1.2` (above the
///   play field but below bullets at Z `1.5`).
/// - [`DespawnOutOfBounds`] — automatically removed when it falls off-screen.
pub fn spawn_item(commands: &mut Commands, pos: Vec2, kind: ItemKind) {
    commands.spawn((
        kind,
        ItemPhysics::default(),
        Sprite {
            color: kind.color(),
            custom_size: Some(kind.sprite_size()),
            ..default()
        },
        Transform::from_translation(pos.extend(1.2)),
        DespawnOutOfBounds,
    ));
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Drops items and awards score when an enemy is defeated.
///
/// Reads [`EnemyDefeatedEvent`]s emitted by
/// [`crate::systems::collision::player_bullet_hit_enemy`] in the same frame
/// (this system runs in [`crate::GameSystemSet::GameLogic`] which executes
/// after [`crate::GameSystemSet::Collision`]).
///
/// # Drop tables
///
/// | Enemy tier | Drop |
/// |---|---|
/// | Normal (`is_boss = false`) | 1× [`ItemKind::PointItem`] + 1× [`ItemKind::PowerSmall`] |
/// | Boss (`is_boss = true`) | 3× [`ItemKind::PointItem`] + 1× [`ItemKind::PowerLarge`] |
///
/// Registered in [`crate::GameSystemSet::GameLogic`].
pub fn on_enemy_defeated(
    mut commands: Commands,
    mut defeated_events: MessageReader<EnemyDefeatedEvent>,
    mut game_data: ResMut<GameData>,
) {
    for event in defeated_events.read() {
        game_data.score += event.score as u64;

        let pos = event.position;

        if event.is_boss {
            // Boss: three point items spread horizontally + a large power item.
            spawn_item(
                &mut commands,
                pos + Vec2::new(-16.0, 0.0),
                ItemKind::PointItem,
            );
            spawn_item(&mut commands, pos, ItemKind::PointItem);
            spawn_item(
                &mut commands,
                pos + Vec2::new(16.0, 0.0),
                ItemKind::PointItem,
            );
            spawn_item(
                &mut commands,
                pos + Vec2::new(0.0, 8.0),
                ItemKind::PowerLarge,
            );
        } else {
            // Normal enemy: one point item + one small power item.
            spawn_item(&mut commands, pos, ItemKind::PointItem);
            spawn_item(
                &mut commands,
                pos + Vec2::new(-8.0, 0.0),
                ItemKind::PowerSmall,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Score calculation
// ---------------------------------------------------------------------------

/// Calculates the score value of a [`ItemKind::PointItem`] based on the
/// player's current Y position.
///
/// Returns [`POI_BASE_VALUE`] when the player is at or above [`SCORE_LINE_Y`],
/// and linearly interpolates down to [`POI_MIN_VALUE`] at the bottom of the
/// play area.
///
/// # Arguments
///
/// * `player_y` — the player's world-space Y coordinate.
///
/// # Examples
///
/// ```rust
/// use scarlet_core::systems::item::{calc_point_item_value, POI_BASE_VALUE, POI_MIN_VALUE};
///
/// // At the score line → maximum value.
/// assert_eq!(calc_point_item_value(192.0), POI_BASE_VALUE);
/// ```
pub fn calc_point_item_value(player_y: f32) -> u32 {
    // At or above the score line → always maximum.
    if player_y >= SCORE_LINE_Y {
        return POI_BASE_VALUE;
    }

    // Linear interpolation between the bottom of the play area and the score
    // line.  The bottom of the play area is -PLAY_AREA_HALF_H (= -224).
    const PLAY_AREA_BOTTOM: f32 = -224.0;
    let range = SCORE_LINE_Y - PLAY_AREA_BOTTOM; // 416
    let t = ((player_y - PLAY_AREA_BOTTOM) / range).clamp(0.0, 1.0);

    let span = (POI_BASE_VALUE - POI_MIN_VALUE) as f32;
    (POI_MIN_VALUE as f32 + span * t) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- calc_point_item_value -------------------------------------------

    /// At the score line the maximum value must be returned.
    #[test]
    fn point_item_at_score_line_gives_max() {
        assert_eq!(calc_point_item_value(SCORE_LINE_Y), POI_BASE_VALUE);
    }

    /// Above the score line must also give the maximum value.
    #[test]
    fn point_item_above_score_line_gives_max() {
        assert_eq!(calc_point_item_value(SCORE_LINE_Y + 50.0), POI_BASE_VALUE);
    }

    /// At the very bottom of the play area the minimum value must be returned.
    #[test]
    fn point_item_at_bottom_gives_min() {
        assert_eq!(calc_point_item_value(-224.0), POI_MIN_VALUE);
    }

    /// A Y position below the play area must clamp to the minimum value.
    #[test]
    fn point_item_below_play_area_clamps_to_min() {
        assert_eq!(calc_point_item_value(-500.0), POI_MIN_VALUE);
    }

    /// The value at the midpoint between the bottom and the score line must be
    /// strictly between the min and max.
    #[test]
    fn point_item_at_midpoint_is_between_min_and_max() {
        let mid = (-224.0 + SCORE_LINE_Y) / 2.0;
        let v = calc_point_item_value(mid);
        assert!(v > POI_MIN_VALUE, "midpoint value should exceed minimum");
        assert!(v < POI_BASE_VALUE, "midpoint value should be below maximum");
    }

    /// The value must be monotonically non-decreasing as Y increases toward the
    /// score line.
    #[test]
    fn point_item_value_is_monotone() {
        let ys: Vec<f32> = (-10..=10).map(|i| i as f32 * 20.0).collect();
        let values: Vec<u32> = ys.iter().copied().map(calc_point_item_value).collect();
        for pair in values.windows(2) {
            assert!(
                pair[0] <= pair[1],
                "value must be non-decreasing: {} > {}",
                pair[0],
                pair[1]
            );
        }
    }
}
