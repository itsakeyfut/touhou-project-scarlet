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

/// Total number of main stages in the game.
///
/// Stage 6 (Remilia Scarlett) is the final stage; clearing it triggers
/// the ending sequence instead of continuing to a next stage.
pub const TOTAL_STAGES: u8 = 6;

/// Score bonus awarded to the player when a stage boss is defeated.
///
/// This is a provisional flat bonus applied in [`crate::systems::stage_clear`].
/// The value may be tuned or replaced with a dynamic formula in a future issue.
pub const STAGE_CLEAR_SCORE_BONUS: u64 = 100_000;
