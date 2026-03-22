use bevy::prelude::*;

use crate::components::bullet::{BulletPattern, EnemyBulletKind};

// ---------------------------------------------------------------------------
// BossType
// ---------------------------------------------------------------------------

/// Identifies which boss character this entity represents.
///
/// Used to look up character-specific data (spell-card colours, dialogue, etc.)
/// and to route boss-spawn logic in stage scripts.
///
/// | Variant | Stage | Character |
/// |---|---|---|
/// | `Rumia` | 1 | Rumia |
/// | `Cirno` | 2 | Cirno |
/// | `Meiling` | 3 | Hong Meiling |
/// | `Patchouli` | 4 | Patchouli Knowledge |
/// | `Sakuya` | 5 | Sakuya Izayoi |
/// | `Remilia` | 6 | Remilia Scarlet |
/// | `Flandre` | Extra | Flandre Scarlet |
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BossType {
    /// Stage 1 boss: Rumia.
    Rumia,
    /// Stage 2 boss: Cirno.
    Cirno,
    /// Stage 3 boss: Hong Meiling.
    Meiling,
    /// Stage 4 boss: Patchouli Knowledge.
    Patchouli,
    /// Stage 5 boss: Sakuya Izayoi.
    Sakuya,
    /// Stage 6 boss: Remilia Scarlet.
    Remilia,
    /// Extra stage boss: Flandre Scarlet.
    Flandre,
}

