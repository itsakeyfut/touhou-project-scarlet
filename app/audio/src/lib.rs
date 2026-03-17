use bevy::prelude::*;

/// Audio plugin.
///
/// Will manage BGM and SFX playback via separate bevy_kira_audio channels
/// (`BgmChannel`, `SfxChannel`). Systems are added in Phase 13.
pub struct ScarletAudioPlugin;

impl Plugin for ScarletAudioPlugin {
    fn build(&self, _app: &mut App) {
        // TODO(phase-13): add AudioPlugin, BGM/SFX channels and systems
    }
}
