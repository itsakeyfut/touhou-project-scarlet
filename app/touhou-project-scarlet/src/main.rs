use bevy::prelude::*;
use scarlet_assets::ScarletAssetsPlugin;
use scarlet_audio::ScarletAudioPlugin;
use scarlet_core::ScarletCorePlugin;
use scarlet_ui::ScarletUiPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "東方紅魔郷 〜 Embodiment of Scarlet Devil".into(),
                        // 16:9 window; the 4:3 play area sits in the center
                        // with dual side HUD panels (see docs/04_ui_ux.md).
                        resolution: (1280_u32, 720_u32).into(),
                        present_mode: bevy::window::PresentMode::AutoVsync,
                        ..default()
                    }),
                    ..default()
                })
                // Nearest-neighbor filtering is required for pixel-art sprites.
                .set(ImagePlugin::default_nearest()),
            ScarletAssetsPlugin,
            ScarletCorePlugin,
            ScarletUiPlugin,
            ScarletAudioPlugin,
        ))
        .run();
}
