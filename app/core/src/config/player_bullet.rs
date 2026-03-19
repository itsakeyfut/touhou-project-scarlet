//! Player-bullet configuration loaded from `assets/config/bullets/player.ron`.
//!
//! Covers movement speed, spread, spawn offset, collision radius, sprite
//! dimensions, and damage for bullets fired by the player.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Fallback constants (used while player.ron is still loading)
// ---------------------------------------------------------------------------

/// Forward speed of a player bullet (px/s).
pub(crate) const DEFAULT_PLAYER_BULLET_SPEED: f32 = 600.0;
/// Total horizontal spread across the bullet fan (px).
pub(crate) const DEFAULT_PLAYER_BULLET_SPREAD: f32 = 10.0;
/// Multiplier applied to the per-bullet x-offset to produce horizontal velocity.
pub(crate) const DEFAULT_PLAYER_BULLET_SPREAD_SPEED_SCALE: f32 = 5.0;
/// Y distance above the player origin where bullets are spawned (px).
pub(crate) const DEFAULT_PLAYER_BULLET_ORIGIN_Y_OFFSET: f32 = 16.0;
/// Radius used for player-bullet → enemy circle-collision detection (px).
pub(crate) const DEFAULT_PLAYER_BULLET_COLLISION_RADIUS: f32 = 3.0;
/// Sprite width of a player bullet (px).
pub(crate) const DEFAULT_PLAYER_BULLET_SPRITE_WIDTH: f32 = 4.0;
/// Sprite height of a player bullet (px).
pub(crate) const DEFAULT_PLAYER_BULLET_SPRITE_HEIGHT: f32 = 12.0;
/// Damage dealt to an enemy on contact.
pub(crate) const DEFAULT_PLAYER_BULLET_DAMAGE: f32 = 12.0;

// ---------------------------------------------------------------------------
// Partial (deserialization mirror)
// ---------------------------------------------------------------------------

/// Deserialization mirror of [`PlayerBulletConfig`] — every field is `Option<T>`
/// so RON files with missing fields still load and emit a `warn!` instead of failing.
#[derive(Deserialize, Default)]
#[serde(default, rename = "PlayerBulletConfig")]
pub(super) struct PlayerBulletConfigPartial {
    pub speed: Option<f32>,
    pub spread: Option<f32>,
    pub spread_speed_scale: Option<f32>,
    pub origin_y_offset: Option<f32>,
    pub collision_radius: Option<f32>,
    pub sprite_width: Option<f32>,
    pub sprite_height: Option<f32>,
    pub damage: Option<f32>,
}

// ---------------------------------------------------------------------------
// Asset type
// ---------------------------------------------------------------------------

/// Player-bullet parameters loaded from `assets/config/bullets/player.ron`.
#[derive(Asset, TypePath, Debug, Clone)]
pub struct PlayerBulletConfig {
    /// Forward speed of a player bullet (px/s).
    pub speed: f32,
    /// Total horizontal spread across the bullet fan (px).
    pub spread: f32,
    /// Multiplier applied to per-bullet x-offset to produce horizontal velocity.
    pub spread_speed_scale: f32,
    /// Y distance above the player where bullets are spawned (px).
    pub origin_y_offset: f32,
    /// Radius used for circle-collision detection against enemies (px).
    pub collision_radius: f32,
    /// Sprite width (px).
    pub sprite_width: f32,
    /// Sprite height (px).
    pub sprite_height: f32,
    /// Damage dealt to an enemy on contact.
    pub damage: f32,
}

