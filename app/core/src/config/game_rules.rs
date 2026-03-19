//! Game-rules configuration loaded from `assets/config/game_rules.ron`.
//!
//! Covers item physics, score constants, and fragment-extend thresholds.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Fallback constants (used while game_rules.ron is still loading)
// ---------------------------------------------------------------------------

/// Speed at which attracted items move toward the player (px/s).
pub(crate) const DEFAULT_ITEM_ATTRACT_SPEED: f32 = 400.0;
/// Maximum downward fall speed for items (px/s, negative = downward).
pub(crate) const DEFAULT_ITEM_MAX_FALL_SPEED: f32 = -200.0;
/// Radius within which the player collects an item (px).
pub(crate) const DEFAULT_ITEM_COLLECT_RADIUS: f32 = 8.0;
/// Y coordinate of the score line (px).  At or above this, all items are
/// auto-attracted and point items yield their maximum value.
pub(crate) const DEFAULT_SCORE_LINE_Y: f32 = 192.0;
/// Maximum score from a point item (player at or above the score line).
pub(crate) const DEFAULT_POI_BASE_VALUE: u32 = 10_000;
/// Minimum score from a point item (player at the bottom of the play area).
pub(crate) const DEFAULT_POI_MIN_VALUE: u32 = 100;
/// Life fragments needed for a 1-UP extend.
pub(crate) const DEFAULT_LIFE_EXTEND_FRAGMENTS: u8 = 5;
/// Bomb fragments needed for an extra bomb extend.
pub(crate) const DEFAULT_BOMB_EXTEND_FRAGMENTS: u8 = 5;
/// Maximum bomb stock a player can hold.
pub(crate) const DEFAULT_MAX_BOMBS: u8 = 3;

// ---------------------------------------------------------------------------
// Asset type
// ---------------------------------------------------------------------------

/// Game-play rules loaded from `assets/config/game_rules.ron`.
///
/// All fields have game-ready defaults so systems never panic while the asset
/// is still loading.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct GameRulesConfig {
    // Item physics
    /// Speed at which attracted items travel toward the player (px/s).
    pub item_attract_speed: f32,
    /// Terminal downward fall speed for un-attracted items (px/s, negative).
    pub item_max_fall_speed: f32,
    /// Pickup collection radius (px).  Item is collected when closer than this.
    pub item_collect_radius: f32,
    // Score constants
    /// Y coordinate of the score line (px).
    pub score_line_y: f32,
    /// Point-item value when the player is at or above the score line.
    pub poi_base_value: u32,
    /// Point-item value when the player is at the bottom of the play area.
    pub poi_min_value: u32,
    // Fragment-extend thresholds
    /// Number of life fragments required for a 1-UP.
    pub life_extend_fragments: u8,
    /// Number of bomb fragments required for an extra bomb.
    pub bomb_extend_fragments: u8,
    /// Maximum bomb stock.
    pub max_bombs: u8,
}

impl Default for GameRulesConfig {
    fn default() -> Self {
        Self {
            item_attract_speed: DEFAULT_ITEM_ATTRACT_SPEED,
            item_max_fall_speed: DEFAULT_ITEM_MAX_FALL_SPEED,
            item_collect_radius: DEFAULT_ITEM_COLLECT_RADIUS,
            score_line_y: DEFAULT_SCORE_LINE_Y,
            poi_base_value: DEFAULT_POI_BASE_VALUE,
            poi_min_value: DEFAULT_POI_MIN_VALUE,
            life_extend_fragments: DEFAULT_LIFE_EXTEND_FRAGMENTS,
            bomb_extend_fragments: DEFAULT_BOMB_EXTEND_FRAGMENTS,
            max_bombs: DEFAULT_MAX_BOMBS,
        }
    }
}

// ---------------------------------------------------------------------------
// Handle resource
// ---------------------------------------------------------------------------

/// Resource that keeps the [`GameRulesConfig`] asset handle alive.
#[derive(Resource)]
pub struct GameRulesConfigHandle(pub Handle<GameRulesConfig>);

