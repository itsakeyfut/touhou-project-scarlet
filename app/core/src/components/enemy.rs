use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Enemy marker
// ---------------------------------------------------------------------------

/// Core component for any entity that the player can defeat.
///
/// Attach to enemy sprites, bosses, and other destructible entities.
/// HP and collision radius are stored together to keep the hot collision
/// query (`player_bullet_hit_enemy`) to a single component access.
#[derive(Component)]
pub struct Enemy {
    /// Current hit points. The entity is despawned when this reaches ≤ 0.
    pub hp: f32,
    /// Maximum hit points at spawn time (used for HP-bar rendering in Phase 08+).
    pub hp_max: f32,
    /// Radius of the circle used for bullet-collision detection (px).
    pub collision_radius: f32,
    /// Score awarded to the player when this enemy is defeated.
    ///
    /// Added to [`crate::resources::GameData::score`] via
    /// [`crate::events::EnemyDefeatedEvent`] in
    /// [`crate::systems::item::on_enemy_defeated`].
    pub score_value: u32,
    /// `true` for boss-tier entities; used to skip cull and drop-table logic.
    pub is_boss: bool,
}

impl Enemy {
    /// Creates a new enemy with the given HP and collision radius.
    ///
    /// `score_value` defaults to `0`; use [`Enemy::with_score`] when the enemy
    /// should award points on defeat. `is_boss` defaults to `false`.
    pub fn new(hp: f32, collision_radius: f32) -> Self {
        Self {
            hp,
            hp_max: hp,
            collision_radius,
            score_value: 0,
            is_boss: false,
        }
    }

    /// Creates a new enemy with an explicit score value.
    pub fn with_score(hp: f32, collision_radius: f32, score_value: u32) -> Self {
        Self {
            hp,
            hp_max: hp,
            collision_radius,
            score_value,
            is_boss: false,
        }
    }
}

// ---------------------------------------------------------------------------
// EnemyKind
// ---------------------------------------------------------------------------

/// Identifies the type of enemy, encoding its base stats.
///
/// Used both as a component on enemy entities and as a parameter in
/// [`crate::resources::SpawnEntry`] to look up default values when spawning.
///
/// | Variant | HP | Radius | Score |
/// |---|---|---|---|
/// | `Fairy` | 10 | 12 px | 100 |
/// | `Bat` | 5 | 8 px | 50 |
/// | `TallFairy` | 30 | 16 px | 300 |
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyKind {
    /// Standard enemy — small fairy. Common, relatively slow.
    Fairy,
    /// Small, fast enemy. Low HP.
    Bat,
    /// Larger variant of the fairy with higher HP and a bigger hitbox.
    TallFairy,
}

impl EnemyKind {
    /// Base (maximum) HP for this enemy type.
    pub fn base_hp(self) -> f32 {
        match self {
            Self::Fairy => 10.0,
            Self::Bat => 5.0,
            Self::TallFairy => 30.0,
        }
    }

    /// Score awarded to the player on defeat.
    pub fn score_value(self) -> u32 {
        match self {
            Self::Fairy => 100,
            Self::Bat => 50,
            Self::TallFairy => 300,
        }
    }

    /// Radius of the bullet-collision hitbox in pixels.
    pub fn collision_radius(self) -> f32 {
        match self {
            Self::Fairy => 12.0,
            Self::Bat => 8.0,
            Self::TallFairy => 16.0,
        }
    }

    /// Placeholder sprite colour used until sprite sheets are added in Phase 19.
    pub fn color(self) -> Color {
        match self {
            Self::Fairy => Color::srgb(0.5, 0.8, 1.0),
            Self::Bat => Color::srgb(0.3, 0.2, 0.5),
            Self::TallFairy => Color::srgb(0.8, 0.6, 1.0),
        }
    }
}

// ---------------------------------------------------------------------------
// EnemyMovement
// ---------------------------------------------------------------------------

/// Defines how an enemy moves each frame.
///
/// Attached as a component alongside [`Enemy`]. The
/// [`crate::systems::enemy::movement::enemy_movement_system`] reads this
/// each frame and updates the entity's [`Transform`] accordingly.
///
/// # Variants
///
/// | Variant | Description |
/// |---|---|
/// | `Linear` | Constant velocity in a fixed direction. |
/// | `LinearThenStop` | Moves at a fixed velocity, then stops after a timer expires. |
/// | `FallDown` | Moves straight down at a constant speed. |
/// | `SineWave` | Follows a sinusoidal path on top of a base velocity. |
/// | `ChasePlayer` | Homes toward the player's position at a fixed speed. |
/// | `Waypoints` | Moves through a list of world-space waypoints in order. |
#[derive(Component, Clone)]
pub enum EnemyMovement {
    /// Move at a constant velocity indefinitely.
    Linear {
        /// World-space velocity in px/s.
        velocity: Vec2,
    },

