use bevy::prelude::*;

/// Marker component for player-fired bullets.
///
/// Attached to every bullet the player shoots. Used to distinguish
/// player bullets from enemy bullets in queries.
#[derive(Component)]
pub struct PlayerBullet {
    /// Damage dealt to an enemy on contact.
    pub damage: f32,
}

impl Default for PlayerBullet {
    fn default() -> Self {
        Self { damage: 12.0 }
    }
}

/// Velocity of a bullet in pixels per second.
///
/// Applied every frame by [`crate::systems::bullet::bullet_movement_system`].
/// Used by both player bullets and enemy bullets.
#[derive(Component)]
pub struct BulletVelocity(pub Vec2);

/// Repeating timer that controls the player's fire rate.
///
/// Attached to the [`crate::components::Player`] entity.
/// [`crate::systems::player::shoot_input_system`] ticks this timer and
/// emits [`crate::events::ShootEvent`] only when it fires.
#[derive(Component)]
pub struct ShootTimer {
    pub timer: Timer,
}

impl Default for ShootTimer {
    fn default() -> Self {
        Self {
            // 10 shots per second.
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
        }
    }
}

/// Marker component — entity is despawned when it leaves the play area.
///
/// Checked every frame by
/// [`crate::systems::bullet::despawn_out_of_bounds_system`].
/// Attach to any entity that should be cleaned up automatically
/// (player bullets, enemy bullets, items, etc.).
#[derive(Component)]
pub struct DespawnOutOfBounds;

// ---------------------------------------------------------------------------
// Enemy bullet components
// ---------------------------------------------------------------------------

/// Visual and collision variant of an enemy bullet.
///
/// Each variant has a distinct [`collision_radius`](EnemyBulletKind::collision_radius)
/// and [`color`](EnemyBulletKind::color). Bullet sprites are placeholder
/// colored rectangles until real sprites are added in Phase 19.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnemyBulletKind {
    SmallRound,
    MediumRound,
    LargeRound,
    Rice,
    Knife,
    Star,
    Bubble,
}

impl EnemyBulletKind {
    /// Radius used for circle-based collision detection (px).
    pub fn collision_radius(self) -> f32 {
        match self {
            Self::Knife => 4.0,
            Self::SmallRound => 4.0,
            Self::Rice => 5.0,
            Self::MediumRound => 7.0,
            Self::Star => 8.0,
            Self::Bubble => 9.0,
            Self::LargeRound => 11.0,
        }
    }

    /// Placeholder sprite color (will be replaced by glow shader in Phase 19).
    pub fn color(self) -> Color {
        match self {
            Self::SmallRound => Color::srgb(1.0, 0.2, 0.2),
            Self::MediumRound => Color::srgb(0.2, 0.5, 1.0),
            Self::LargeRound => Color::srgb(0.8, 0.2, 0.8),
            Self::Rice => Color::srgb(1.0, 0.8, 0.2),
            Self::Knife => Color::srgb(0.3, 1.0, 0.3),
            Self::Star => Color::srgb(1.0, 1.0, 0.2),
            Self::Bubble => Color::srgb(0.4, 0.9, 1.0),
        }
    }

    /// Sprite size derived from collision radius (diameter × 2 for visibility).
    pub fn sprite_size(self) -> Vec2 {
        Vec2::splat(self.collision_radius() * 2.0)
    }
}

/// Marker component for enemy-fired bullets.
#[derive(Component)]
pub struct EnemyBullet {
    /// Damage dealt to the player on contact.
    pub damage: u8,
}

/// Marker component for the trail ribbon child entity attached to an [`EnemyBullet`].
///
/// Spawned as a child of the bullet entity so that it is automatically
/// despawned when the parent bullet is removed. The trail mesh is a
/// `Rectangle` oriented along the bullet's travel direction.
#[derive(Component)]
pub struct BulletTrail;

/// Bullet-fire pattern attached to an enemy emitter.
///
/// Used by [`BulletEmitter`] to determine how bullets are spawned each tick.
#[derive(Clone, Debug)]
pub enum BulletPattern {
    /// Fires `count` bullets equally spaced around a full circle.
    Ring { count: u8, speed: f32 },
    /// Fires `count` bullets spread over `spread_deg` degrees aimed at the player.
    Aimed {
        count: u8,
        spread_deg: f32,
        speed: f32,
    },
    /// Fires `count` bullets spread over `spread_deg` degrees at a fixed `angle_offset`.
    Spread {
        count: u8,
        spread_deg: f32,
        speed: f32,
        /// Rotation offset in degrees from straight down.
        angle_offset: f32,
    },
    /// Fires `arms` bullets continuously rotating at `rotation_speed_deg` deg/s.
    ///
    /// Requires [`crate::systems::danmaku::emitter::SpiralState`] on the same entity.
    Spiral {
        arms: u8,
        speed: f32,
        rotation_speed_deg: f32,
    },
}

/// Attached to an enemy to make it periodically fire bullets.
///
/// The emitter ticks its `timer` every frame; when the timer fires it
/// delegates to the appropriate pattern emit function.
#[derive(Component, Clone)]
pub struct BulletEmitter {
    /// Firing pattern for this emitter.
    pub pattern: BulletPattern,
    /// Bullet variant to spawn.
    pub bullet_kind: EnemyBulletKind,
    /// Repeating timer that controls fire rate.
    pub timer: Timer,
    /// When `false` the emitter is paused and will not fire.
    pub active: bool,
}
