/// Global resource that tracks the state of the current stage.
///
/// Inserted at game start by [`crate::ScarletCorePlugin`] with
/// [`StageData::default`]. Updated each frame by
/// [`crate::systems::stage::stage_control_system`].
///
/// # Fields
///
/// | Field | Description |
/// |---|---|
/// | `stage_number` | The 1-based index of the current stage (1 = Stage 1, etc.). |
/// | `elapsed_time` | Seconds elapsed since the stage began. |
/// | `boss_active` | `true` once the boss entity has been spawned and is still alive. |
/// | `boss_defeated` | `true` after the boss HP reaches zero; triggers stage clear. |
#[derive(bevy::prelude::Resource, Debug, Clone, PartialEq)]
pub struct StageData {
    /// 1-based stage index.
    pub stage_number: u8,
    /// Seconds elapsed since stage start. Incremented by `delta_secs` each frame.
    pub elapsed_time: f32,
    /// Set to `true` when the boss entity is spawned; cleared to `false` on defeat.
    pub boss_active: bool,
    /// Set to `true` permanently once the boss has been defeated.
    ///
    /// Used by `stage_control_system` to trigger the stage-clear transition.
    pub boss_defeated: bool,
}

impl Default for StageData {
    fn default() -> Self {
        Self {
            stage_number: 1,
            elapsed_time: 0.0,
            boss_active: false,
            boss_defeated: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Default StageData must start at stage 1 with zero elapsed time.
    #[test]
    fn default_stage_data() {
        let sd = StageData::default();
        assert_eq!(sd.stage_number, 1);
        assert_eq!(sd.elapsed_time, 0.0);
        assert!(!sd.boss_active);
        assert!(!sd.boss_defeated);
    }

    /// elapsed_time can be incremented as a plain f32 field.
    #[test]
    fn elapsed_time_is_mutable() {
        let mut sd = StageData::default();
        sd.elapsed_time += 1.0 / 60.0;
        assert!(sd.elapsed_time > 0.0);
    }
}