// ---------------------------------------------------------------------------
// SystemParam bundle
// ---------------------------------------------------------------------------

/// Convenience accessor for [`GameRulesConfig`].
///
/// Returns `None` from `.get()` while the asset is still loading.
/// `.get_or_default()` always returns a usable value.
#[derive(SystemParam)]
pub struct GameRulesConfigParams<'w> {
    handle: Option<Res<'w, GameRulesConfigHandle>>,
    assets: Option<Res<'w, Assets<GameRulesConfig>>>,
}

impl<'w> GameRulesConfigParams<'w> {
    /// Returns the loaded [`GameRulesConfig`], or `None` while loading.
    pub fn get(&self) -> Option<&GameRulesConfig> {
        self.handle
            .as_ref()
            .and_then(|h| self.assets.as_ref().and_then(|a| a.get(&h.0)))
    }

    /// Returns the loaded config, or [`GameRulesConfig::default`] as a fallback.
    pub fn get_or_default(&self) -> GameRulesConfig {
        self.get().cloned().unwrap_or_default()
    }
}

// ---------------------------------------------------------------------------
// Hot-reload system
// ---------------------------------------------------------------------------

/// Logs asset lifecycle events for [`GameRulesConfig`].
pub fn hot_reload_game_rules_config(mut events: MessageReader<AssetEvent<GameRulesConfig>>) {
    for event in events.read() {
        match event {
            AssetEvent::Added { .. } => info!("GameRulesConfig loaded"),
            AssetEvent::Modified { .. } => info!("GameRulesConfig modified (hot-reload)"),
            AssetEvent::Removed { .. } => warn!("GameRulesConfig removed"),
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
        let cfg = GameRulesConfig::default();
        assert_eq!(cfg.item_attract_speed, DEFAULT_ITEM_ATTRACT_SPEED);
        assert_eq!(cfg.item_max_fall_speed, DEFAULT_ITEM_MAX_FALL_SPEED);
        assert_eq!(cfg.item_collect_radius, DEFAULT_ITEM_COLLECT_RADIUS);
        assert_eq!(cfg.score_line_y, DEFAULT_SCORE_LINE_Y);
        assert_eq!(cfg.poi_base_value, DEFAULT_POI_BASE_VALUE);
        assert_eq!(cfg.poi_min_value, DEFAULT_POI_MIN_VALUE);
        assert_eq!(cfg.life_extend_fragments, DEFAULT_LIFE_EXTEND_FRAGMENTS);
        assert_eq!(cfg.bomb_extend_fragments, DEFAULT_BOMB_EXTEND_FRAGMENTS);
        assert_eq!(cfg.max_bombs, DEFAULT_MAX_BOMBS);
    }

    #[test]
    fn ron_deserialization_full() {
        let src = r#"
GameRulesConfig(
    item_attract_speed: 500.0,
    item_max_fall_speed: -300.0,
    item_collect_radius: 10.0,
    score_line_y: 200.0,
    poi_base_value: 12000,
    poi_min_value: 200,
    life_extend_fragments: 6,
    bomb_extend_fragments: 4,
    max_bombs: 4,
)
"#;
        let cfg: GameRulesConfig = ron::from_str(src).unwrap();
        assert_eq!(cfg.item_attract_speed, 500.0);
        assert_eq!(cfg.poi_base_value, 12000);
        assert_eq!(cfg.life_extend_fragments, 6);
        assert_eq!(cfg.max_bombs, 4);
    }

    #[test]
    fn ron_deserialization_partial_uses_defaults() {
        let src = r#"GameRulesConfig(item_attract_speed: 600.0)"#;
        let cfg: GameRulesConfig = ron::from_str(src).unwrap();
        assert_eq!(cfg.item_attract_speed, 600.0);
        assert_eq!(cfg.item_max_fall_speed, DEFAULT_ITEM_MAX_FALL_SPEED);
        assert_eq!(cfg.poi_base_value, DEFAULT_POI_BASE_VALUE);
    }
}
