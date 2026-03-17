use bevy::prelude::*;

/// Global game state shared across all in-game systems.
///
/// Inserted at the start of a run and read/written by scoring, power,
/// and life-management systems throughout the game.
///
/// # Field ranges
///
/// | Field       | Type  | Range / notes                          |
/// |-------------|-------|----------------------------------------|
/// | `score`     | `u64` | Unbounded; display with comma separators |
/// | `hi_score`  | `u64` | Persisted between runs                 |
/// | `lives`     | `u8`  | Starting value: 2 (Phase 03 default)   |
/// | `bombs`     | `u8`  | Starting value: 3                      |
/// | `power`     | `u8`  | 0–128; affects bullet count/spread     |
/// | `graze`     | `u32` | Total graze count for the current run  |
#[derive(Resource, Default)]
pub struct GameData {
    /// Current score for this run.
    pub score: u64,
    /// All-time high score (loaded from save on startup).
    pub hi_score: u64,
    /// Remaining lives (extra lives can be earned at score thresholds).
    pub lives: u8,
    /// Remaining bomb stocks.
    pub bombs: u8,
    /// Power level (0–128). Determines bullet count and spread.
    pub power: u8,
    /// Total graze count accumulated this run.
    pub graze: u32,
}

impl GameData {
    /// Returns a `GameData` initialised to standard EoSD starting values.
    pub fn new_game() -> Self {
        Self {
            score: 0,
            hi_score: 0,
            lives: 2,
            bombs: 3,
            power: 0,
            graze: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_game_has_correct_defaults() {
        let gd = GameData::new_game();
        assert_eq!(gd.lives, 2);
        assert_eq!(gd.bombs, 3);
        assert_eq!(gd.power, 0);
        assert_eq!(gd.score, 0);
    }
}
