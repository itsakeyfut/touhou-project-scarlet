//! Fodder-enemy configuration loaded from `assets/config/enemies/fodder.ron`.
//!
//! Covers per-kind HP, collision radius, and score for the three fodder
//! enemy types: Fairy, Bat, and TallFairy.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

use crate::components::enemy::EnemyKind;

// ---------------------------------------------------------------------------
// Fallback constants (used while fodder.ron is still loading)
// ---------------------------------------------------------------------------

/// Base HP for a Fairy enemy.
pub(crate) const DEFAULT_FAIRY_HP: f32 = 10.0;
/// Bullet-collision hitbox radius for a Fairy (px).
pub(crate) const DEFAULT_FAIRY_RADIUS: f32 = 12.0;
/// Score awarded on defeating a Fairy.
pub(crate) const DEFAULT_FAIRY_SCORE: u32 = 100;

/// Base HP for a Bat enemy.
pub(crate) const DEFAULT_BAT_HP: f32 = 5.0;
/// Bullet-collision hitbox radius for a Bat (px).
pub(crate) const DEFAULT_BAT_RADIUS: f32 = 8.0;
/// Score awarded on defeating a Bat.
pub(crate) const DEFAULT_BAT_SCORE: u32 = 50;

/// Base HP for a TallFairy enemy.
pub(crate) const DEFAULT_TALL_FAIRY_HP: f32 = 30.0;
/// Bullet-collision hitbox radius for a TallFairy (px).
pub(crate) const DEFAULT_TALL_FAIRY_RADIUS: f32 = 16.0;
/// Score awarded on defeating a TallFairy.
pub(crate) const DEFAULT_TALL_FAIRY_SCORE: u32 = 300;

// ---------------------------------------------------------------------------
// Partial (deserialization mirror)
// ---------------------------------------------------------------------------

/// Deserialization mirror of [`FodderEnemyConfig`] — every field is `Option<T>`
/// so RON files with missing fields still load and emit a `warn!` instead of failing.
#[derive(Deserialize, Default)]
#[serde(default, rename = "FodderEnemyConfig")]
pub(super) struct FodderEnemyConfigPartial {
    pub fairy_hp: Option<f32>,
    pub fairy_radius: Option<f32>,
    pub fairy_score: Option<u32>,
    pub bat_hp: Option<f32>,
    pub bat_radius: Option<f32>,
    pub bat_score: Option<u32>,
    pub tall_fairy_hp: Option<f32>,
    pub tall_fairy_radius: Option<f32>,
    pub tall_fairy_score: Option<u32>,
}

// ---------------------------------------------------------------------------
// Asset type
// ---------------------------------------------------------------------------

/// Per-kind stats for fodder enemies loaded from `assets/config/enemies/fodder.ron`.
///
/// Covers the three basic enemy types that appear in Stage 1:
/// Fairy, Bat, and TallFairy.
#[derive(Asset, TypePath, Debug, Clone)]
pub struct FodderEnemyConfig {
    /// Base (maximum) HP for a Fairy.
    pub fairy_hp: f32,
    /// Bullet-collision hitbox radius for a Fairy (px).
    pub fairy_radius: f32,
    /// Score awarded on defeating a Fairy.
    pub fairy_score: u32,
    /// Base (maximum) HP for a Bat.
    pub bat_hp: f32,
    /// Bullet-collision hitbox radius for a Bat (px).
    pub bat_radius: f32,
    /// Score awarded on defeating a Bat.
    pub bat_score: u32,
    /// Base (maximum) HP for a TallFairy.
    pub tall_fairy_hp: f32,
    /// Bullet-collision hitbox radius for a TallFairy (px).
    pub tall_fairy_radius: f32,
    /// Score awarded on defeating a TallFairy.
    pub tall_fairy_score: u32,
}

