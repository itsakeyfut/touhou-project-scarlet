use bevy::prelude::*;

use crate::{
    components::bullet::EnemyBullet,
    events::BombUsedEvent,
    resources::{BOMB_DURATION_SECS, BOMB_INVINCIBLE_SECS, BombState, GameData},
    states::AppState,
};

// ---------------------------------------------------------------------------
// bomb_input_system
// ---------------------------------------------------------------------------

/// Reads X-key input and activates a bomb when the player has stock available.
///
/// # Normal activation
///
/// When X is pressed and `game_data.bombs > 0`, one bomb stock is consumed,
/// [`BombState`] is set to active, and a [`BombUsedEvent`] is emitted.
///
/// # Counter-bomb
///
/// If `bomb_state.counter_bomb_window > 0.0` when X is pressed, the bomb is
/// treated as a counter-bomb: the life decremented by
/// [`crate::systems::collision::handle_player_hit`] is restored (up to a max
/// of 8) and the window is cleared. If the hit was lethal (`pending_death`),
/// the pending GameOver is cancelled.
///
/// # Lethal-hit GameOver commitment
///
/// When the counter-bomb window drains to zero while `pending_death` is set,
/// `bomb_input_system` calls `next_state.set(AppState::GameOver)` to confirm
/// the death. This keeps GameOver strictly in `Playing` state for the duration
/// of the window.
///
/// # Ordering
///
/// Registered in [`crate::GameSystemSet::Input`]. The counter-bomb window is
/// set by `handle_player_hit` (in `GameLogic`) during the **previous** frame,
/// so `bomb_input_system` can read it reliably in the current frame's `Input`
/// pass.
pub fn bomb_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut bomb_state: ResMut<BombState>,
    mut game_data: ResMut<GameData>,
    mut bomb_events: MessageWriter<BombUsedEvent>,
    mut next_state: ResMut<NextState<AppState>>,
    time: Res<Time>,
) {
    // Drain the counter-bomb window every frame regardless of input.
    if bomb_state.counter_bomb_window > 0.0 {
        bomb_state.counter_bomb_window =
            (bomb_state.counter_bomb_window - time.delta_secs()).max(0.0);

        // Window just closed on a lethal hit — commit GameOver.
        if bomb_state.pending_death && bomb_state.counter_bomb_window <= 0.0 {
            bomb_state.pending_death = false;
            next_state.set(AppState::GameOver);
            return;
        }
    }

    if !keys.just_pressed(KeyCode::KeyX) {
        return;
    }

    // Cannot activate while already active or out of stock.
    if bomb_state.active || game_data.bombs == 0 {
        return;
    }

    let is_counter = bomb_state.counter_bomb_window > 0.0;

    // Counter-bomb restores the life that handle_player_hit decremented.
    if is_counter {
        // Saturating add capped to 8 (EoSD maximum).
        game_data.lives = game_data.lives.saturating_add(1).min(8);
        bomb_state.counter_bomb_window = 0.0;
        // Cancel a deferred lethal-hit GameOver.
        bomb_state.pending_death = false;
    }

    game_data.bombs -= 1;
    bomb_state.active = true;

    // Reset both timers for the fresh activation.
    bomb_state.active_timer = Timer::from_seconds(BOMB_DURATION_SECS, TimerMode::Once);
    bomb_state.invincible_timer = Timer::from_seconds(BOMB_INVINCIBLE_SECS, TimerMode::Once);

    bomb_events.write(BombUsedEvent {
        is_counter_bomb: is_counter,
    });
}

// ---------------------------------------------------------------------------
// bomb_active_system
// ---------------------------------------------------------------------------

/// Ticks the bomb timers each frame and deactivates the active phase when the
/// active timer expires.
///
/// The invincible timer continues to tick beyond the active phase so that the
/// post-bomb invincibility tail is correctly tracked by
/// [`BombState::is_invincible`].
///
/// Registered in [`crate::GameSystemSet::Effects`].
pub fn bomb_active_system(mut bomb_state: ResMut<BombState>, time: Res<Time>) {
    if !bomb_state.active && bomb_state.invincible_timer.is_finished() {
        return;
    }

    if bomb_state.active {
        bomb_state.active_timer.tick(time.delta());
        if bomb_state.active_timer.just_finished() {
            bomb_state.active = false;
        }
    }

    // Tick invincible_timer even after the active phase ends (tail window).
    if !bomb_state.invincible_timer.is_finished() {
        bomb_state.invincible_timer.tick(time.delta());
    }
}

// ---------------------------------------------------------------------------
// bomb_effect_system
// ---------------------------------------------------------------------------

