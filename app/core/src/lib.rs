use bevy::prelude::*;

pub mod states;

pub use states::AppState;

/// Core game plugin.
///
/// Registers the [`AppState`] state machine. Subsequent phases will add
/// resources, events, system sets, and game-logic systems here.
pub struct ScarletCorePlugin;

impl Plugin for ScarletCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>();
    }
}