impl From<FodderEnemyConfigPartial> for FodderEnemyConfig {
    fn from(p: FodderEnemyConfigPartial) -> Self {
        FodderEnemyConfig {
            fairy_hp: p.fairy_hp.unwrap_or_else(|| {
                warn!(
                    "fodder.ron: `fairy_hp` missing → using default {DEFAULT_FAIRY_HP}"
                );
                DEFAULT_FAIRY_HP
            }),
            fairy_radius: p.fairy_radius.unwrap_or_else(|| {
                warn!(
                    "fodder.ron: `fairy_radius` missing → using default {DEFAULT_FAIRY_RADIUS}"
                );
                DEFAULT_FAIRY_RADIUS
            }),
            fairy_score: p.fairy_score.unwrap_or_else(|| {
                warn!(
                    "fodder.ron: `fairy_score` missing → using default {DEFAULT_FAIRY_SCORE}"
                );
                DEFAULT_FAIRY_SCORE
            }),
            bat_hp: p.bat_hp.unwrap_or_else(|| {
                warn!(
                    "fodder.ron: `bat_hp` missing → using default {DEFAULT_BAT_HP}"
                );
                DEFAULT_BAT_HP
            }),
            bat_radius: p.bat_radius.unwrap_or_else(|| {
                warn!(
                    "fodder.ron: `bat_radius` missing → using default {DEFAULT_BAT_RADIUS}"
                );
                DEFAULT_BAT_RADIUS
            }),
            bat_score: p.bat_score.unwrap_or_else(|| {
                warn!(
                    "fodder.ron: `bat_score` missing → using default {DEFAULT_BAT_SCORE}"
                );
                DEFAULT_BAT_SCORE
            }),
            tall_fairy_hp: p.tall_fairy_hp.unwrap_or_else(|| {
                warn!(
                    "fodder.ron: `tall_fairy_hp` missing → using default {DEFAULT_TALL_FAIRY_HP}"
                );
                DEFAULT_TALL_FAIRY_HP
            }),
            tall_fairy_radius: p.tall_fairy_radius.unwrap_or_else(|| {
                warn!(
                    "fodder.ron: `tall_fairy_radius` missing → using default {DEFAULT_TALL_FAIRY_RADIUS}"
                );
                DEFAULT_TALL_FAIRY_RADIUS
            }),
            tall_fairy_score: p.tall_fairy_score.unwrap_or_else(|| {
                warn!(
                    "fodder.ron: `tall_fairy_score` missing → using default {DEFAULT_TALL_FAIRY_SCORE}"
                );
                DEFAULT_TALL_FAIRY_SCORE
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// Handle resource
// ---------------------------------------------------------------------------

/// Resource that keeps the [`FodderEnemyConfig`] asset handle alive.
#[derive(Resource)]
pub struct FodderEnemyConfigHandle(pub Handle<FodderEnemyConfig>);

// ---------------------------------------------------------------------------
// SystemParam bundle
// ---------------------------------------------------------------------------

/// Convenience accessor for [`FodderEnemyConfig`].
///
/// Returns `None` from `.get()` while the asset is still loading.
/// Per-kind getter methods always return a usable value by falling back to
/// the `DEFAULT_*` constants.
#[derive(SystemParam)]
pub struct FodderEnemyConfigParams<'w> {
    handle: Option<Res<'w, FodderEnemyConfigHandle>>,
    assets: Option<Res<'w, Assets<FodderEnemyConfig>>>,
}

impl<'w> FodderEnemyConfigParams<'w> {
    /// Returns the loaded [`FodderEnemyConfig`], or `None` while loading.
    pub fn get(&self) -> Option<&FodderEnemyConfig> {
        self.handle
            .as_ref()
            .and_then(|h| self.assets.as_ref().and_then(|a| a.get(&h.0)))
    }

    /// Base HP for the given enemy kind.
    pub fn hp_for(&self, kind: EnemyKind) -> f32 {
        match self.get() {
            Some(cfg) => match kind {
                EnemyKind::Fairy => cfg.fairy_hp,
                EnemyKind::Bat => cfg.bat_hp,
                EnemyKind::TallFairy => cfg.tall_fairy_hp,
            },
            None => match kind {
                EnemyKind::Fairy => DEFAULT_FAIRY_HP,
                EnemyKind::Bat => DEFAULT_BAT_HP,
                EnemyKind::TallFairy => DEFAULT_TALL_FAIRY_HP,
            },
        }
    }

    /// Bullet-collision hitbox radius for the given enemy kind (px).
    pub fn radius_for(&self, kind: EnemyKind) -> f32 {
        match self.get() {
            Some(cfg) => match kind {
                EnemyKind::Fairy => cfg.fairy_radius,
                EnemyKind::Bat => cfg.bat_radius,
                EnemyKind::TallFairy => cfg.tall_fairy_radius,
            },
            None => match kind {
                EnemyKind::Fairy => DEFAULT_FAIRY_RADIUS,
                EnemyKind::Bat => DEFAULT_BAT_RADIUS,
                EnemyKind::TallFairy => DEFAULT_TALL_FAIRY_RADIUS,
            },
        }
    }

    /// Score awarded on defeating the given enemy kind.
    pub fn score_for(&self, kind: EnemyKind) -> u32 {
        match self.get() {
            Some(cfg) => match kind {
                EnemyKind::Fairy => cfg.fairy_score,
                EnemyKind::Bat => cfg.bat_score,
                EnemyKind::TallFairy => cfg.tall_fairy_score,
            },
            None => match kind {
                EnemyKind::Fairy => DEFAULT_FAIRY_SCORE,
                EnemyKind::Bat => DEFAULT_BAT_SCORE,
                EnemyKind::TallFairy => DEFAULT_TALL_FAIRY_SCORE,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Hot-reload system
// ---------------------------------------------------------------------------

/// Logs asset lifecycle events for [`FodderEnemyConfig`].
pub fn hot_reload_fodder_enemy_config(
    mut events: MessageReader<AssetEvent<FodderEnemyConfig>>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Added { .. } => info!("FodderEnemyConfig loaded"),
            AssetEvent::Modified { .. } => info!("FodderEnemyConfig modified (hot-reload)"),
            AssetEvent::Removed { .. } => warn!("FodderEnemyConfig removed"),
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

    fn make_config_from_ron(src: &str) -> FodderEnemyConfig {
        let options = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME);
        let partial: FodderEnemyConfigPartial = options.from_str(src).unwrap();
        FodderEnemyConfig::from(partial)
    }

    #[test]
    fn ron_deserialization_full() {
        let src = r#"
FodderEnemyConfig(
    fairy_hp: 12.0,
    fairy_radius: 14.0,
    fairy_score: 120,
    bat_hp: 6.0,
    bat_radius: 9.0,
    bat_score: 60,
    tall_fairy_hp: 35.0,
    tall_fairy_radius: 18.0,
    tall_fairy_score: 350,
)
"#;
        let cfg = make_config_from_ron(src);
        assert_eq!(cfg.fairy_hp, 12.0);
        assert_eq!(cfg.fairy_radius, 14.0);
        assert_eq!(cfg.fairy_score, 120);
        assert_eq!(cfg.bat_hp, 6.0);
        assert_eq!(cfg.bat_radius, 9.0);
        assert_eq!(cfg.bat_score, 60);
        assert_eq!(cfg.tall_fairy_hp, 35.0);
        assert_eq!(cfg.tall_fairy_radius, 18.0);
        assert_eq!(cfg.tall_fairy_score, 350);
    }

    #[test]
    fn ron_deserialization_partial_uses_defaults() {
        let src = r#"FodderEnemyConfig(fairy_hp: 15.0)"#;
        let cfg = make_config_from_ron(src);
        assert_eq!(cfg.fairy_hp, 15.0);
        assert_eq!(cfg.fairy_radius, DEFAULT_FAIRY_RADIUS);
        assert_eq!(cfg.fairy_score, DEFAULT_FAIRY_SCORE);
        assert_eq!(cfg.bat_hp, DEFAULT_BAT_HP);
        assert_eq!(cfg.bat_radius, DEFAULT_BAT_RADIUS);
        assert_eq!(cfg.bat_score, DEFAULT_BAT_SCORE);
        assert_eq!(cfg.tall_fairy_hp, DEFAULT_TALL_FAIRY_HP);
        assert_eq!(cfg.tall_fairy_radius, DEFAULT_TALL_FAIRY_RADIUS);
        assert_eq!(cfg.tall_fairy_score, DEFAULT_TALL_FAIRY_SCORE);
    }

    #[test]
    fn empty_ron_falls_back_to_all_defaults() {
        let src = r#"FodderEnemyConfig()"#;
        let cfg = make_config_from_ron(src);
        assert_eq!(cfg.fairy_hp, DEFAULT_FAIRY_HP);
        assert_eq!(cfg.bat_hp, DEFAULT_BAT_HP);
        assert_eq!(cfg.tall_fairy_hp, DEFAULT_TALL_FAIRY_HP);
    }
}
