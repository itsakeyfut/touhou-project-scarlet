use bevy::prelude::*;

/// Causes an entity's [`Sprite`] to fade from full opacity to transparent
/// over a fixed duration, then despawns the entity.
///
/// Attach this component to any entity with a [`Sprite`] that should
/// gracefully fade out rather than instantly disappear.
///
/// # Life-cycle
///
/// ```text
/// Spawned with FadeOut
///   │  fade_out_system ticks timer every frame
///   │  α = 1.0 − fraction()
///   ▼
/// timer.just_finished() → despawn
/// ```
///
/// # Usage
///
/// ```rust,ignore
/// commands.entity(entity).insert(FadeOut::new(0.3));
/// ```
///
/// # Fields
///
/// | Field   | Description |
/// |---------|-------------|
/// | `timer` | Tracks elapsed / duration; exposes `.elapsed_secs()` and `.duration()` |
#[derive(Component, Debug, Clone)]
pub struct FadeOut {
    /// Countdown timer — `elapsed_secs()` / `duration()` give the current
    /// progress fraction used to compute the alpha value each frame.
    pub timer: Timer,
}

impl FadeOut {
    /// Creates a new [`FadeOut`] that completes in `duration_secs` seconds.
    ///
    /// # Panics
    ///
    /// Panics if `duration_secs` is not finite or is not greater than `0.0`.
    /// Passing zero, a negative value, or a non-finite value (NaN / infinity)
    /// would create a timer that is immediately finished, causing the entity to
    /// be despawned on the first tick without any visible fade.
    pub fn new(duration_secs: f32) -> Self {
        assert!(
            duration_secs.is_finite() && duration_secs > 0.0,
            "FadeOut duration must be finite and > 0.0 (got {duration_secs})"
        );
        Self {
            timer: Timer::from_seconds(duration_secs, TimerMode::Once),
        }
    }

    /// Total duration of the fade in seconds.
    pub fn duration_secs(&self) -> f32 {
        self.timer.duration().as_secs_f32()
    }

    /// Seconds elapsed since the fade began.
    pub fn elapsed_secs(&self) -> f32 {
        self.timer.elapsed_secs()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// A freshly created FadeOut has zero elapsed time.
    #[test]
    fn new_fade_out_starts_at_zero_elapsed() {
        let fade = FadeOut::new(1.0);
        assert_eq!(fade.elapsed_secs(), 0.0);
    }

    /// duration_secs matches the value passed to new().
    #[test]
    fn duration_matches_constructor() {
        let fade = FadeOut::new(2.5);
        assert!((fade.duration_secs() - 2.5).abs() < 1e-6);
    }

    /// Alpha fraction should be 1.0 at the start and 0.0 at finish.
    #[test]
    fn alpha_fraction_range() {
        let mut fade = FadeOut::new(1.0);
        // Start: fraction == 0.0 → alpha == 1.0.
        assert_eq!(fade.timer.fraction(), 0.0);
        assert!((1.0 - fade.timer.fraction() - 1.0).abs() < 1e-6);

        // Tick almost to the end.
        fade.timer.tick(std::time::Duration::from_secs_f32(0.999));
        let alpha = 1.0 - fade.timer.fraction();
        assert!(
            alpha >= 0.0 && alpha <= 1.0,
            "alpha must stay in [0, 1] during fade, got {alpha}"
        );

        // Tick past the end.
        fade.timer.tick(std::time::Duration::from_secs_f32(0.1));
        assert!(fade.timer.is_finished());
    }

    /// Positive duration must be accepted.
    #[test]
    fn new_with_short_duration() {
        let fade = FadeOut::new(0.3);
        assert!((fade.duration_secs() - 0.3).abs() < 1e-6);
        assert!(!fade.timer.is_finished());
    }
}
