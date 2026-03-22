use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Timing constants
// ---------------------------------------------------------------------------

/// Duration of the active bomb effect in seconds (bullets cleared, damage dealt).
pub const BOMB_DURATION_SECS: f32 = 3.5;

/// Total invincibility window granted by a bomb in seconds.
///
/// Begins at the moment the bomb is activated and extends beyond `BOMB_DURATION_SECS`
/// so the player remains safe during the fade-out of the bomb effect.
pub const BOMB_INVINCIBLE_SECS: f32 = 5.0;

/// Counter-bomb window in seconds.
///
/// If the player presses X within this many seconds of taking a hit, the bomb
/// activates as a counter-bomb, negating the life loss.
pub const COUNTER_BOMB_WINDOW_SECS: f32 = 0.1;

// ---------------------------------------------------------------------------
// BombState
// ---------------------------------------------------------------------------

/// Global resource tracking the current bomb activation state.
///
/// Inserted as [`Default`] (inactive) at game start and reset between runs.
/// Written exclusively by `bomb_input_system` and `bomb_active_system`;
/// read by `player_hit_detection`, `bomb_effect_system`, and the UI.
///
/// # Life-cycle
///
/// ```text
/// Inactive
///   │  X pressed, bombs > 0
///   ▼
/// Active ──► active_timer ticks ──► active = false
///   │                                  │
///   └──── invincible_timer ticks ───────┘
///                     │
///              remaining_secs() == 0.0
///                     │
///              fully inactive again
/// ```
///
/// # Counter-bomb
///
/// When the player is hit, `handle_player_hit` sets
/// `counter_bomb_window = COUNTER_BOMB_WINDOW_SECS`. `bomb_input_system`
/// decrements that window each frame and checks it when X is pressed.
///
/// # Lethal-hit deferral (`pending_death`)
///
/// When a hit is lethal (`lives` would reach 0), `handle_player_hit` sets
/// `pending_death = true` instead of immediately calling
/// `NextState::set(GameOver)`. This keeps the game in `Playing` so that
/// `bomb_input_system` can still activate a counter-bomb within the open
/// window. Once `counter_bomb_window` drains to zero, `bomb_input_system`
/// commits the GameOver transition.
///
/// # Invincibility check
///
/// Use [`BombState::is_invincible`] rather than reading `active` directly so
/// that the post-bomb invincibility tail is included.
///
/// # Layout
///
/// | Field                  | Meaning |
/// |------------------------|---------|
/// | `active`               | `true` while the bomb effect is running. |
/// | `active_timer`         | Counts down `BOMB_DURATION_SECS`; finishes when the visual effect ends. |
/// | `invincible_timer`     | Counts down `BOMB_INVINCIBLE_SECS`; player is immune while `> 0`. |
/// | `counter_bomb_window`  | Remaining counter-bomb window in seconds (`0.0` when elapsed). |
/// | `pending_death`        | `true` after a lethal hit while the counter-bomb window is still open; GameOver is deferred until the window closes or a counter-bomb fires. |
#[derive(Resource)]
pub struct BombState {
    /// `true` while the bomb visual effect is active and bullets are being cleared.
    pub active: bool,
    /// Countdown for the bomb's active effect phase.
    ///
    /// Initialised to [`BOMB_DURATION_SECS`] on activation. The system
    /// `bomb_active_system` ticks this every frame and sets `active = false`
    /// when it finishes.
    pub active_timer: Timer,
    /// Countdown for the total invincibility window.
    ///
    /// Initialised to [`BOMB_INVINCIBLE_SECS`] on activation and ticked every
    /// frame. The player is immune to all incoming damage while
    /// `remaining_secs() > 0.0` **or** `active == true`.
    pub invincible_timer: Timer,
    /// Remaining counter-bomb window in seconds.
    ///
    /// Set to [`COUNTER_BOMB_WINDOW_SECS`] by `player_hit_detection` when a
    /// hit is registered. Decremented to `0.0` by `bomb_input_system` each
    /// frame.  If the player presses X while this is `> 0.0`, the activation
    /// is treated as a counter-bomb.
    pub counter_bomb_window: f32,
    /// `true` when a lethal hit has been registered but GameOver is deferred.
    ///
    /// Set by [`crate::systems::collision::handle_player_hit`] when
    /// `lives` reaches zero instead of immediately transitioning to
    /// [`crate::states::AppState::GameOver`]. This keeps the game in
    /// `Playing` state so `bomb_input_system` can still fire a counter-bomb
    /// within the open `counter_bomb_window`.
    ///
    /// Cleared by `bomb_input_system` either when:
    /// - A counter-bomb fires (hit cancelled, game continues), or
    /// - `counter_bomb_window` reaches `0.0` (GameOver confirmed).
    pub pending_death: bool,
}

