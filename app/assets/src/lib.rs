use bevy::prelude::*;

/// Assets plugin.
///
/// Will manage loading of all game assets (sprites, audio, shaders, fonts)
/// via `ScarletAssets` resource and `AssetsLoading` tracker.
/// Systems are added in Phase 19.
pub struct ScarletAssetsPlugin;

impl Plugin for ScarletAssetsPlugin {
    fn build(&self, _app: &mut App) {
        // TODO(phase-19): add ScarletAssets resource and loading systems
    }
}
