use bevy::prelude::*;

use crate::{
    AppState,
    constants::{PLAY_AREA_HALF_H, PLAY_AREA_HALF_W, PLAY_AREA_HEIGHT, PLAY_AREA_WIDTH},
};

/// Immediately transitions to [`AppState::Playing`] on startup.
///
/// Skips the title / character-select flow so developers can test gameplay
/// without implementing UI screens first. Only active with `debug-hitbox`.
pub fn debug_skip_to_playing(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::Playing);
}

/// Draws the play-area boundary and center cross using Bevy Gizmos.
///
/// Enabled only when the `debug-hitbox` feature is active:
/// ```bash
/// cargo run -p touhou-project-scarlet --features scarlet-core/debug-hitbox
/// ```
pub fn debug_play_area_system(mut gizmos: Gizmos) {
    let color = Color::srgba(1.0, 1.0, 0.0, 0.8);

    // Outer boundary rectangle.
    gizmos.rect_2d(
        Isometry2d::IDENTITY,
        Vec2::new(PLAY_AREA_WIDTH, PLAY_AREA_HEIGHT),
        color,
    );

    // Center cross for orientation reference.
    gizmos.line_2d(
        Vec2::new(-PLAY_AREA_HALF_W, 0.0),
        Vec2::new(PLAY_AREA_HALF_W, 0.0),
        Color::srgba(1.0, 1.0, 0.0, 0.3),
    );
    gizmos.line_2d(
        Vec2::new(0.0, -PLAY_AREA_HALF_H),
        Vec2::new(0.0, PLAY_AREA_HALF_H),
        Color::srgba(1.0, 1.0, 0.0, 0.3),
    );
}