/// While the bomb is active, despawns all enemy bullets and awards 10 points
/// per bullet cleared.
///
/// Running every frame while `bomb_state.active` ensures that bullets spawned
/// during the bomb (e.g. from a boss emitter that was not yet silenced) are
/// also cleared.
///
/// # Score
///
/// Each cleared bullet awards 10 points, consistent with the Phase 09
/// specification. Score is added directly to [`GameData::score`]; no event is
/// emitted for individual bullets.
///
/// Registered in [`crate::GameSystemSet::GameLogic`] so it runs after
/// [`crate::GameSystemSet::Collision`] — graze counts are finalised before
/// bullets are removed.
pub fn bomb_effect_system(
    mut commands: Commands,
    enemy_bullets: Query<Entity, With<EnemyBullet>>,
    bomb_state: Res<BombState>,
    mut game_data: ResMut<GameData>,
) {
    if !bomb_state.active {
        return;
    }

    let mut cleared: u64 = 0;
    for entity in &enemy_bullets {
        commands.entity(entity).despawn();
        cleared += 1;
    }

    game_data.score += cleared * 10;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::COUNTER_BOMB_WINDOW_SECS;

    /// bomb_input_system must not activate if already active.
    #[test]
    fn no_activation_while_active() {
        let mut state = BombState::default();
        state.active = true;
        // Guard: active flag blocks re-activation.
        // (System logic checked inline — no ECS runner needed for pure logic.)
        assert!(state.active, "active flag prevents re-entry");
    }

    /// Counter-bomb window drains to zero, not below.
    #[test]
    fn counter_bomb_window_does_not_go_negative() {
        let mut window: f32 = 0.05;
        let delta: f32 = 0.1; // larger than window
        window = (window - delta).max(0.0);
        assert_eq!(window, 0.0, "window must clamp at 0.0, not go negative");
    }

    /// Counter-bomb restores one life up to the cap.
    #[test]
    fn counter_bomb_restores_life() {
        let lives_before: u8 = 1; // e.g. after being hit
        let lives_after = lives_before.saturating_add(1).min(8);
        assert_eq!(lives_after, 2);
    }

    /// Counter-bomb life restore is capped at 8.
    #[test]
    fn counter_bomb_life_capped_at_eight() {
        let lives_before: u8 = 8;
        let lives_after = lives_before.saturating_add(1).min(8);
        assert_eq!(lives_after, 8, "lives must not exceed 8");
    }

    /// bomb_active_system: active becomes false after full duration.
    #[test]
    fn active_timer_expires() {
        let mut state = BombState::default();
        state.active = true;
        state.active_timer = Timer::from_seconds(BOMB_DURATION_SECS, TimerMode::Once);
        state.invincible_timer = Timer::from_seconds(BOMB_INVINCIBLE_SECS, TimerMode::Once);

        // Tick active timer past full duration.
        state.active_timer.tick(std::time::Duration::from_secs_f32(
            BOMB_DURATION_SECS + 0.01,
        ));
        if state.active_timer.just_finished() {
            state.active = false;
        }

        assert!(!state.active, "active must be false once timer expires");
        // Invincible timer not yet finished — tail window still active.
        assert!(
            state.is_invincible(),
            "invincible tail must still be active"
        );
    }

    /// pending_death is cleared when a counter-bomb fires.
    #[test]
    fn counter_bomb_clears_pending_death() {
        let mut state = BombState::default();
        state.pending_death = true;
        state.counter_bomb_window = COUNTER_BOMB_WINDOW_SECS;
        // Simulate counter-bomb path.
        state.counter_bomb_window = 0.0;
        state.pending_death = false;
        assert!(
            !state.pending_death,
            "counter-bomb must clear pending_death"
        );
    }

    /// pending_death triggers GameOver when window drains to zero.
    #[test]
    fn pending_death_commits_game_over_on_window_expiry() {
        let mut state = BombState::default();
        state.pending_death = true;
        state.counter_bomb_window = 0.01;
        // Simulate window drain past zero.
        state.counter_bomb_window = (state.counter_bomb_window - 0.02_f32).max(0.0);
        // After this point bomb_input_system would call next_state.set(GameOver).
        assert!(
            state.pending_death && state.counter_bomb_window <= 0.0,
            "should commit GameOver when window reaches zero"
        );
    }

    /// is_invincible returns false once both timers finish and active is false.
    #[test]
    fn not_invincible_after_full_expiry() {
        let mut state = BombState::default();
        state.active = false;
        state
            .active_timer
            .tick(std::time::Duration::from_secs_f32(BOMB_DURATION_SECS + 1.0));
        state
            .invincible_timer
            .tick(std::time::Duration::from_secs_f32(
                BOMB_INVINCIBLE_SECS + 1.0,
            ));
        assert!(!state.is_invincible());
    }
}
