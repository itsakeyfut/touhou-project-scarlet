use bevy::prelude::*;

/// Game state machine representing every distinct screen or mode.
///
/// State transitions:
/// ```text
/// Title → CharacterSelect → DifficultySelect → Loading → Playing
///                                                          ↓      ↓      ↓
///                                                       Paused StageClear GameOver
///                                                          ↓      ↓
///                                                       Playing Ending → StaffRoll → Title
/// ```
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    /// Main title screen (default starting state).
    #[default]
    Title,
    /// Character and shot-type selection screen.
    CharacterSelect,
    /// Difficulty selection screen (Easy / Normal / Hard / Lunatic / Extra / Phantasm).
    DifficultySelect,
    /// Asset loading screen; transitions to Playing once all assets are ready.
    Loading,
    /// Active gameplay.
    Playing,
    /// Pause overlay shown over the game (ESC to toggle).
    Paused,
    /// Stage-clear results screen shown after defeating a stage boss.
    StageClear,
    /// Game-over screen shown when the player loses all lives.
    GameOver,
    /// Ending sequence (character-specific story scene).
    Ending,
    /// Staff roll shown after the ending.
    StaffRoll,
}
