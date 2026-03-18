use bevy::{post_process::bloom::Bloom, prelude::*};

/// Spawns the primary 2D camera with bloom enabled.
///
/// Registered as a `Startup` system in [`crate::ScarletUiPlugin`].
/// The camera is centered at the origin; the play area is drawn centered
/// there as well, so no translation offset is needed for Phase 02.
///
/// [`Bloom`] drives the glow effect for [`scarlet_core::shaders::BulletGlowMaterial`].
///
/// Future phases will offset the camera to accommodate the HUD panel
/// on the right side of the window.
pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera::default(),
        Bloom {
            intensity: 0.3,
            ..default()
        },
    ));
}
