use bevy::prelude::*;

use crate::{
    components::{
        GameSessionEntity,
        boss::{Boss, BossMovement, BossPhaseData, BossType},
        bullet::{BulletEmitter, BulletPattern, EnemyBulletKind},
        enemy::Enemy,
    },
    events::BossSpawnEvent,
    resources::{Difficulty, DifficultyParams},
};

// ---------------------------------------------------------------------------
// Phase constants
// ---------------------------------------------------------------------------

/// Collision radius for Rumia's hitbox (px).
const RUMIA_COLLISION_RADIUS: f32 = 22.0;

// Phase 0 — non-spell normal attack.
const P0_HP: f32 = 600.0;
const P0_TIME: f32 = 30.0;
const P0_FIRE_INTERVAL: f32 = 1.0;

// Phase 1 — spell card "夜符「ナイトバード」".
const P1_HP: f32 = 500.0;
const P1_TIME: f32 = 40.0;
const P1_FIRE_INTERVAL: f32 = 0.8;
const P1_BONUS: u32 = 500_000;

// Phase 2 — spell card "闇符「ディマーケーション」".
const P2_HP: f32 = 700.0;
const P2_TIME: f32 = 50.0;
const P2_FIRE_INTERVAL: f32 = 0.65;
const P2_BONUS: u32 = 1_000_000;

// ---------------------------------------------------------------------------
// rumia_phases
// ---------------------------------------------------------------------------

/// Returns the ordered [`BossPhaseData`] list for Rumia (Stage 1 boss).
///
/// # Phase layout
///
/// | # | Type | Spell card | Pattern | Movement |
/// |---|------|-----------|---------|----------|
/// | 0 | Non-spell | — | 3-way aimed | Pendulum |
/// | 1 | Spell card | 夜符「ナイトバード」 | 5-way aimed (Rice) | Circle orbit |
/// | 2 | Spell card | 闇符「ディマーケーション」 | 16-Ring burst | Wide pendulum |
///
/// ## Phase 0 — Non-spell
///
/// A slow 3-way aimed spread. Introductory difficulty; gives the player time
/// to learn Rumia's hitbox and pendulum rhythm.
///
/// ## Phase 1 — 夜符「ナイトバード」(Night Sign "Night Bird")
///
/// Five targeted Rice bullets in a tight spread fired while Rumia orbits a
/// fixed centre above mid-screen. The pattern evokes a bird swooping toward
/// the player.
///
/// ## Phase 2 — 闇符「ディマーケーション」(Dark Sign "Demarcation")
///
/// 16-bullet rings expand outward while Rumia sweeps wide horizontally. The
/// pattern references the original Touhou game's Demarcation spell.
pub fn rumia_phases() -> Vec<BossPhaseData> {
    vec![
        // ------------------------------------------------------------------
        // Phase 0: Non-spell — 3-way slow aimed attack.
        // ------------------------------------------------------------------
        BossPhaseData {
            hp: P0_HP,
            hp_max: P0_HP,
            is_spell_card: false,
            spell_card_name: None,
            time_limit_secs: P0_TIME,
            pattern: BulletPattern::Aimed {
                count: 3,
                spread_deg: 20.0,
                speed: 110.0,
            },
            bullet_kind: EnemyBulletKind::MediumRound,
            fire_interval_secs: P0_FIRE_INTERVAL,
            movement: BossMovement::Pendulum {
                amplitude: 80.0,
                frequency: 0.4,
                base_x: 0.0,
            },
            spell_card_bonus: 0,
        },
        // ------------------------------------------------------------------
        // Phase 1: 夜符「ナイトバード」— 5-way tight aimed (Rice bullets).
        // ------------------------------------------------------------------
        BossPhaseData {
            hp: P1_HP,
            hp_max: P1_HP,
            is_spell_card: true,
            spell_card_name: Some("夜符「ナイトバード」".to_string()),
            time_limit_secs: P1_TIME,
            pattern: BulletPattern::Aimed {
                count: 5,
                spread_deg: 28.0,
                speed: 130.0,
            },
            bullet_kind: EnemyBulletKind::Rice,
            fire_interval_secs: P1_FIRE_INTERVAL,
            movement: BossMovement::Circle {
                radius: 60.0,
                speed_deg: 45.0,
                center: Vec2::new(0.0, 80.0),
            },
            spell_card_bonus: P1_BONUS,
        },
        // ------------------------------------------------------------------
        // Phase 2: 闇符「ディマーケーション」— 16-ring expanding burst.
        // ------------------------------------------------------------------
        BossPhaseData {
            hp: P2_HP,
            hp_max: P2_HP,
            is_spell_card: true,
            spell_card_name: Some("闇符「ディマーケーション」".to_string()),
            time_limit_secs: P2_TIME,
            pattern: BulletPattern::Ring {
                count: 16,
                speed: 95.0,
            },
            bullet_kind: EnemyBulletKind::MediumRound,
            fire_interval_secs: P2_FIRE_INTERVAL,
            movement: BossMovement::Pendulum {
                amplitude: 110.0,
                frequency: 0.3,
                base_x: 0.0,
            },
            spell_card_bonus: P2_BONUS,
        },
    ]
}