impl BossType {
    /// Returns the `(pattern_id, primary_color, secondary_color)` tuple used
    /// by `SpellCardBgMaterial` to render the spell-card background shader.
    ///
    /// `pattern_id` selects the WGSL pattern branch (0–6).
    /// Colors use HDR `LinearRgba` values (> 1.0 for bloom).
    pub fn spell_card_colors(&self) -> (u32, LinearRgba, LinearRgba) {
        match self {
            Self::Rumia => (
                0,
                LinearRgba::new(0.15, 0.05, 0.25, 0.7),
                LinearRgba::new(0.02, 0.01, 0.05, 0.9),
            ),
            Self::Cirno => (
                1,
                LinearRgba::new(0.5, 0.9, 1.0, 0.6),
                LinearRgba::new(0.8, 0.95, 1.0, 0.4),
            ),
            Self::Meiling => (
                2,
                LinearRgba::new(1.0, 0.5, 0.2, 0.6),
                LinearRgba::new(0.2, 0.8, 0.4, 0.5),
            ),
            Self::Patchouli => (
                3,
                LinearRgba::new(0.5, 0.2, 0.8, 0.7),
                LinearRgba::new(0.8, 0.7, 0.1, 0.5),
            ),
            Self::Sakuya => (
                4,
                LinearRgba::new(0.9, 0.9, 0.95, 0.5),
                LinearRgba::new(0.6, 0.7, 0.8, 0.4),
            ),
            Self::Remilia => (
                5,
                LinearRgba::new(0.7, 0.05, 0.1, 0.7),
                LinearRgba::new(0.1, 0.02, 0.02, 0.9),
            ),
            Self::Flandre => (
                6,
                LinearRgba::new(1.0, 0.3, 0.0, 0.6),
                LinearRgba::new(0.5, 0.0, 0.5, 0.5),
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// BossMovement
// ---------------------------------------------------------------------------

/// Movement pattern applied to a boss entity each frame.
///
/// Stored inside [`BossPhaseData`] so that each phase can have a different
/// movement style. The `boss_movement_system` (Issue #60) reads this variant
/// and updates the boss [`Transform`] accordingly.
///
/// | Variant | Description |
/// |---|---|
/// | `Static` | Does not move; stays at spawn position. |
/// | `Pendulum` | Swings horizontally about `base_x`. |
/// | `Circle` | Orbits a fixed `center` at `speed_deg` degrees/s. |
/// | `Teleport` | Snaps through a list of `waypoints` with a wait between each. |
#[derive(Clone, Debug)]
pub enum BossMovement {
    /// The boss does not move at all.
    Static,

    /// Oscillates horizontally around `base_x`.
    ///
    /// X position is `base_x + amplitude * sin(elapsed * frequency * 2π)`.
    Pendulum {
        /// Peak horizontal offset from `base_x` in pixels.
        amplitude: f32,
        /// Oscillation frequency in Hz (cycles per second).
        frequency: f32,
        /// Centre X coordinate for the oscillation.
        base_x: f32,
    },

    /// Orbits a fixed point at a constant angular speed.
    Circle {
        /// Orbit radius in pixels.
        radius: f32,
        /// Angular speed in degrees per second (positive = counter-clockwise).
        speed_deg: f32,
        /// World-space centre of the orbit.
        center: Vec2,
    },

    /// Teleports between a list of `waypoints`, waiting at each.
    ///
    /// The `boss_movement_system` advances `current` and resets `wait_timer`
    /// every time the timer expires.
    Teleport {
        /// Ordered list of world-space target positions.
        waypoints: Vec<Vec2>,
        /// Seconds to wait at each waypoint before teleporting to the next.
        wait_secs: f32,
        /// Elapsed wait time at the current waypoint (mutated by the system).
        elapsed_secs: f32,
        /// Index of the current waypoint. Mutated by the movement system.
        current: usize,
    },
}

// ---------------------------------------------------------------------------
// BossPhaseData
// ---------------------------------------------------------------------------

/// Data describing a single boss phase (normal attack or spell card).
///
/// A boss entity holds a `Vec<BossPhaseData>` inside [`Boss::phases`].
/// The `boss_phase_system` (Issue #58) advances `Boss::current_phase` when
/// the current phase's HP reaches zero or the time limit expires.
#[derive(Clone, Debug)]
pub struct BossPhaseData {
    /// Current HP for this phase. The phase ends when this reaches ≤ 0.
    pub hp: f32,
    /// Maximum HP at the start of this phase (for HP-bar rendering).
    pub hp_max: f32,
    /// `true` when this phase is a spell card (enables spell-card bonus and
    /// the spell-card background shader).
    pub is_spell_card: bool,
    /// Display name of the spell card, or `None` for non-spell phases.
    pub spell_card_name: Option<String>,
    /// Time limit in seconds. The phase ends early if this elapses.
    pub time_limit_secs: f32,
    /// Bullet pattern fired by the boss's [`BulletEmitter`] during this phase.
    pub pattern: BulletPattern,
    /// Bullet variant spawned by the emitter during this phase.
    pub bullet_kind: EnemyBulletKind,
    /// Fire interval in seconds for the [`BulletEmitter`] during this phase.
    pub fire_interval_secs: f32,
    /// Movement pattern applied to the boss entity during this phase.
    pub movement: BossMovement,
    /// Score bonus awarded when the player defeats a spell-card phase without
    /// bombing or getting hit. Only meaningful when `is_spell_card` is `true`.
    pub spell_card_bonus: u32,
}

// ---------------------------------------------------------------------------
// Boss
// ---------------------------------------------------------------------------

/// Core component for boss entities.
///
/// Attach to a boss entity alongside [`crate::components::enemy::Enemy`],
/// [`BulletEmitter`], [`Transform`], and [`Sprite`] (or a mesh).
///
/// The `boss_phase_system` (Issue #58) drives phase transitions by:
/// 1. Ticking `phase_timer` each frame.
/// 2. Checking whether `phases[current_phase].hp ≤ 0` or `phase_timer` fired.
/// 3. Awarding the spell-card bonus and advancing `current_phase`.
/// 4. Emitting `BossPhaseChangedEvent` so other systems can react.
#[derive(Component)]
pub struct Boss {
    /// Which boss character this entity represents.
    pub boss_type: BossType,
    /// Index into `phases` for the currently active phase (0-based).
    pub current_phase: usize,
    /// All phases for this boss, in order.
    ///
    /// Must contain at least one entry. The phase system despawns the boss
    /// once `current_phase` advances past the last entry.
    pub phases: Vec<BossPhaseData>,
    /// Countdown timer for the current phase. Reset when the phase changes.
    pub phase_timer: Timer,
    /// `true` while a spell-card phase is active.
    ///
    /// Used by the UI to display the spell-card name overlay and by the
    /// `SpellCardBgMaterial` system to activate the background shader.
    pub spell_card_active: bool,
}

impl Boss {
    /// Creates a new `Boss` from its phase list, starting at phase 0.
    ///
    /// # Panics
    ///
    /// Panics if `phases` is empty.
    pub fn new(boss_type: BossType, phases: Vec<BossPhaseData>) -> Self {
        assert!(!phases.is_empty(), "Boss must have at least one phase");
        let first_time = phases[0].time_limit_secs;
        let first_is_spell = phases[0].is_spell_card;
        Self {
            boss_type,
            current_phase: 0,
            phases,
            phase_timer: Timer::from_seconds(first_time, TimerMode::Once),
            spell_card_active: first_is_spell,
        }
    }

    /// Returns a reference to the currently active [`BossPhaseData`].
    pub fn current(&self) -> &BossPhaseData {
        &self.phases[self.current_phase]
    }

    /// Returns a mutable reference to the currently active [`BossPhaseData`].
    pub fn current_mut(&mut self) -> &mut BossPhaseData {
        &mut self.phases[self.current_phase]
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_phase(hp: f32, is_spell_card: bool) -> BossPhaseData {
        BossPhaseData {
            hp,
            hp_max: hp,
            is_spell_card,
            spell_card_name: None,
            time_limit_secs: 30.0,
            pattern: BulletPattern::Ring {
                count: 8,
                speed: 100.0,
            },
            bullet_kind: EnemyBulletKind::SmallRound,
            fire_interval_secs: 0.5,
            movement: BossMovement::Static,
            spell_card_bonus: 0,
        }
    }

    /// `Boss::new` must initialise `current_phase` to 0.
    #[test]
    fn boss_new_starts_at_phase_zero() {
        let boss = Boss::new(BossType::Rumia, vec![make_phase(500.0, false)]);
        assert_eq!(boss.current_phase, 0);
    }

    /// `Boss::new` must start the phase timer with the first phase's time limit.
    #[test]
    fn boss_new_initialises_timer_from_first_phase() {
        let boss = Boss::new(BossType::Rumia, vec![make_phase(500.0, false)]);
        assert!((boss.phase_timer.duration().as_secs_f32() - 30.0).abs() < 1e-6);
    }

    /// `spell_card_active` must reflect the first phase's `is_spell_card`.
    #[test]
    fn boss_spell_card_active_matches_first_phase() {
        let normal = Boss::new(BossType::Rumia, vec![make_phase(500.0, false)]);
        assert!(!normal.spell_card_active);

        let spell = Boss::new(BossType::Rumia, vec![make_phase(500.0, true)]);
        assert!(spell.spell_card_active);
    }

    /// `Boss::current()` must return the active phase.
    #[test]
    fn boss_current_returns_active_phase() {
        let boss = Boss::new(BossType::Cirno, vec![make_phase(300.0, false)]);
        assert!((boss.current().hp - 300.0).abs() < 1e-6);
    }

    /// `Boss::new` must panic when given an empty phase list.
    #[test]
    #[should_panic(expected = "Boss must have at least one phase")]
    fn boss_new_panics_on_empty_phases() {
        let _ = Boss::new(BossType::Rumia, vec![]);
    }

    /// All `BossType` variants must return a unique `pattern_id`.
    #[test]
    fn boss_type_spell_card_colors_unique_pattern_ids() {
        let types = [
            BossType::Rumia,
            BossType::Cirno,
            BossType::Meiling,
            BossType::Patchouli,
            BossType::Sakuya,
            BossType::Remilia,
            BossType::Flandre,
        ];
        let ids: Vec<u32> = types.iter().map(|t| t.spell_card_colors().0).collect();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            ids.len(),
            "each BossType must have a unique pattern_id"
        );
    }

    /// `BossType::spell_card_colors` primary alpha must be > 0.
    #[test]
    fn boss_type_spell_card_colors_have_positive_alpha() {
        for t in [
            BossType::Rumia,
            BossType::Cirno,
            BossType::Meiling,
            BossType::Patchouli,
            BossType::Sakuya,
            BossType::Remilia,
            BossType::Flandre,
        ] {
            let (_, primary, secondary) = t.spell_card_colors();
            assert!(primary.alpha > 0.0, "{t:?} primary alpha must be > 0");
            assert!(secondary.alpha > 0.0, "{t:?} secondary alpha must be > 0");
        }
    }
}
