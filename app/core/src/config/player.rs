//! Player configuration loaded from `assets/config/player.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Fallback constants (used while player.ron is still loading)
// ---------------------------------------------------------------------------

const DEFAULT_SPEED: f32 = 200.0;
const DEFAULT_SLOW_SPEED: f32 = 100.0;
const DEFAULT_HITBOX_RADIUS: f32 = 2.0;
const DEFAULT_GRAZE_RADIUS: f32 = 16.0;
const DEFAULT_PICKUP_RADIUS: f32 = 24.0;
const DEFAULT_SHOOT_INTERVAL_SECS: f32 = 0.1;
const DEFAULT_BULLET_DAMAGE: f32 = 12.0;
/// Invincibility window after being hit (seconds).
/// Used by [`crate::systems::collision`] when inserting `InvincibilityTimer`.
pub(crate) const DEFAULT_INVINCIBILITY_SECS: f32 = 3.0;
const DEFAULT_INITIAL_LIVES: u8 = 2;
const DEFAULT_INITIAL_BOMBS: u8 = 3;
const DEFAULT_INITIAL_POWER: u8 = 0;

// ---------------------------------------------------------------------------
// Asset type
// ---------------------------------------------------------------------------

/// Player stats and starting values, loaded from `assets/config/player.ron`.
///
/// All fields have game-ready defaults so the game runs correctly even if the
/// RON file has not loaded yet.  [`PlayerConfigParams`] always returns a valid
/// value by falling back to these defaults.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct PlayerConfig {
    // Movement
    /// Normal movement speed (pixels / second).
    pub speed: f32,
    /// Focus (slow) movement speed while Shift is held (pixels / second).
    pub slow_speed: f32,
    // Collision radii
    /// Bullet hitbox radius (pixels).
    pub hitbox_radius: f32,
    /// Graze detection radius (pixels).
    pub graze_radius: f32,
    /// Item-attraction trigger radius (pixels).
    pub pickup_radius: f32,
    // Shooting
    /// Seconds between consecutive shots.
    pub shoot_interval_secs: f32,
    /// Damage per player bullet.
    pub bullet_damage: f32,
    // After-hit invincibility
    /// Duration of post-hit invincibility (seconds).
    pub invincibility_secs: f32,
    // Starting values for a new game
    pub initial_lives: u8,
    pub initial_bombs: u8,
    pub initial_power: u8,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            speed: DEFAULT_SPEED,
            slow_speed: DEFAULT_SLOW_SPEED,
            hitbox_radius: DEFAULT_HITBOX_RADIUS,
            graze_radius: DEFAULT_GRAZE_RADIUS,
            pickup_radius: DEFAULT_PICKUP_RADIUS,
            shoot_interval_secs: DEFAULT_SHOOT_INTERVAL_SECS,
            bullet_damage: DEFAULT_BULLET_DAMAGE,
            invincibility_secs: DEFAULT_INVINCIBILITY_SECS,
            initial_lives: DEFAULT_INITIAL_LIVES,
            initial_bombs: DEFAULT_INITIAL_BOMBS,
            initial_power: DEFAULT_INITIAL_POWER,
        }
    }
}

// ---------------------------------------------------------------------------
// Handle resource
// ---------------------------------------------------------------------------

/// Resource that keeps the [`PlayerConfig`] asset handle alive.
#[derive(Resource)]
pub struct PlayerConfigHandle(pub Handle<PlayerConfig>);

// ---------------------------------------------------------------------------
// SystemParam bundle
// ---------------------------------------------------------------------------

/// Convenience accessor for [`PlayerConfig`].
///
/// Returns `None` from `.get()` while the asset is still loading.
/// `.get_or_default()` always returns a usable value.
///
/// # Example
/// ```ignore
/// pub fn spawn_player(mut commands: Commands, cfg: PlayerConfigParams) {
///     let player_cfg = cfg.get_or_default();
///     // …
/// }
/// ```
#[derive(SystemParam)]
pub struct PlayerConfigParams<'w> {
    handle: Option<Res<'w, PlayerConfigHandle>>,
    assets: Option<Res<'w, Assets<PlayerConfig>>>,
}

