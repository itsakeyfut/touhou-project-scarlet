//! Stage-clear handler: runs once when `AppState::StageClear` is entered.
//!
//! Responsibilities (provisional — Phase 10):
//! * Award the flat [`STAGE_CLEAR_SCORE_BONUS`] to the player's score.
//! * Update `hi_score` if the current score exceeds it.
//! * If the cleared stage was the final stage, transition to
//!   [`AppState::Ending`].
//! * Otherwise, increment [`StageData::stage_number`] so the next
//!   `OnEnter(AppState::Playing)` load system prepares the correct stage.
//!
//! The StageClear UI screen (results display, continue prompt) is tracked in a
//! separate issue. For now, the transition away from StageClear is driven
//! externally (UI not yet implemented).

use bevy::prelude::*;

use crate::{
    constants::{STAGE_CLEAR_SCORE_BONUS, TOTAL_STAGES},
    resources::{GameData, StageData},
    states::AppState,
};

/// Runs on `OnEnter(AppState::StageClear)`.
///
/// Awards the stage-clear bonus, updates `hi_score`, and either advances the
/// stage number or (for the final stage) triggers the ending transition.
pub fn on_stage_clear(
    mut game_data: ResMut<GameData>,
    mut stage_data: ResMut<StageData>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    game_data.score += STAGE_CLEAR_SCORE_BONUS;
    if game_data.score > game_data.hi_score {
        game_data.hi_score = game_data.score;
    }

    if stage_data.stage_number >= TOTAL_STAGES {
        info!(
            stage = stage_data.stage_number,
            "Final stage cleared — transitioning to Ending"
        );
        next_state.set(AppState::Ending);
    } else {
        let next = stage_data.stage_number + 1;
        info!(
            cleared = stage_data.stage_number,
            next = next,
            "Stage cleared — advancing to stage {next}"
        );
        stage_data.stage_number = next;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Score bonus must match the public constant.
    #[test]
    fn stage_clear_score_bonus_is_positive() {
        assert!(STAGE_CLEAR_SCORE_BONUS > 0);
    }

    /// TOTAL_STAGES must cover the six EoSD stages.
    #[test]
    fn total_stages_is_six() {
        assert_eq!(TOTAL_STAGES, 6);
    }
}