impl Default for BombState {
    /// Returns an inactive `BombState` with fully-elapsed timers.
    ///
    /// Both `active_timer` and `invincible_timer` are ticked past their full
    /// duration so that `remaining_secs() == 0.0` and `is_invincible()` returns
    /// `false` immediately. The duration is preserved so the timers can be
    /// reset to their correct values when a bomb is actually activated.
    ///
    /// Inserted at game start by [`crate::ScarletCorePlugin`].
    fn default() -> Self {
        let mut active_timer = Timer::from_seconds(BOMB_DURATION_SECS, TimerMode::Once);
        let mut invincible_timer = Timer::from_seconds(BOMB_INVINCIBLE_SECS, TimerMode::Once);
        // Tick past the full duration so remaining_secs() == 0.0 from the start.
        active_timer.tick(std::time::Duration::from_secs_f32(BOMB_DURATION_SECS));
        invincible_timer.tick(std::time::Duration::from_secs_f32(BOMB_INVINCIBLE_SECS));
        Self {
            active: false,
            active_timer,
            invincible_timer,
            counter_bomb_window: 0.0,
            pending_death: false,
        }
    }
}

impl BombState {
    /// Returns `true` while the player is immune to incoming damage.
    ///
    /// Immunity is granted both during the active bomb effect and during the
    /// tail period after the effect ends (`invincible_timer` still running).
    ///
    /// # Usage
    ///
    /// ```rust,ignore
    /// if bomb_state.is_invincible() {
    ///     return; // skip damage processing
    /// }
    /// ```
    pub fn is_invincible(&self) -> bool {
        self.active || self.invincible_timer.remaining_secs() > 0.0
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Default BombState must be inactive with counter_bomb_window == 0.
    #[test]
    fn default_is_inactive() {
        let state = BombState::default();
        assert!(!state.active, "default BombState must not be active");
        assert_eq!(state.counter_bomb_window, 0.0);
        assert!(
            !state.pending_death,
            "default BombState must not be pending_death"
        );
        assert!(
            !state.is_invincible(),
            "default BombState must not be invincible"
        );
    }

    /// is_invincible must return false when inactive and timers have elapsed.
    #[test]
    fn not_invincible_when_inactive() {
        let mut state = BombState::default();
        // Manually finish both timers to simulate a fully elapsed state.
        state.active_timer.tick(std::time::Duration::from_secs_f32(
            BOMB_INVINCIBLE_SECS + 1.0,
        ));
        state
            .invincible_timer
            .tick(std::time::Duration::from_secs_f32(
                BOMB_INVINCIBLE_SECS + 1.0,
            ));
        assert!(!state.is_invincible());
    }

    /// is_invincible must return true while active == true.
    #[test]
    fn invincible_while_active() {
        let mut state = BombState::default();
        state.active = true;
        assert!(state.is_invincible());
    }

    /// is_invincible must return true while invincible_timer has remaining time.
    #[test]
    fn invincible_while_timer_running() {
        let mut state = BombState::default();
        // invincible_timer was just set — remaining_secs() > 0
        state.invincible_timer = Timer::from_seconds(BOMB_INVINCIBLE_SECS, TimerMode::Once);
        // Tick only a small amount so it has not yet finished.
        state
            .invincible_timer
            .tick(std::time::Duration::from_secs_f32(0.1));
        assert!(
            state.is_invincible(),
            "should be invincible while timer has time left"
        );
    }

    /// is_invincible must return false once both conditions clear.
    #[test]
    fn not_invincible_after_full_expiry() {
        let mut state = BombState::default();
        state.active = false;
        state
            .invincible_timer
            .tick(std::time::Duration::from_secs_f32(
                BOMB_INVINCIBLE_SECS + 1.0,
            ));
        assert!(!state.is_invincible());
    }

    /// BOMB_INVINCIBLE_SECS must be greater than BOMB_DURATION_SECS.
    #[test]
    fn invincible_period_exceeds_active_period() {
        assert!(
            BOMB_INVINCIBLE_SECS > BOMB_DURATION_SECS,
            "invincibility must outlast the active effect period"
        );
    }

    /// COUNTER_BOMB_WINDOW_SECS must be a short positive value.
    #[test]
    fn counter_bomb_window_is_short_positive() {
        assert!(COUNTER_BOMB_WINDOW_SECS > 0.0);
        assert!(
            COUNTER_BOMB_WINDOW_SECS < 0.5,
            "counter-bomb window should be < 0.5 s"
        );
    }
}
