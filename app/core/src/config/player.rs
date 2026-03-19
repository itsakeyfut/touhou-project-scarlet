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
pub(crate) const DEFAULT_INVINCIBILITY_SECS: f32 = 3.0;
const DEFAULT_INITIAL_LIVES: u8 = 2;
const DEFAULT_INITIAL_BOMBS: u8 = 3;
const DEFAULT_INITIAL_POWER: u8 = 0;

// ---------------------------------------------------------------------------
// Partial (deserialization mirror)
// ---------------------------------------------------------------------------

/// Deserialization mirror of [`PlayerConfig`] — every field is `Option<T>` so
/// RON files with missing fields still load and emit a `warn!` instead of failing.
#[derive(Deserialize, Default)]
#[serde(default, rename = "PlayerConfig")]
pub(super) struct PlayerConfigPartial {
    pub speed: Option<f32>,
    pub slow_speed: Option<f32>,
    pub hitbox_radius: Option<f32>,
    pub graze_radius: Option<f32>,
    pub pickup_radius: Option<f32>,
    pub shoot_interval_secs: Option<f32>,
    pub bullet_damage: Option<f32>,
    pub invincibility_secs: Option<f32>,
    pub initial_lives: Option<u8>,
    pub initial_bombs: Option<u8>,
    pub initial_power: Option<u8>,
}

// ---------------------------------------------------------------------------
// Asset type
// ---------------------------------------------------------------------------

/// Player stats and starting values, loaded from `assets/config/player.ron`.
///
/// Hot-reloading this file takes effect the next time the player entity is
/// spawned; in-progress runs are unaffected.
#[derive(Asset, TypePath, Debug, Clone)]
pub struct PlayerConfig {
    /// Normal movement speed (pixels / second).
    pub speed: f32,
    /// Focus (slow) movement speed while Shift is held (pixels / second).
    pub slow_speed: f32,
    /// Bullet hitbox radius (pixels).
    pub hitbox_radius: f32,
    /// Graze detection radius (pixels).
    pub graze_radius: f32,
    /// Item-attraction trigger radius (pixels).
    pub pickup_radius: f32,
    /// Seconds between consecutive shots.
    pub shoot_interval_secs: f32,
    /// Damage per player bullet.
    pub bullet_damage: f32,
    /// Duration of post-hit invincibility (seconds).
    pub invincibility_secs: f32,
    /// Lives at the start of a new game.
    pub initial_lives: u8,
    /// Bomb stocks at the start of a new game.
    pub initial_bombs: u8,
    /// Power level at the start of a new game (0–128).
    pub initial_power: u8,
}