    /// Move at a constant velocity, then stop after `stop_after` seconds.
    ///
    /// The timer counts down each frame; movement halts when it reaches zero.
    LinearThenStop {
        /// World-space velocity in px/s while moving.
        velocity: Vec2,
        /// Remaining seconds before the enemy stops. Decremented each frame.
        stop_after: f32,
    },

    /// Fall straight down at a constant speed.
    ///
    /// Equivalent to `Linear { velocity: Vec2::new(0.0, -speed) }` but more
    /// readable in stage scripts and avoids the sign-convention confusion.
    FallDown {
        /// Downward speed in px/s (positive value, applied as −Y each frame).
        speed: f32,
    },

    /// Follow a sinusoidal path layered on top of a base velocity.
    ///
    /// Each frame the horizontal component is offset by
    /// `amplitude * sin(elapsed * frequency * 2π)`, producing the
    /// characteristic weaving motion.
    SineWave {
        /// Base velocity applied every frame (px/s), before the sine offset.
        base_velocity: Vec2,
        /// Peak horizontal displacement of the sine oscillation (px).
        amplitude: f32,
        /// Oscillation frequency in cycles per second (Hz).
        frequency: f32,
    },

    /// Track the player, turning toward them each frame.
    ///
    /// Direction is recalculated every frame so the enemy continuously
    /// adjusts; it is not a ballistic homing missile.
    ChasePlayer {
        /// Movement speed in px/s.
        speed: f32,
    },

    /// Move through a sequence of world-space waypoints in order.
    ///
    /// The enemy advances toward `points[current]`; when it arrives within
    /// one frame's travel distance it snaps to that point and increments
    /// `current`. Movement stops once the last waypoint is reached.
    Waypoints {
        /// Ordered list of target positions (world space).
        points: Vec<Vec2>,
        /// Travel speed in px/s.
        speed: f32,
        /// Index of the next waypoint to move toward. Incremented by the
        /// movement system; must be initialised to `0`.
        current: usize,
    },
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Every EnemyKind must have a positive HP value.
    #[test]
    fn all_enemy_kinds_have_positive_hp() {
        for kind in [EnemyKind::Fairy, EnemyKind::Bat, EnemyKind::TallFairy] {
            assert!(kind.base_hp() > 0.0, "{kind:?} base_hp must be positive");
        }
    }

    /// Every EnemyKind must have a positive collision radius.
    #[test]
    fn all_enemy_kinds_have_positive_radius() {
        for kind in [EnemyKind::Fairy, EnemyKind::Bat, EnemyKind::TallFairy] {
            assert!(
                kind.collision_radius() > 0.0,
                "{kind:?} collision_radius must be positive"
            );
        }
    }

    /// Every EnemyKind must award a non-zero score.
    #[test]
    fn all_enemy_kinds_have_nonzero_score() {
        for kind in [EnemyKind::Fairy, EnemyKind::Bat, EnemyKind::TallFairy] {
            assert!(
                kind.score_value() > 0,
                "{kind:?} score_value must be non-zero"
            );
        }
    }

    /// TallFairy should have higher HP than Fairy, which should have higher HP than Bat.
    #[test]
    fn enemy_hp_ordering() {
        assert!(EnemyKind::Bat.base_hp() < EnemyKind::Fairy.base_hp());
        assert!(EnemyKind::Fairy.base_hp() < EnemyKind::TallFairy.base_hp());
    }

    /// `Enemy::new` must set `hp_max` equal to `hp` and `is_boss` to `false`.
    #[test]
    fn enemy_new_defaults() {
        let e = Enemy::new(50.0, 10.0);
        assert_eq!(e.hp, 50.0);
        assert_eq!(e.hp_max, 50.0);
        assert!(!e.is_boss);
    }

    /// `Enemy::with_score` must preserve `hp_max` and set `is_boss` to `false`.
    #[test]
    fn enemy_with_score_defaults() {
        let e = Enemy::with_score(20.0, 8.0, 500);
        assert_eq!(e.hp_max, 20.0);
        assert_eq!(e.score_value, 500);
        assert!(!e.is_boss);
    }
}
