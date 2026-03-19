use bevy::prelude::*;

/// Game state machine representing every distinct screen or mode.
///
/// State transitions:
/// ```text
/// Loading → Playing (once all RON configs are ready — see wait_for_configs)
///
/// Full future flow (UI screens not yet implemented):
/// Title → CharacterSelect → DifficultySelect → Loading → Playing
///                                                          ↓      ↓      ↓
///                                                       Paused StageClear GameOver
///                                                          ↓      ↓
///                                                       Playing Ending → StaffRoll → Title
/// ```
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    /// Asset / config loading screen (default starting state).
    ///
    /// [`crate::systems::loading::wait_for_configs`] transitions to
    /// [`AppState::Playing`] once every RON config file has finished loading.
    /// Future phases will insert Title/CharacterSelect/DifficultySelect before
    /// this state.
    #[default]
    Loading,
    /// Main title screen (not yet implemented — placeholder for future phase).
    Title,
    /// Character and shot-type selection screen.
    CharacterSelect,
    /// Difficulty selection screen (Easy / Normal / Hard / Lunatic / Extra / Phantasm).
    DifficultySelect,
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
