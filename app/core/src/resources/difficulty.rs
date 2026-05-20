/// Selectable difficulty levels, matching the original EoSD nomenclature.
///
/// Inserted as a [`bevy::prelude::Resource`] at game start via
/// [`crate::ScarletCorePlugin`] with [`Difficulty::Normal`] as the default.
/// Future phases will add a `DifficultySelect` UI screen that writes this
/// resource before entering `AppState::Playing`.
#[derive(bevy::prelude::Resource, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Difficulty {
    Easy,
    #[default]
    Normal,
    Hard,
    Lunatic,
}

impl Difficulty {
    /// Short display label used in UI and debug output.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Easy => "Easy",
            Self::Normal => "Normal",
            Self::Hard => "Hard",
            Self::Lunatic => "Lunatic",
        }
    }
}

// ---------------------------------------------------------------------------
// DifficultyParams
// ---------------------------------------------------------------------------

/// Per-difficulty scaling parameters applied to enemy and boss statistics.
///
/// All multipliers are relative to the Normal baseline (all fields = 1.0).
///
/// # Usage
///
/// ```rust,ignore
/// let params = DifficultyParams::for_difficulty(Difficulty::Hard);
/// let scaled_hp = base_hp * params.boss_hp_multiplier;
/// ```
#[derive(Debug, Clone, Copy)]
pub struct DifficultyParams {
    /// HP multiplier for regular (non-boss) enemies.
    pub enemy_hp_multiplier: f32,
    /// HP multiplier for boss phases.
    pub boss_hp_multiplier: f32,
    /// Time-limit multiplier for boss phases.
    ///
    /// Values > 1.0 give the player more time (easier); < 1.0 increase pressure.
    pub boss_time_multiplier: f32,
    /// Speed multiplier applied to all bullet patterns on boss phases.
    pub bullet_speed_multiplier: f32,
}

impl DifficultyParams {
    /// Returns the scaling parameters for `difficulty`.
    pub fn for_difficulty(difficulty: Difficulty) -> Self {
        match difficulty {
            Difficulty::Easy => Self {
                enemy_hp_multiplier: 0.7,
                boss_hp_multiplier: 0.7,
                boss_time_multiplier: 1.2,
                bullet_speed_multiplier: 0.8,
            },
            Difficulty::Normal => Self {
                enemy_hp_multiplier: 1.0,
                boss_hp_multiplier: 1.0,
                boss_time_multiplier: 1.0,
                bullet_speed_multiplier: 1.0,
            },
            Difficulty::Hard => Self {
                enemy_hp_multiplier: 1.3,
                boss_hp_multiplier: 1.3,
                boss_time_multiplier: 0.85,
                bullet_speed_multiplier: 1.2,
            },
            Difficulty::Lunatic => Self {
                enemy_hp_multiplier: 1.8,
                boss_hp_multiplier: 1.8,
                boss_time_multiplier: 0.7,
                bullet_speed_multiplier: 1.5,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_difficulty_has_unit_multipliers() {
        let p = DifficultyParams::for_difficulty(Difficulty::Normal);
        assert_eq!(p.enemy_hp_multiplier, 1.0);
        assert_eq!(p.boss_hp_multiplier, 1.0);
        assert_eq!(p.boss_time_multiplier, 1.0);
        assert_eq!(p.bullet_speed_multiplier, 1.0);
    }

    #[test]
    fn easy_is_softer_than_normal() {
        let e = DifficultyParams::for_difficulty(Difficulty::Easy);
        let n = DifficultyParams::for_difficulty(Difficulty::Normal);
        assert!(e.enemy_hp_multiplier < n.enemy_hp_multiplier);
        assert!(e.boss_hp_multiplier < n.boss_hp_multiplier);
        assert!(e.boss_time_multiplier > n.boss_time_multiplier);
        assert!(e.bullet_speed_multiplier < n.bullet_speed_multiplier);
    }

    #[test]
    fn lunatic_is_hardest() {
        let h = DifficultyParams::for_difficulty(Difficulty::Hard);
        let l = DifficultyParams::for_difficulty(Difficulty::Lunatic);
        assert!(l.boss_hp_multiplier > h.boss_hp_multiplier);
        assert!(l.bullet_speed_multiplier > h.bullet_speed_multiplier);
        assert!(l.boss_time_multiplier < h.boss_time_multiplier);
    }

    #[test]
    fn all_difficulties_have_positive_multipliers() {
        for d in [
            Difficulty::Easy,
            Difficulty::Normal,
            Difficulty::Hard,
            Difficulty::Lunatic,
        ] {
            let p = DifficultyParams::for_difficulty(d);
            assert!(
                p.enemy_hp_multiplier > 0.0,
                "{} enemy_hp must be positive",
                d.label()
            );
            assert!(
                p.boss_hp_multiplier > 0.0,
                "{} boss_hp must be positive",
                d.label()
            );
            assert!(
                p.boss_time_multiplier > 0.0,
                "{} time must be positive",
                d.label()
            );
            assert!(
                p.bullet_speed_multiplier > 0.0,
                "{} bullet_speed must be positive",
                d.label()
            );
        }
    }

    #[test]
    fn default_difficulty_is_normal() {
        assert_eq!(Difficulty::default(), Difficulty::Normal);
    }

    #[test]
    fn labels_are_nonempty() {
        for d in [
            Difficulty::Easy,
            Difficulty::Normal,
            Difficulty::Hard,
            Difficulty::Lunatic,
        ] {
            assert!(!d.label().is_empty());
        }
    }
}
