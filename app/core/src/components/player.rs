use bevy::prelude::*;

/// Marker component that identifies the player entity.
#[derive(Component, Default)]
pub struct Player;

/// Core stats that control player movement and collision radii.
///
/// Values are tuned to match the original EoSD feel:
/// - Normal speed: 200 px/s
/// - Focus speed:  100 px/s (Shift held)
/// - Hitbox: 2.0 px radius (tight, as in the original)
/// - Graze:  16 px radius
#[derive(Component)]
pub struct PlayerStats {
    /// Movement speed in pixels per second (normal mode).
    pub speed: f32,
    /// Movement speed in pixels per second while Shift is held (focus/slow mode).
    pub slow_speed: f32,
    /// Radius of the bullet-collision hitbox in pixels.
    pub hitbox_radius: f32,
    /// Radius of the graze detection zone in pixels.
    pub graze_radius: f32,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            speed: 200.0,
            slow_speed: 100.0,
            hitbox_radius: 2.0,
            graze_radius: 16.0,
        }
    }
}

/// Tracks remaining invincibility frames after a player hit.
#[derive(Component)]
pub struct InvincibilityTimer {
    pub timer: Timer,
}

/// Marker component for the graze-field visual child entity.
///
/// Spawned as a `Mesh2d(Circle::new(16.0))` child of the player entity by
/// [`crate::shaders::plugin::setup_graze_visual`]. The mesh uses
/// [`crate::shaders::GrazeMaterial`] which renders the electric ring effect.
#[derive(Component)]
pub struct GrazeVisual;
