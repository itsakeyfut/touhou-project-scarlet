/// Width of the 4:3 play area in world-space pixels.
///
/// The play area occupies the center of the 16:9 window.
/// Left and right HUD panels fill the remaining width on each side.
pub const PLAY_AREA_WIDTH: f32 = 384.0;

/// Height of the 4:3 play area in world-space pixels.
pub const PLAY_AREA_HEIGHT: f32 = 448.0;

/// Half-width of the play area (used for boundary clamping).
pub const PLAY_AREA_HALF_W: f32 = PLAY_AREA_WIDTH / 2.0;

/// Half-height of the play area (used for boundary clamping).
pub const PLAY_AREA_HALF_H: f32 = PLAY_AREA_HEIGHT / 2.0;
