use bevy::prelude::*;

use crate::{
    components::{
        bullet::DespawnOutOfBounds,
        item::{ItemKind, ItemPhysics},
        player::{Player, PlayerStats},
    },
    events::EnemyDefeatedEvent,
    resources::{FragmentTracker, GameData},
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Speed at which attracted items move toward the player (px/s).
///
/// Must exceed the maximum fall speed (`200 px/s`) so attracted items
/// visibly accelerate upward toward the player.
pub const ITEM_ATTRACT_SPEED: f32 = 400.0;

/// Maximum downward fall speed for items under gravity (px/s).
const ITEM_MAX_FALL_SPEED: f32 = -200.0;

/// Radius within which the player collects an item (px).
///
/// This is the final pickup threshold; items are attracted at a larger
/// radius ([`PlayerStats::pickup_radius`]) before being collected here.
const ITEM_COLLECT_RADIUS: f32 = 8.0;

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

/// Moves items each frame according to their [`ItemPhysics`] state.
///
/// Two attraction triggers cause an item to switch from falling to tracking
/// the player:
///
/// 1. **Score-line auto-attract** — the player's Y is at or above
///    [`SCORE_LINE_Y`] (192 px). All on-screen items are pulled in.
/// 2. **Proximity attract** — the item is within [`PlayerStats::pickup_radius`]
///    of the player.
///
/// Once `attracted` is set to `true` it is never reset; the item continues
/// homing until collected or despawned.
///
/// Items that have not been attracted fall downward under constant acceleration
/// (`fall_speed`) up to a terminal velocity of [`ITEM_MAX_FALL_SPEED`].
///
/// Registered in [`crate::GameSystemSet::Movement`].
#[allow(clippy::type_complexity)]
pub fn item_movement_system(
    mut items: Query<(&mut Transform, &mut ItemPhysics)>,
    player: Query<(&Transform, &PlayerStats), (With<Player>, Without<ItemPhysics>)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    let (player_pos, auto_attract, pickup_radius) = if let Ok((player_tf, stats)) = player.single()
    {
        let pos = player_tf.translation.truncate();
        (pos, pos.y >= SCORE_LINE_Y, stats.pickup_radius)
    } else {
        return;
    };

    for (mut tf, mut physics) in &mut items {
        let item_pos = tf.translation.truncate();
        let to_player = player_pos - item_pos;
        let distance = to_player.length();

        if physics.attracted || auto_attract || distance <= pickup_radius {
            // Switch to attracted mode and home toward the player.
            physics.attracted = true;

            if distance > 0.0 {
                let dir = to_player / distance;
                // Clamp speed so this frame's displacement cannot exceed the
                // remaining distance to the player. Without this clamp, a
                // frame spike could push the item past the player and outside
                // the collect radius, causing indefinite ping-ponging.
                let max_speed = distance / dt;
                physics.velocity = dir * ITEM_ATTRACT_SPEED.min(max_speed);
            } else {
                physics.velocity = Vec2::ZERO;
            }
        } else {
            // Apply gravity: accelerate downward, cap at terminal velocity.
            physics.velocity.y =
                (physics.velocity.y - physics.fall_speed * dt).max(ITEM_MAX_FALL_SPEED);
        }

        tf.translation += (physics.velocity * dt).extend(0.0);
    }
}

/// Collects items that overlap the player and applies their effects.
///
/// An item is collected when the distance between its centre and the player
/// is ≤ [`ITEM_COLLECT_RADIUS`]. On collection:
///
/// - The appropriate effect is applied via [`apply_item`].
/// - The item entity is despawned.
///
/// Effects per [`ItemKind`]:
///
/// | Variant | Effect |
/// |---|---|
/// | `PowerSmall` | `power += 1` (max 128) |
/// | `PowerLarge` | `power += 8` (max 128) |
/// | `FullPower`  | `power = 128` |
/// | `PointItem`  | `score += calc_point_item_value(player_y)` |
/// | `LifeFragment` | fragment counter +1; at 5 → `lives += 1`, counter reset |
/// | `BombFragment` | fragment counter +1; at 5 → `bombs = (bombs+1).min(3)`, counter reset |
///
/// Registered in [`crate::GameSystemSet::GameLogic`].
pub fn item_collection_system(
    mut commands: Commands,
    items: Query<(Entity, &Transform, &ItemKind)>,
    player: Query<(&Transform, &PlayerStats), With<Player>>,
    mut game_data: ResMut<GameData>,
    mut tracker: ResMut<FragmentTracker>,
) {
    let Ok((player_tf, stats)) = player.single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();
    let player_y = player_tf.translation.y;

    // Use the larger of hitbox_radius and ITEM_COLLECT_RADIUS so the pickup
    // zone always covers at least the defined constant.
    let collect_radius = ITEM_COLLECT_RADIUS.max(stats.hitbox_radius);

    for (entity, tf, &kind) in &items {
        let item_pos = tf.translation.truncate();
        if item_pos.distance(player_pos) > collect_radius {
            continue;
        }
        apply_item(&mut game_data, &mut tracker, kind, player_y);
        commands.entity(entity).despawn();
    }
}

/// Applies the gameplay effect of collecting an item and returns the score awarded.
///
/// Called by [`item_collection_system`] for each collected item.
fn apply_item(
    game_data: &mut GameData,
    tracker: &mut FragmentTracker,
    kind: ItemKind,
    player_y: f32,
) {
    match kind {
        ItemKind::PowerSmall => {
            game_data.power = game_data.power.saturating_add(1).min(128);
        }
        ItemKind::PowerLarge => {
            game_data.power = game_data.power.saturating_add(8).min(128);
        }
        ItemKind::FullPower => {
            game_data.power = 128;
        }
        ItemKind::PointItem => {
            let value = calc_point_item_value(player_y);
            game_data.score += value as u64;
        }
        ItemKind::LifeFragment => {
            tracker.life_fragments = tracker.life_fragments.saturating_add(1);
        }
        ItemKind::BombFragment => {
            tracker.bomb_fragments = tracker.bomb_fragments.saturating_add(1);
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

    // ---- apply_item ---------------------------------------------------------

    fn make_game_data() -> GameData {
        GameData {
            score: 0,
            hi_score: 0,
            lives: 2,
            bombs: 3,
            power: 0,
            graze: 0,
        }
    }

    /// PowerSmall must add exactly 1 power.
    #[test]
    fn power_small_adds_one() {
        let mut gd = make_game_data();
        let mut tracker = FragmentTracker::default();
        apply_item(&mut gd, &mut tracker, ItemKind::PowerSmall, 0.0);
        assert_eq!(gd.power, 1);
    }

    /// PowerLarge must add exactly 8 power.
    #[test]
    fn power_large_adds_eight() {
        let mut gd = make_game_data();
        let mut tracker = FragmentTracker::default();
        apply_item(&mut gd, &mut tracker, ItemKind::PowerLarge, 0.0);
        assert_eq!(gd.power, 8);
    }

    /// Power must not exceed 128.
    #[test]
    fn power_caps_at_128() {
        let mut gd = make_game_data();
        gd.power = 127;
        let mut tracker = FragmentTracker::default();
        apply_item(&mut gd, &mut tracker, ItemKind::PowerLarge, 0.0);
        assert_eq!(gd.power, 128);
    }

    /// FullPower must set power to exactly 128 regardless of current value.
    #[test]
    fn full_power_sets_max() {
        let mut gd = make_game_data();
        gd.power = 50;
        let mut tracker = FragmentTracker::default();
        apply_item(&mut gd, &mut tracker, ItemKind::FullPower, 0.0);
        assert_eq!(gd.power, 128);
    }

    /// PointItem at score line must add POI_BASE_VALUE to score.
    #[test]
    fn point_item_at_score_line_awards_max_score() {
        let mut gd = make_game_data();
        let mut tracker = FragmentTracker::default();
        apply_item(&mut gd, &mut tracker, ItemKind::PointItem, SCORE_LINE_Y);
        assert_eq!(gd.score, POI_BASE_VALUE as u64);
    }

    /// Four LifeFragments must increment the counter to 4; no extend occurs
    /// (extend is handled by [`crate::systems::score::check_extend_system`]).
    #[test]
    fn four_life_fragments_increments_counter() {
        let mut gd = make_game_data();
        let mut tracker = FragmentTracker::default();
        for _ in 0..4 {
            apply_item(&mut gd, &mut tracker, ItemKind::LifeFragment, 0.0);
        }
        assert_eq!(gd.lives, 2, "apply_item must not extend lives");
        assert_eq!(tracker.life_fragments, 4);
    }

    /// Five LifeFragments must set the counter to 5; extend is deferred to
    /// [`crate::systems::score::check_extend_system`].
    #[test]
    fn five_life_fragments_increments_counter() {
        let mut gd = make_game_data();
        let mut tracker = FragmentTracker::default();
        for _ in 0..5 {
            apply_item(&mut gd, &mut tracker, ItemKind::LifeFragment, 0.0);
        }
        assert_eq!(gd.lives, 2, "apply_item must not extend lives");
        assert_eq!(tracker.life_fragments, 5);
    }

    /// Five BombFragments must set the counter to 5; extend is deferred to
    /// [`crate::systems::score::check_extend_system`].
    #[test]
    fn five_bomb_fragments_increments_counter() {
        let mut gd = make_game_data();
        let mut tracker = FragmentTracker::default();
        for _ in 0..5 {
            apply_item(&mut gd, &mut tracker, ItemKind::BombFragment, 0.0);
        }
        assert_eq!(gd.bombs, 3, "apply_item must not modify bombs");
        assert_eq!(tracker.bomb_fragments, 5);
    }
}