impl From<PlayerBulletConfigPartial> for PlayerBulletConfig {
    fn from(p: PlayerBulletConfigPartial) -> Self {
        PlayerBulletConfig {
            speed: p.speed.unwrap_or_else(|| {
                warn!(
                    "bullets/player.ron: `speed` missing → using default {DEFAULT_PLAYER_BULLET_SPEED}"
                );
                DEFAULT_PLAYER_BULLET_SPEED
            }),
            spread: p.spread.unwrap_or_else(|| {
                warn!(
                    "bullets/player.ron: `spread` missing → using default {DEFAULT_PLAYER_BULLET_SPREAD}"
                );
                DEFAULT_PLAYER_BULLET_SPREAD
            }),
            spread_speed_scale: p.spread_speed_scale.unwrap_or_else(|| {
                warn!(
                    "bullets/player.ron: `spread_speed_scale` missing → using default {DEFAULT_PLAYER_BULLET_SPREAD_SPEED_SCALE}"
                );
                DEFAULT_PLAYER_BULLET_SPREAD_SPEED_SCALE
            }),
            origin_y_offset: p.origin_y_offset.unwrap_or_else(|| {
                warn!(
                    "bullets/player.ron: `origin_y_offset` missing → using default {DEFAULT_PLAYER_BULLET_ORIGIN_Y_OFFSET}"
                );
                DEFAULT_PLAYER_BULLET_ORIGIN_Y_OFFSET
            }),
            collision_radius: p.collision_radius.unwrap_or_else(|| {
                warn!(
                    "bullets/player.ron: `collision_radius` missing → using default {DEFAULT_PLAYER_BULLET_COLLISION_RADIUS}"
                );
                DEFAULT_PLAYER_BULLET_COLLISION_RADIUS
            }),
            sprite_width: p.sprite_width.unwrap_or_else(|| {
                warn!(
                    "bullets/player.ron: `sprite_width` missing → using default {DEFAULT_PLAYER_BULLET_SPRITE_WIDTH}"
                );
                DEFAULT_PLAYER_BULLET_SPRITE_WIDTH
            }),
            sprite_height: p.sprite_height.unwrap_or_else(|| {
                warn!(
                    "bullets/player.ron: `sprite_height` missing → using default {DEFAULT_PLAYER_BULLET_SPRITE_HEIGHT}"
                );
                DEFAULT_PLAYER_BULLET_SPRITE_HEIGHT
            }),
            damage: p.damage.unwrap_or_else(|| {
                warn!(
                    "bullets/player.ron: `damage` missing → using default {DEFAULT_PLAYER_BULLET_DAMAGE}"
                );
                DEFAULT_PLAYER_BULLET_DAMAGE
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// Handle resource
// ---------------------------------------------------------------------------

/// Resource that keeps the [`PlayerBulletConfig`] asset handle alive.
#[derive(Resource)]
pub struct PlayerBulletConfigHandle(pub Handle<PlayerBulletConfig>);

// ---------------------------------------------------------------------------
// SystemParam bundle
// ---------------------------------------------------------------------------

/// Convenience accessor for [`PlayerBulletConfig`].
///
/// Returns `None` from `.get()` while the asset is still loading.
/// Individual getter methods always return a usable value by falling back to
/// the `DEFAULT_*` constants.
#[derive(SystemParam)]
pub struct PlayerBulletConfigParams<'w> {
    handle: Option<Res<'w, PlayerBulletConfigHandle>>,
    assets: Option<Res<'w, Assets<PlayerBulletConfig>>>,
}

impl<'w> PlayerBulletConfigParams<'w> {
    /// Returns the loaded [`PlayerBulletConfig`], or `None` while loading.
    pub fn get(&self) -> Option<&PlayerBulletConfig> {
        self.handle
            .as_ref()
            .and_then(|h| self.assets.as_ref().and_then(|a| a.get(&h.0)))
    }

    pub fn speed(&self) -> f32 {
        self.get()
            .map(|c| c.speed)
            .unwrap_or(DEFAULT_PLAYER_BULLET_SPEED)
    }

    pub fn spread(&self) -> f32 {
        self.get()
            .map(|c| c.spread)
            .unwrap_or(DEFAULT_PLAYER_BULLET_SPREAD)
    }

    pub fn spread_speed_scale(&self) -> f32 {
        self.get()
            .map(|c| c.spread_speed_scale)
            .unwrap_or(DEFAULT_PLAYER_BULLET_SPREAD_SPEED_SCALE)
    }

    pub fn origin_y_offset(&self) -> f32 {
        self.get()
            .map(|c| c.origin_y_offset)
            .unwrap_or(DEFAULT_PLAYER_BULLET_ORIGIN_Y_OFFSET)
    }

    pub fn collision_radius(&self) -> f32 {
        self.get()
            .map(|c| c.collision_radius)
            .unwrap_or(DEFAULT_PLAYER_BULLET_COLLISION_RADIUS)
    }

    pub fn sprite_width(&self) -> f32 {
        self.get()
            .map(|c| c.sprite_width)
            .unwrap_or(DEFAULT_PLAYER_BULLET_SPRITE_WIDTH)
    }

    pub fn sprite_height(&self) -> f32 {
        self.get()
            .map(|c| c.sprite_height)
            .unwrap_or(DEFAULT_PLAYER_BULLET_SPRITE_HEIGHT)
    }

    pub fn damage(&self) -> f32 {
        self.get()
            .map(|c| c.damage)
            .unwrap_or(DEFAULT_PLAYER_BULLET_DAMAGE)
    }
}

// ---------------------------------------------------------------------------
// Hot-reload system
// ---------------------------------------------------------------------------

/// Logs asset lifecycle events for [`PlayerBulletConfig`].
pub fn hot_reload_player_bullet_config(
    mut events: MessageReader<AssetEvent<PlayerBulletConfig>>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Added { .. } => info!("PlayerBulletConfig loaded"),
            AssetEvent::Modified { .. } => info!("PlayerBulletConfig modified (hot-reload)"),
            AssetEvent::Removed { .. } => warn!("PlayerBulletConfig removed"),
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

    fn make_config_from_ron(src: &str) -> PlayerBulletConfig {
        let options = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME);
        let partial: PlayerBulletConfigPartial = options.from_str(src).unwrap();
        PlayerBulletConfig::from(partial)
    }

    #[test]
    fn ron_deserialization_full() {
        let src = r#"
PlayerBulletConfig(
    speed: 700.0,
    spread: 12.0,
    spread_speed_scale: 6.0,
    origin_y_offset: 18.0,
    collision_radius: 4.0,
    sprite_width: 5.0,
    sprite_height: 14.0,
    damage: 15.0,
)
"#;
        let cfg = make_config_from_ron(src);
        assert_eq!(cfg.speed, 700.0);
        assert_eq!(cfg.spread, 12.0);
        assert_eq!(cfg.spread_speed_scale, 6.0);
        assert_eq!(cfg.origin_y_offset, 18.0);
        assert_eq!(cfg.collision_radius, 4.0);
        assert_eq!(cfg.sprite_width, 5.0);
        assert_eq!(cfg.sprite_height, 14.0);
        assert_eq!(cfg.damage, 15.0);
    }

    #[test]
    fn ron_deserialization_partial_uses_defaults() {
        let src = r#"PlayerBulletConfig(speed: 800.0)"#;
        let cfg = make_config_from_ron(src);
        assert_eq!(cfg.speed, 800.0);
        assert_eq!(cfg.spread, DEFAULT_PLAYER_BULLET_SPREAD);
        assert_eq!(cfg.spread_speed_scale, DEFAULT_PLAYER_BULLET_SPREAD_SPEED_SCALE);
        assert_eq!(cfg.origin_y_offset, DEFAULT_PLAYER_BULLET_ORIGIN_Y_OFFSET);
        assert_eq!(cfg.collision_radius, DEFAULT_PLAYER_BULLET_COLLISION_RADIUS);
        assert_eq!(cfg.sprite_width, DEFAULT_PLAYER_BULLET_SPRITE_WIDTH);
        assert_eq!(cfg.sprite_height, DEFAULT_PLAYER_BULLET_SPRITE_HEIGHT);
        assert_eq!(cfg.damage, DEFAULT_PLAYER_BULLET_DAMAGE);
    }

    #[test]
    fn empty_ron_falls_back_to_all_defaults() {
        let src = r#"PlayerBulletConfig()"#;
        let cfg = make_config_from_ron(src);
        assert_eq!(cfg.speed, DEFAULT_PLAYER_BULLET_SPEED);
        assert_eq!(cfg.collision_radius, DEFAULT_PLAYER_BULLET_COLLISION_RADIUS);
        assert_eq!(cfg.damage, DEFAULT_PLAYER_BULLET_DAMAGE);
    }
}