// ---------------------------------------------------------------------------
// Difficulty scaling helpers
// ---------------------------------------------------------------------------

/// Returns Rumia's phase list scaled to the given difficulty.
///
/// Applies [`DifficultyParams`] multipliers to HP, time limits, and bullet
/// speeds. The Normal baseline (`rumia_phases()`) is always the source, so
/// round-trips through this function are idempotent.
pub fn rumia_phases_for_difficulty(difficulty: Difficulty) -> Vec<BossPhaseData> {
    let params = DifficultyParams::for_difficulty(difficulty);
    scale_phases(rumia_phases(), &params)
}

/// Applies `params` multipliers to every phase in `phases`.
fn scale_phases(phases: Vec<BossPhaseData>, params: &DifficultyParams) -> Vec<BossPhaseData> {
    phases
        .into_iter()
        .map(|mut p| {
            p.hp = (p.hp * params.boss_hp_multiplier).ceil();
            p.hp_max = p.hp;
            p.time_limit_secs *= params.boss_time_multiplier;
            p.pattern = scale_pattern_speed(p.pattern, params.bullet_speed_multiplier);
            p
        })
        .collect()
}

/// Multiplies the bullet speed component inside a [`BulletPattern`] by `mult`.
fn scale_pattern_speed(pattern: BulletPattern, mult: f32) -> BulletPattern {
    match pattern {
        BulletPattern::Aimed {
            count,
            spread_deg,
            speed,
        } => BulletPattern::Aimed {
            count,
            spread_deg,
            speed: speed * mult,
        },
        BulletPattern::Ring { count, speed } => BulletPattern::Ring {
            count,
            speed: speed * mult,
        },
        BulletPattern::Spread {
            count,
            spread_deg,
            speed,
            angle_offset,
        } => BulletPattern::Spread {
            count,
            spread_deg,
            speed: speed * mult,
            angle_offset,
        },
        BulletPattern::Spiral {
            arms,
            speed,
            rotation_speed_deg,
        } => BulletPattern::Spiral {
            arms,
            speed: speed * mult,
            rotation_speed_deg,
        },
    }
}

// ---------------------------------------------------------------------------
// spawn_rumia  (consumed by on_boss_spawn_stage1)
// ---------------------------------------------------------------------------

/// Spawns the Rumia boss entity with phases scaled to `difficulty`.
///
/// Called by [`on_boss_spawn_stage1`] when a [`BossSpawnEvent`] arrives for
/// stage 1. The entity receives:
///
/// - [`Boss`] — phase list, timer, spell-card flag.
/// - [`Enemy`] — collision radius and `is_boss = true` flag.
/// - [`BulletEmitter`] — first-phase pattern; swapped by
///   [`crate::systems::boss::phase::update_boss_emitter_on_phase_change`] on
///   each phase transition.
/// - [`GameSessionEntity`] — automatic cleanup on session end.
/// - [`Transform`] — spawns at (0, 120) just below the top of the play area.
///
/// Visual representation is a placeholder coloured sprite until Phase 19
/// replaces it with the proper sprite sheet.
pub fn spawn_rumia(commands: &mut Commands, difficulty: Difficulty) {
    let phases = rumia_phases_for_difficulty(difficulty);
    let first = &phases[0];
    let first_hp = first.hp;
    let first_pattern = first.pattern.clone();
    let first_bullet_kind = first.bullet_kind;
    let first_fire_interval = first.fire_interval_secs;

    commands.spawn((
        Boss::new(BossType::Rumia, phases),
        Enemy {
            hp: first_hp,
            hp_max: first_hp,
            collision_radius: RUMIA_COLLISION_RADIUS,
            score_value: 0, // boss score is awarded per-phase via spell_card_bonus
            is_boss: true,
        },
        BulletEmitter {
            pattern: first_pattern,
            bullet_kind: first_bullet_kind,
            timer: Timer::from_seconds(first_fire_interval, TimerMode::Repeating),
            active: true,
        },
        GameSessionEntity,
        // Placeholder visual: purple-tinted rectangle until sprite assets land.
        Sprite {
            color: Color::srgb(0.25, 0.1, 0.35),
            custom_size: Some(Vec2::splat(40.0)),
            ..default()
        },
        // Spawn just below the visible top edge of the play area.
        Transform::from_xyz(0.0, 120.0, 1.0),
    ));
}

