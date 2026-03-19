//! Game configuration loaded from RON files.
//!
//! Handles loading and hot-reloading of game configuration from RON
//! (Rusty Object Notation) files in `assets/config/`.
//!
//! Supports hot-reloading: edit config files while the game is running
//! and changes will be applied automatically (see individual hot-reload
//! systems for what updates live).
//!
//! # Sub-modules
//!
//! | Module          | Contents |
//! |-----------------|----------|
//! | [`player`]      | `PlayerConfig` + `PlayerConfigHandle` + `PlayerConfigParams` |
//! | [`game_rules`]  | `GameRulesConfig` + `GameRulesConfigHandle` + `GameRulesConfigParams` |

pub mod game_rules;
pub mod player;

pub use game_rules::*;
pub use player::*;

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;

// ---------------------------------------------------------------------------
// RON asset loader macro (two-step: Partial → Asset via From)
// ---------------------------------------------------------------------------

/// Generates a RON-based [`AssetLoader`] implementation for a config type.
///
/// Uses the two-step deserialization form: RON bytes are first parsed into a
/// `$partial` type (all fields `Option<T>` with `#[serde(default)]`), then
/// converted to `$asset` via `From`.  Missing fields in the RON file are
/// caught in `From::from` and logged as `warn!` messages, so the game
/// continues with fallback values instead of panicking.
///
/// # Usage
/// ```ignore
/// ron_asset_loader!(MyConfigLoader, MyConfigPartial => MyConfig);
/// ```
macro_rules! ron_asset_loader {
    ($loader:ident, $partial:ty => $asset:ty) => {
        #[derive(Default)]
        struct $loader;

        impl AssetLoader for $loader {
            type Asset = $asset;
            type Settings = ();
            type Error = std::io::Error;

            async fn load(
                &self,
                reader: &mut dyn Reader,
                _settings: &Self::Settings,
                _load_context: &mut LoadContext<'_>,
            ) -> Result<Self::Asset, Self::Error> {
                let mut bytes = Vec::new();
                reader.read_to_end(&mut bytes).await?;
                let options = ron::Options::default()
                    .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME);
                let partial: $partial = options
                    .from_bytes(&bytes)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                Ok(<$asset>::from(partial))
            }

            fn extensions(&self) -> &[&str] {
                &["ron"]
            }
        }
    };
}

ron_asset_loader!(PlayerConfigLoader, player::PlayerConfigPartial => PlayerConfig);
ron_asset_loader!(GameRulesConfigLoader, game_rules::GameRulesConfigPartial => GameRulesConfig);

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Plugin for Scarlet game configuration management.
///
/// - Registers RON asset loaders for all config types.
/// - Loads `assets/config/player.ron` and `assets/config/game_rules.ron`.
/// - Inserts `PlayerConfigHandle` and `GameRulesConfigHandle` as resources.
/// - Registers hot-reload logging systems (run in all states).
///
/// **Must be added before [`crate::ScarletCorePlugin`]** so handles are
/// available when systems first run.
pub struct ScarletConfigPlugin;

impl Plugin for ScarletConfigPlugin {
    fn build(&self, app: &mut App) {
        // Register asset types and their RON loaders.
        app.init_asset::<PlayerConfig>()
            .register_asset_loader(PlayerConfigLoader)
            .init_asset::<GameRulesConfig>()
            .register_asset_loader(GameRulesConfigLoader);

        // Load config files and insert handles as resources.
        let asset_server = app.world_mut().resource::<AssetServer>();
        let player_handle: Handle<PlayerConfig> = asset_server.load("config/player.ron");
        let game_rules_handle: Handle<GameRulesConfig> =
            asset_server.load("config/game_rules.ron");

        app.insert_resource(PlayerConfigHandle(player_handle))
            .insert_resource(GameRulesConfigHandle(game_rules_handle));

        // Hot-reload logging (runs unconditionally in all states).
        app.add_systems(
            Update,
            (hot_reload_player_config, hot_reload_game_rules_config),
        );
    }
}
