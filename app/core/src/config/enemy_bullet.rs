//! Enemy-bullet configuration loaded from `assets/config/bullets/enemy.ron`.
//!
//! Stores the collision radius for each [`crate::components::bullet::EnemyBulletKind`]
//! variant. Radii are used both for hit detection and for sizing the glow mesh.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

use crate::components::bullet::EnemyBulletKind;

// ---------------------------------------------------------------------------
// Fallback constants (used while enemy.ron is still loading)
// ---------------------------------------------------------------------------

/// Collision radius for [`EnemyBulletKind::SmallRound`] (px).
pub(crate) const DEFAULT_SMALL_ROUND_RADIUS: f32 = 4.0;
/// Collision radius for [`EnemyBulletKind::Knife`] (px).
pub(crate) const DEFAULT_KNIFE_RADIUS: f32 = 4.0;
/// Collision radius for [`EnemyBulletKind::Rice`] (px).
pub(crate) const DEFAULT_RICE_RADIUS: f32 = 5.0;
/// Collision radius for [`EnemyBulletKind::MediumRound`] (px).
pub(crate) const DEFAULT_MEDIUM_ROUND_RADIUS: f32 = 7.0;
/// Collision radius for [`EnemyBulletKind::Star`] (px).
pub(crate) const DEFAULT_STAR_RADIUS: f32 = 8.0;
/// Collision radius for [`EnemyBulletKind::Bubble`] (px).
pub(crate) const DEFAULT_BUBBLE_RADIUS: f32 = 9.0;
/// Collision radius for [`EnemyBulletKind::LargeRound`] (px).
pub(crate) const DEFAULT_LARGE_ROUND_RADIUS: f32 = 11.0;

// ---------------------------------------------------------------------------
// Partial (deserialization mirror)
// ---------------------------------------------------------------------------

/// Deserialization mirror of [`EnemyBulletConfig`] — every field is `Option<T>`
/// so RON files with missing fields still load and emit a `warn!` instead of failing.
#[derive(Deserialize, Default)]
#[serde(default, rename = "EnemyBulletConfig")]
pub(super) struct EnemyBulletConfigPartial {
    pub small_round_radius: Option<f32>,
    pub knife_radius: Option<f32>,
    pub rice_radius: Option<f32>,
    pub medium_round_radius: Option<f32>,
    pub star_radius: Option<f32>,
    pub bubble_radius: Option<f32>,
    pub large_round_radius: Option<f32>,
}

// ---------------------------------------------------------------------------
// Asset type
// ---------------------------------------------------------------------------

/// Per-kind collision radii for enemy bullets loaded from
/// `assets/config/bullets/enemy.ron`.
///
/// Hot-reloading takes effect within one frame for collision detection.
/// The glow mesh radius in [`crate::systems::danmaku::patterns`] will also
/// use these values once Phase 19 sprite work is complete.
#[derive(Asset, TypePath, Debug, Clone)]
pub struct EnemyBulletConfig {
    /// Collision radius for [`EnemyBulletKind::SmallRound`] (px).
    pub small_round_radius: f32,
    /// Collision radius for [`EnemyBulletKind::Knife`] (px).
    pub knife_radius: f32,
    /// Collision radius for [`EnemyBulletKind::Rice`] (px).
    pub rice_radius: f32,
    /// Collision radius for [`EnemyBulletKind::MediumRound`] (px).
    pub medium_round_radius: f32,
    /// Collision radius for [`EnemyBulletKind::Star`] (px).
    pub star_radius: f32,
    /// Collision radius for [`EnemyBulletKind::Bubble`] (px).
    pub bubble_radius: f32,
    /// Collision radius for [`EnemyBulletKind::LargeRound`] (px).
    pub large_round_radius: f32,
}