// ---------------------------------------------------------------------------
// on_boss_spawn_stage1
// ---------------------------------------------------------------------------

/// System that reacts to [`BossSpawnEvent`] and spawns Rumia for stage 1.
///
/// Reads the current [`Difficulty`] resource to select appropriately scaled
/// boss phases. Registered in [`crate::GameSystemSet::StageControl`].
pub fn on_boss_spawn_stage1(
    mut commands: Commands,
    mut boss_events: MessageReader<BossSpawnEvent>,
    difficulty: Res<Difficulty>,
) {
    for event in boss_events.read() {
        if event.stage_number != 1 {
            continue;
        }
        spawn_rumia(&mut commands, *difficulty);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// rumia_phases must return exactly 3 phases.
    #[test]
    fn rumia_has_three_phases() {
        let phases = rumia_phases();
        assert_eq!(phases.len(), 3, "Rumia must have exactly 3 phases");
    }

    /// Phase 0 must be a non-spell phase.
    #[test]
    fn phase_0_is_non_spell() {
        let phases = rumia_phases();
        assert!(
            !phases[0].is_spell_card,
            "phase 0 (non-spell) must not be a spell card"
        );
        assert!(phases[0].spell_card_name.is_none());
        assert_eq!(phases[0].spell_card_bonus, 0);
    }

    /// Phase 1 must be the Night Bird spell card.
    #[test]
    fn phase_1_is_night_bird_spell_card() {
        let phases = rumia_phases();
        let p = &phases[1];
        assert!(p.is_spell_card, "phase 1 must be a spell card");
        assert_eq!(p.spell_card_name.as_deref(), Some("夜符「ナイトバード」"));
        assert_eq!(p.spell_card_bonus, P1_BONUS);
    }

    /// Phase 2 must be the Demarcation spell card.
    #[test]
    fn phase_2_is_demarcation_spell_card() {
        let phases = rumia_phases();
        let p = &phases[2];
        assert!(p.is_spell_card, "phase 2 must be a spell card");
        assert_eq!(
            p.spell_card_name.as_deref(),
            Some("闇符「ディマーケーション」")
        );
        assert_eq!(p.spell_card_bonus, P2_BONUS);
    }

    /// All phases must have positive HP and positive time limits.
    #[test]
    fn all_phases_have_positive_hp_and_time() {
        for (i, phase) in rumia_phases().iter().enumerate() {
            assert!(phase.hp > 0.0, "phase {i} hp must be positive");
            assert!(phase.hp_max > 0.0, "phase {i} hp_max must be positive");
            assert!(
                (phase.hp - phase.hp_max).abs() < 1e-6,
                "phase {i} hp must equal hp_max at start"
            );
            assert!(
                phase.time_limit_secs > 0.0,
                "phase {i} time limit must be positive"
            );
            assert!(
                phase.fire_interval_secs > 0.0,
                "phase {i} fire interval must be positive"
            );
        }
    }

    /// Phase 0 must use a Pendulum movement pattern.
    #[test]
    fn phase_0_uses_pendulum_movement() {
        let phases = rumia_phases();
        assert!(
            matches!(phases[0].movement, BossMovement::Pendulum { .. }),
            "phase 0 must use Pendulum movement"
        );
    }

    /// Phase 1 must use a Circle movement pattern.
    #[test]
    fn phase_1_uses_circle_movement() {
        let phases = rumia_phases();
        assert!(
            matches!(phases[1].movement, BossMovement::Circle { .. }),
            "phase 1 must use Circle movement"
        );
    }

    /// Phase 2 must use a Pendulum movement pattern.
    #[test]
    fn phase_2_uses_pendulum_movement() {
        let phases = rumia_phases();
        assert!(
            matches!(phases[2].movement, BossMovement::Pendulum { .. }),
            "phase 2 must use Pendulum movement"
        );
    }

    /// Phase 0 Aimed pattern must have count ≥ 1 and speed > 0.
    #[test]
    fn phase_0_aimed_pattern_is_valid() {
        let phases = rumia_phases();
        match phases[0].pattern {
            BulletPattern::Aimed { count, speed, .. } => {
                assert!(count >= 1, "aimed count must be at least 1");
                assert!(speed > 0.0, "aimed speed must be positive");
            }
            _ => panic!("phase 0 must use an Aimed pattern"),
        }
    }

    /// Phase 1 Aimed pattern must use Rice bullets for the night-bird feel.
    #[test]
    fn phase_1_uses_rice_bullets() {
        let phases = rumia_phases();
        assert_eq!(
            phases[1].bullet_kind,
            EnemyBulletKind::Rice,
            "phase 1 (Night Bird) should use Rice bullets"
        );
    }

    /// Phase 2 Ring pattern must have count ≥ 8 for a proper ring burst.
    #[test]
    fn phase_2_ring_has_sufficient_bullets() {
        let phases = rumia_phases();
        match phases[2].pattern {
            BulletPattern::Ring { count, speed } => {
                assert!(count >= 8, "ring count must be at least 8, got {count}");
                assert!(speed > 0.0, "ring speed must be positive");
            }
            _ => panic!("phase 2 must use a Ring pattern"),
        }
    }

    /// Spell card bonuses must be in ascending order (harder phases pay more).
    #[test]
    fn spell_card_bonuses_are_in_order() {
        let phases = rumia_phases();
        let spell_phases: Vec<_> = phases.iter().filter(|p| p.is_spell_card).collect();
        for window in spell_phases.windows(2) {
            assert!(
                window[1].spell_card_bonus >= window[0].spell_card_bonus,
                "later spell cards must award at least as many points as earlier ones"
            );
        }
    }

    // ---- difficulty scaling tests ------------------------------------------

    /// Scaled phases must preserve the same count as the Normal baseline.
    #[test]
    fn scaled_phases_preserve_count() {
        for d in [
            Difficulty::Easy,
            Difficulty::Normal,
            Difficulty::Hard,
            Difficulty::Lunatic,
        ] {
            let phases = rumia_phases_for_difficulty(d);
            assert_eq!(phases.len(), 3, "{} must still have 3 phases", d.label());
        }
    }

    /// Hard phases must have higher HP than Normal.
    #[test]
    fn hard_has_more_hp_than_normal() {
        let normal = rumia_phases_for_difficulty(Difficulty::Normal);
        let hard = rumia_phases_for_difficulty(Difficulty::Hard);
        for i in 0..normal.len() {
            assert!(
                hard[i].hp > normal[i].hp,
                "Hard phase {i} HP ({}) must exceed Normal ({})",
                hard[i].hp,
                normal[i].hp
            );
        }
    }

    /// Easy phases must have less HP than Normal.
    #[test]
    fn easy_has_less_hp_than_normal() {
        let normal = rumia_phases_for_difficulty(Difficulty::Normal);
        let easy = rumia_phases_for_difficulty(Difficulty::Easy);
        for i in 0..normal.len() {
            assert!(
                easy[i].hp < normal[i].hp,
                "Easy phase {i} HP ({}) must be below Normal ({})",
                easy[i].hp,
                normal[i].hp
            );
        }
    }

    /// All scaled difficulties must keep hp == hp_max.
    #[test]
    fn scaled_hp_equals_hp_max() {
        for d in [
            Difficulty::Easy,
            Difficulty::Normal,
            Difficulty::Hard,
            Difficulty::Lunatic,
        ] {
            for (i, p) in rumia_phases_for_difficulty(d).iter().enumerate() {
                assert_eq!(
                    p.hp,
                    p.hp_max,
                    "{} phase {i}: hp ({}) != hp_max ({})",
                    d.label(),
                    p.hp,
                    p.hp_max
                );
            }
        }
    }

    /// Lunatic boss phases must have shorter time limits than Normal.
    #[test]
    fn lunatic_has_shorter_time_limits() {
        let normal = rumia_phases_for_difficulty(Difficulty::Normal);
        let lunatic = rumia_phases_for_difficulty(Difficulty::Lunatic);
        for i in 0..normal.len() {
            assert!(
                lunatic[i].time_limit_secs < normal[i].time_limit_secs,
                "Lunatic phase {i} time ({}) must be shorter than Normal ({})",
                lunatic[i].time_limit_secs,
                normal[i].time_limit_secs
            );
        }
    }
}