impl From<PlayerConfigPartial> for PlayerConfig {
    fn from(p: PlayerConfigPartial) -> Self {
        PlayerConfig {
            speed: p.speed.unwrap_or_else(|| {
                warn!("player.ron: `speed` missing → using default {DEFAULT_SPEED}");
                DEFAULT_SPEED
            }),
            slow_speed: p.slow_speed.unwrap_or_else(|| {
                warn!("player.ron: `slow_speed` missing → using default {DEFAULT_SLOW_SPEED}");
                DEFAULT_SLOW_SPEED
            }),
            hitbox_radius: p.hitbox_radius.unwrap_or_else(|| {
                warn!(
                    "player.ron: `hitbox_radius` missing → using default {DEFAULT_HITBOX_RADIUS}"
                );
                DEFAULT_HITBOX_RADIUS
            }),
            graze_radius: p.graze_radius.unwrap_or_else(|| {
                warn!(
                    "player.ron: `graze_radius` missing → using default {DEFAULT_GRAZE_RADIUS}"
                );
                DEFAULT_GRAZE_RADIUS
            }),
            pickup_radius: p.pickup_radius.unwrap_or_else(|| {
                warn!(
                    "player.ron: `pickup_radius` missing → using default {DEFAULT_PICKUP_RADIUS}"
                );
                DEFAULT_PICKUP_RADIUS
            }),
            shoot_interval_secs: p.shoot_interval_secs.unwrap_or_else(|| {
                warn!(
                    "player.ron: `shoot_interval_secs` missing → using default {DEFAULT_SHOOT_INTERVAL_SECS}"
                );
                DEFAULT_SHOOT_INTERVAL_SECS
            }),
            bullet_damage: p.bullet_damage.unwrap_or_else(|| {
                warn!(
                    "player.ron: `bullet_damage` missing → using default {DEFAULT_BULLET_DAMAGE}"
                );
                DEFAULT_BULLET_DAMAGE
            }),
            invincibility_secs: p.invincibility_secs.unwrap_or_else(|| {
                warn!(
                    "player.ron: `invincibility_secs` missing → using default {DEFAULT_INVINCIBILITY_SECS}"
                );
                DEFAULT_INVINCIBILITY_SECS
            }),
            initial_lives: p.initial_lives.unwrap_or_else(|| {
                warn!(
                    "player.ron: `initial_lives` missing → using default {DEFAULT_INITIAL_LIVES}"
                );
                DEFAULT_INITIAL_LIVES
            }),
            initial_bombs: p.initial_bombs.unwrap_or_else(|| {
                warn!(
                    "player.ron: `initial_bombs` missing → using default {DEFAULT_INITIAL_BOMBS}"
                );
                DEFAULT_INITIAL_BOMBS
            }),
            initial_power: p.initial_power.unwrap_or_else(|| {
                warn!(
                    "player.ron: `initial_power` missing → using default {DEFAULT_INITIAL_POWER}"
                );
                DEFAULT_INITIAL_POWER
            }),
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
/// Individual getter methods (`.speed()`, `.hitbox_radius()`, …) always
/// return a usable value by falling back to the `DEFAULT_*` constants.
///
/// # Example
/// ```ignore
/// pub fn spawn_player(mut commands: Commands, cfg: PlayerConfigParams) {
///     commands.spawn(PlayerStats {
///         speed: cfg.speed(),
///         hitbox_radius: cfg.hitbox_radius(),
///         // …
///     });
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

    pub fn speed(&self) -> f32 {
        self.get().map(|c| c.speed).unwrap_or(DEFAULT_SPEED)
    }

    pub fn slow_speed(&self) -> f32 {
        self.get()
            .map(|c| c.slow_speed)
            .unwrap_or(DEFAULT_SLOW_SPEED)
    }

    pub fn hitbox_radius(&self) -> f32 {
        self.get()
            .map(|c| c.hitbox_radius)
            .unwrap_or(DEFAULT_HITBOX_RADIUS)
    }

    pub fn graze_radius(&self) -> f32 {
        self.get()
            .map(|c| c.graze_radius)
            .unwrap_or(DEFAULT_GRAZE_RADIUS)
    }

    pub fn pickup_radius(&self) -> f32 {
        self.get()
            .map(|c| c.pickup_radius)
            .unwrap_or(DEFAULT_PICKUP_RADIUS)
    }

    pub fn shoot_interval_secs(&self) -> f32 {
        self.get()
            .map(|c| c.shoot_interval_secs)
            .unwrap_or(DEFAULT_SHOOT_INTERVAL_SECS)
    }

    pub fn bullet_damage(&self) -> f32 {
        self.get()
            .map(|c| c.bullet_damage)
            .unwrap_or(DEFAULT_BULLET_DAMAGE)
    }

    pub fn invincibility_secs(&self) -> f32 {
        self.get()
            .map(|c| c.invincibility_secs)
            .unwrap_or(DEFAULT_INVINCIBILITY_SECS)
    }

    pub fn initial_lives(&self) -> u8 {
        self.get()
            .map(|c| c.initial_lives)
            .unwrap_or(DEFAULT_INITIAL_LIVES)
    }

    pub fn initial_bombs(&self) -> u8 {
        self.get()
            .map(|c| c.initial_bombs)
            .unwrap_or(DEFAULT_INITIAL_BOMBS)
    }

    pub fn initial_power(&self) -> u8 {
        self.get()
            .map(|c| c.initial_power)
            .unwrap_or(DEFAULT_INITIAL_POWER)
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

    fn make_config_from_ron(src: &str) -> PlayerConfig {
        let options = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME);
        let partial: PlayerConfigPartial = options.from_str(src).unwrap();
        PlayerConfig::from(partial)
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
        let cfg = make_config_from_ron(src);
        assert_eq!(cfg.speed, 250.0);
        assert_eq!(cfg.slow_speed, 120.0);
        assert_eq!(cfg.hitbox_radius, 3.0);
        assert_eq!(cfg.graze_radius, 20.0);
        assert_eq!(cfg.pickup_radius, 30.0);
        assert_eq!(cfg.shoot_interval_secs, 0.08);
        assert_eq!(cfg.bullet_damage, 15.0);
        assert_eq!(cfg.invincibility_secs, 2.5);
        assert_eq!(cfg.initial_lives, 3);
        assert_eq!(cfg.initial_bombs, 2);
        assert_eq!(cfg.initial_power, 0);
    }

    #[test]
    fn ron_deserialization_partial_uses_defaults() {
        // Only one field specified — rest must fall back to DEFAULT_* values.
        let src = r#"PlayerConfig(speed: 300.0)"#;
        let cfg = make_config_from_ron(src);
        assert_eq!(cfg.speed, 300.0);
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
    fn empty_ron_falls_back_to_all_defaults() {
        let src = r#"PlayerConfig()"#;
        let cfg = make_config_from_ron(src);
        assert_eq!(cfg.speed, DEFAULT_SPEED);
        assert_eq!(cfg.slow_speed, DEFAULT_SLOW_SPEED);
        assert_eq!(cfg.initial_lives, DEFAULT_INITIAL_LIVES);
    }
}