impl<'w> PlayerConfigParams<'w> {
    /// Returns the loaded [`PlayerConfig`], or `None` while loading.
    pub fn get(&self) -> Option<&PlayerConfig> {
        self.handle
            .as_ref()
            .and_then(|h| self.assets.as_ref().and_then(|a| a.get(&h.0)))
    }

    /// Returns the loaded config, or [`PlayerConfig::default`] as a fallback.
    pub fn get_or_default(&self) -> PlayerConfig {
        self.get().cloned().unwrap_or_default()
    }
}

// ---------------------------------------------------------------------------
// Hot-reload system
// ---------------------------------------------------------------------------

/// Logs asset lifecycle events for [`PlayerConfig`].
///
/// Full hot-reload of live player entities will be wired in a later phase
/// once sprite components are attached.
pub fn hot_reload_player_config(mut events: MessageReader<AssetEvent<PlayerConfig>>) {
    for event in events.read() {
        match event {
            AssetEvent::Added { .. } => info!("PlayerConfig loaded"),
            AssetEvent::Modified { .. } => info!("PlayerConfig modified (hot-reload)"),
            AssetEvent::Removed { .. } => warn!("PlayerConfig removed"),
            _ => {}
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
    fn default_matches_constants() {
        let cfg = PlayerConfig::default();
        assert_eq!(cfg.speed, DEFAULT_SPEED);
        assert_eq!(cfg.slow_speed, DEFAULT_SLOW_SPEED);
        assert_eq!(cfg.hitbox_radius, DEFAULT_HITBOX_RADIUS);
        assert_eq!(cfg.graze_radius, DEFAULT_GRAZE_RADIUS);
        assert_eq!(cfg.pickup_radius, DEFAULT_PICKUP_RADIUS);
        assert_eq!(cfg.shoot_interval_secs, DEFAULT_SHOOT_INTERVAL_SECS);
        assert_eq!(cfg.bullet_damage, DEFAULT_BULLET_DAMAGE);
        assert_eq!(cfg.invincibility_secs, DEFAULT_INVINCIBILITY_SECS);
        assert_eq!(cfg.initial_lives, DEFAULT_INITIAL_LIVES);
        assert_eq!(cfg.initial_bombs, DEFAULT_INITIAL_BOMBS);
        assert_eq!(cfg.initial_power, DEFAULT_INITIAL_POWER);
    }

    #[test]
    fn ron_deserialization_full() {
        let src = r#"
PlayerConfig(
    speed: 250.0,
    slow_speed: 120.0,
    hitbox_radius: 3.0,
    graze_radius: 20.0,
    pickup_radius: 30.0,
    shoot_interval_secs: 0.08,
    bullet_damage: 15.0,
    invincibility_secs: 2.5,
    initial_lives: 3,
    initial_bombs: 2,
    initial_power: 0,
)
"#;
        let cfg: PlayerConfig = ron::from_str(src).unwrap();
        assert_eq!(cfg.speed, 250.0);
        assert_eq!(cfg.slow_speed, 120.0);
        assert_eq!(cfg.initial_lives, 3);
        assert_eq!(cfg.shoot_interval_secs, 0.08);
    }

    #[test]
    fn ron_deserialization_partial_uses_defaults() {
        // Only override one field — remaining fields should fall back to serde defaults.
        let src = r#"PlayerConfig(speed: 300.0)"#;
        let cfg: PlayerConfig = ron::from_str(src).unwrap();
        assert_eq!(cfg.speed, 300.0);
        assert_eq!(cfg.slow_speed, DEFAULT_SLOW_SPEED);
        assert_eq!(cfg.hitbox_radius, DEFAULT_HITBOX_RADIUS);
    }
}