impl From<EnemyBulletConfigPartial> for EnemyBulletConfig {
    fn from(p: EnemyBulletConfigPartial) -> Self {
        EnemyBulletConfig {
            small_round_radius: p.small_round_radius.unwrap_or_else(|| {
                warn!(
                    "bullets/enemy.ron: `small_round_radius` missing → using default {DEFAULT_SMALL_ROUND_RADIUS}"
                );
                DEFAULT_SMALL_ROUND_RADIUS
            }),
            knife_radius: p.knife_radius.unwrap_or_else(|| {
                warn!(
                    "bullets/enemy.ron: `knife_radius` missing → using default {DEFAULT_KNIFE_RADIUS}"
                );
                DEFAULT_KNIFE_RADIUS
            }),
            rice_radius: p.rice_radius.unwrap_or_else(|| {
                warn!(
                    "bullets/enemy.ron: `rice_radius` missing → using default {DEFAULT_RICE_RADIUS}"
                );
                DEFAULT_RICE_RADIUS
            }),
            medium_round_radius: p.medium_round_radius.unwrap_or_else(|| {
                warn!(
                    "bullets/enemy.ron: `medium_round_radius` missing → using default {DEFAULT_MEDIUM_ROUND_RADIUS}"
                );
                DEFAULT_MEDIUM_ROUND_RADIUS
            }),
            star_radius: p.star_radius.unwrap_or_else(|| {
                warn!(
                    "bullets/enemy.ron: `star_radius` missing → using default {DEFAULT_STAR_RADIUS}"
                );
                DEFAULT_STAR_RADIUS
            }),
            bubble_radius: p.bubble_radius.unwrap_or_else(|| {
                warn!(
                    "bullets/enemy.ron: `bubble_radius` missing → using default {DEFAULT_BUBBLE_RADIUS}"
                );
                DEFAULT_BUBBLE_RADIUS
            }),
            large_round_radius: p.large_round_radius.unwrap_or_else(|| {
                warn!(
                    "bullets/enemy.ron: `large_round_radius` missing → using default {DEFAULT_LARGE_ROUND_RADIUS}"
                );
                DEFAULT_LARGE_ROUND_RADIUS
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// Handle resource
// ---------------------------------------------------------------------------

/// Resource that keeps the [`EnemyBulletConfig`] asset handle alive.
#[derive(Resource)]
pub struct EnemyBulletConfigHandle(pub Handle<EnemyBulletConfig>);

// ---------------------------------------------------------------------------
// SystemParam bundle
// ---------------------------------------------------------------------------

/// Convenience accessor for [`EnemyBulletConfig`].
///
/// Returns `None` from `.get()` while the asset is still loading.
/// `radius_for` always returns a usable value by falling back to
/// the `DEFAULT_*` constants.
#[derive(SystemParam)]
pub struct EnemyBulletConfigParams<'w> {
    handle: Option<Res<'w, EnemyBulletConfigHandle>>,
    assets: Option<Res<'w, Assets<EnemyBulletConfig>>>,
}

impl<'w> EnemyBulletConfigParams<'w> {
    /// Returns the loaded [`EnemyBulletConfig`], or `None` while loading.
    pub fn get(&self) -> Option<&EnemyBulletConfig> {
        self.handle
            .as_ref()
            .and_then(|h| self.assets.as_ref().and_then(|a| a.get(&h.0)))
    }

    /// Collision radius for the given enemy bullet kind (px).
    pub fn radius_for(&self, kind: EnemyBulletKind) -> f32 {
        match self.get() {
            Some(cfg) => match kind {
                EnemyBulletKind::SmallRound => cfg.small_round_radius,
                EnemyBulletKind::Knife => cfg.knife_radius,
                EnemyBulletKind::Rice => cfg.rice_radius,
                EnemyBulletKind::MediumRound => cfg.medium_round_radius,
                EnemyBulletKind::Star => cfg.star_radius,
                EnemyBulletKind::Bubble => cfg.bubble_radius,
                EnemyBulletKind::LargeRound => cfg.large_round_radius,
            },
            None => match kind {
                EnemyBulletKind::SmallRound => DEFAULT_SMALL_ROUND_RADIUS,
                EnemyBulletKind::Knife => DEFAULT_KNIFE_RADIUS,
                EnemyBulletKind::Rice => DEFAULT_RICE_RADIUS,
                EnemyBulletKind::MediumRound => DEFAULT_MEDIUM_ROUND_RADIUS,
                EnemyBulletKind::Star => DEFAULT_STAR_RADIUS,
                EnemyBulletKind::Bubble => DEFAULT_BUBBLE_RADIUS,
                EnemyBulletKind::LargeRound => DEFAULT_LARGE_ROUND_RADIUS,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Hot-reload system
// ---------------------------------------------------------------------------

/// Logs asset lifecycle events for [`EnemyBulletConfig`].
pub fn hot_reload_enemy_bullet_config(
    mut events: MessageReader<AssetEvent<EnemyBulletConfig>>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Added { .. } => info!("EnemyBulletConfig loaded"),
            AssetEvent::Modified { .. } => info!("EnemyBulletConfig modified (hot-reload)"),
            AssetEvent::Removed { .. } => warn!("EnemyBulletConfig removed"),
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

    fn make_config_from_ron(src: &str) -> EnemyBulletConfig {
        let options = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME);
        let partial: EnemyBulletConfigPartial = options.from_str(src).unwrap();
        EnemyBulletConfig::from(partial)
    }

    #[test]
    fn ron_deserialization_full() {
        let src = r#"
EnemyBulletConfig(
    small_round_radius: 5.0,
    knife_radius: 3.0,
    rice_radius: 6.0,
    medium_round_radius: 8.0,
    star_radius: 9.0,
    bubble_radius: 10.0,
    large_round_radius: 12.0,
)
"#;
        let cfg = make_config_from_ron(src);
        assert_eq!(cfg.small_round_radius, 5.0);
        assert_eq!(cfg.knife_radius, 3.0);
        assert_eq!(cfg.rice_radius, 6.0);
        assert_eq!(cfg.medium_round_radius, 8.0);
        assert_eq!(cfg.star_radius, 9.0);
        assert_eq!(cfg.bubble_radius, 10.0);
        assert_eq!(cfg.large_round_radius, 12.0);
    }

    #[test]
    fn ron_deserialization_partial_uses_defaults() {
        let src = r#"EnemyBulletConfig(small_round_radius: 3.0)"#;
        let cfg = make_config_from_ron(src);
        assert_eq!(cfg.small_round_radius, 3.0);
        assert_eq!(cfg.knife_radius, DEFAULT_KNIFE_RADIUS);
        assert_eq!(cfg.rice_radius, DEFAULT_RICE_RADIUS);
        assert_eq!(cfg.medium_round_radius, DEFAULT_MEDIUM_ROUND_RADIUS);
        assert_eq!(cfg.star_radius, DEFAULT_STAR_RADIUS);
        assert_eq!(cfg.bubble_radius, DEFAULT_BUBBLE_RADIUS);
        assert_eq!(cfg.large_round_radius, DEFAULT_LARGE_ROUND_RADIUS);
    }

    #[test]
    fn empty_ron_falls_back_to_all_defaults() {
        let src = r#"EnemyBulletConfig()"#;
        let cfg = make_config_from_ron(src);
        assert_eq!(cfg.small_round_radius, DEFAULT_SMALL_ROUND_RADIUS);
        assert_eq!(cfg.knife_radius, DEFAULT_KNIFE_RADIUS);
        assert_eq!(cfg.large_round_radius, DEFAULT_LARGE_ROUND_RADIUS);
    }

    #[test]
    fn radius_for_matches_defaults_when_config_absent() {
        // Verify DEFAULT_* constants match each EnemyBulletKind variant.
        assert_eq!(DEFAULT_SMALL_ROUND_RADIUS, 4.0);
        assert_eq!(DEFAULT_KNIFE_RADIUS, 4.0);
        assert_eq!(DEFAULT_RICE_RADIUS, 5.0);
        assert_eq!(DEFAULT_MEDIUM_ROUND_RADIUS, 7.0);
        assert_eq!(DEFAULT_STAR_RADIUS, 8.0);
        assert_eq!(DEFAULT_BUBBLE_RADIUS, 9.0);
        assert_eq!(DEFAULT_LARGE_ROUND_RADIUS, 11.0);
    }
}
