use bevy::prelude::*;

use crate::{
    components::{
        bullet::{BulletEmitter, BulletPattern, EnemyBulletKind},
        enemy::{EnemyKind, EnemyMovement},
    },
    resources::{EnemySpawner, SpawnEntry},
};

// ---------------------------------------------------------------------------
// Spawn helpers
// ---------------------------------------------------------------------------

/// Spawn Y just above the visible play area (top edge is +224).
const TOP: f32 = 290.0;

/// Spawn X beyond the left edge of the play area (left edge is -192).
const LEFT: f32 = -230.0;

/// Spawn X beyond the right edge of the play area (right edge is +192).
const RIGHT: f32 = 230.0;

/// Creates an [`BulletEmitter`] for a typical normal fairy (aimed spread, slow fire rate).
fn fairy_emitter() -> BulletEmitter {
    BulletEmitter {
        pattern: BulletPattern::Aimed {
            count: 3,
            spread_deg: 20.0,
            speed: 110.0,
        },
        bullet_kind: EnemyBulletKind::SmallRound,
        timer: Timer::from_seconds(2.0, TimerMode::Repeating),
        active: true,
    }
}

/// Creates an [`BulletEmitter`] for a ring-pattern enemy (TallFairy / stop-and-shoot).
fn ring_emitter(count: u8, speed: f32, interval_secs: f32) -> BulletEmitter {
    BulletEmitter {
        pattern: BulletPattern::Ring { count, speed },
        bullet_kind: EnemyBulletKind::SmallRound,
        timer: Timer::from_seconds(interval_secs, TimerMode::Repeating),
        active: true,
    }
}

// ---------------------------------------------------------------------------
// Stage 1 script
// ---------------------------------------------------------------------------

/// Returns the complete enemy spawn timeline for Stage 1.
///
/// Covers approximately 100 seconds of gameplay leading up to the Rumia boss
/// fight. The script is sorted ascending by [`SpawnEntry::time`] so that
/// [`crate::resources::EnemySpawner`] can advance its index linearly.
///
/// # Wave overview
///
/// | Time (s) | Wave | Enemies |
/// |---|---|---|
/// | 3 – 5 | Opening | 3 fairies falling from above |
/// | 7 – 9 | Flanks | 4 fairies entering from the sides |
/// | 12 – 14 | Bat rush L | 5 fast bats from the left |
/// | 16 – 18 | Bat rush R | 5 fast bats from the right |
/// | 22 – 24 | Ring fairies | 3 fairies: move down, stop, ring-fire |
/// | 28 – 31 | Zigzag | 3 sine-wave fairies |
/// | 35 – 38 | TallFairy 1 | 1 TallFairy + 2 escort fairies |
/// | 44 – 46 | Bat curtain | 6 bats falling from the top |
/// | 52 – 55 | Side fairies | 4 fairies entering from the sides |
/// | 60 – 63 | Zigzag wave 2 | 4 sine-wave fairies (faster) |
/// | 68 – 70 | Two TallFairies | 2 TallFairies + escort bats |
/// | 77 – 80 | Homing wave | 3 homing fairies |
/// | 85 – 89 | Density surge | 4 fairies + 4 bats |
/// | 93 – 97 | Final wave | 3 TallFairies + closing bats |
///
/// No entries exist after t = 100 s; once all enemies are cleared the
/// [`crate::systems::stage::stage_control_system`] emits
/// [`crate::events::BossSpawnEvent`].
pub fn stage1_script() -> Vec<SpawnEntry> {
    vec![
        // ----------------------------------------------------------------
        // Wave 1: Opening — 3 fairies falling from the top (t = 3 – 5 s)
        // ----------------------------------------------------------------
        SpawnEntry {
            time: 3.0,
            kind: EnemyKind::Fairy,
            position: Vec2::new(-60.0, TOP),
            movement: EnemyMovement::FallDown { speed: 80.0 },
            emitter: Some(fairy_emitter()),
        },
        SpawnEntry {
            time: 3.5,
            kind: EnemyKind::Fairy,
            position: Vec2::new(0.0, TOP),
            movement: EnemyMovement::FallDown { speed: 80.0 },
            emitter: Some(fairy_emitter()),
        },
        SpawnEntry {
            time: 4.0,
            kind: EnemyKind::Fairy,
            position: Vec2::new(60.0, TOP),
            movement: EnemyMovement::FallDown { speed: 80.0 },
            emitter: Some(fairy_emitter()),
        },
        // ----------------------------------------------------------------
        // Wave 2: Flanks — fairies entering from left and right (t = 7 – 9 s)
        // ----------------------------------------------------------------
        SpawnEntry {
            time: 7.0,
            kind: EnemyKind::Fairy,
            position: Vec2::new(LEFT, 100.0),
            movement: EnemyMovement::Linear {
                velocity: Vec2::new(90.0, -50.0),
            },
            emitter: Some(fairy_emitter()),
        },
        SpawnEntry {
            time: 7.5,
            kind: EnemyKind::Fairy,
            position: Vec2::new(LEFT, 60.0),
            movement: EnemyMovement::Linear {
                velocity: Vec2::new(90.0, -50.0),
            },
            emitter: Some(fairy_emitter()),
        },
        SpawnEntry {
            time: 8.0,
            kind: EnemyKind::Fairy,
            position: Vec2::new(RIGHT, 100.0),
            movement: EnemyMovement::Linear {
                velocity: Vec2::new(-90.0, -50.0),
            },
            emitter: Some(fairy_emitter()),
        },
        SpawnEntry {
            time: 8.5,
            kind: EnemyKind::Fairy,
            position: Vec2::new(RIGHT, 60.0),
            movement: EnemyMovement::Linear {
                velocity: Vec2::new(-90.0, -50.0),
            },
            emitter: Some(fairy_emitter()),
        },
        // ----------------------------------------------------------------
        // Wave 3: Bat rush from the left (t = 12 – 14 s)
        // ----------------------------------------------------------------
        SpawnEntry {
            time: 12.0,
            kind: EnemyKind::Bat,
            position: Vec2::new(LEFT, 120.0),
            movement: EnemyMovement::Linear {
                velocity: Vec2::new(160.0, -30.0),
            },
            emitter: None,
        },
        SpawnEntry {
            time: 12.4,
            kind: EnemyKind::Bat,
            position: Vec2::new(LEFT, 90.0),
            movement: EnemyMovement::Linear {
                velocity: Vec2::new(160.0, -30.0),
            },
            emitter: None,
        },
        SpawnEntry {
            time: 12.8,
            kind: EnemyKind::Bat,
            position: Vec2::new(LEFT, 60.0),
            movement: EnemyMovement::Linear {
                velocity: Vec2::new(160.0, -30.0),
            },
            emitter: None,
        },
        SpawnEntry {
            time: 13.2,
            kind: EnemyKind::Bat,
            position: Vec2::new(LEFT, 30.0),
            movement: EnemyMovement::Linear {
                velocity: Vec2::new(160.0, -30.0),
            },
            emitter: None,
        },
        SpawnEntry {
            time: 13.6,
            kind: EnemyKind::Bat,
            position: Vec2::new(LEFT, 0.0),
            movement: EnemyMovement::Linear {
                velocity: Vec2::new(160.0, -30.0),
            },
            emitter: None,
        },
        // ----------------------------------------------------------------
        // Wave 4: Bat rush from the right (t = 16 – 18 s)
        // ----------------------------------------------------------------
        SpawnEntry {
            time: 16.0,
            kind: EnemyKind::Bat,
            position: Vec2::new(RIGHT, 120.0),
            movement: EnemyMovement::Linear {
                velocity: Vec2::new(-160.0, -30.0),
            },
            emitter: None,
        },
        SpawnEntry {
            time: 16.4,
            kind: EnemyKind::Bat,
            position: Vec2::new(RIGHT, 90.0),
            movement: EnemyMovement::Linear {
                velocity: Vec2::new(-160.0, -30.0),
            },
            emitter: None,
        },
        SpawnEntry {
            time: 16.8,
            kind: EnemyKind::Bat,
            position: Vec2::new(RIGHT, 60.0),
            movement: EnemyMovement::Linear {
                velocity: Vec2::new(-160.0, -30.0),
            },
            emitter: None,
        },
        SpawnEntry {
            time: 17.2,
            kind: EnemyKind::Bat,
            position: Vec2::new(RIGHT, 30.0),
            movement: EnemyMovement::Linear {
                velocity: Vec2::new(-160.0, -30.0),
            },
            emitter: None,
        },
        SpawnEntry {
            time: 17.6,
            kind: EnemyKind::Bat,
            position: Vec2::new(RIGHT, 0.0),
            movement: EnemyMovement::Linear {
                velocity: Vec2::new(-160.0, -30.0),
            },
            emitter: None,
        },
        // ----------------------------------------------------------------
        // Wave 5: Stop-and-shoot fairies (t = 22 – 24 s)
        // ----------------------------------------------------------------
        SpawnEntry {
            time: 22.0,
            kind: EnemyKind::Fairy,
            position: Vec2::new(-120.0, TOP),
            movement: EnemyMovement::LinearThenStop {
                velocity: Vec2::new(0.0, -110.0),
                stop_after: 1.8,
            },
            emitter: Some(ring_emitter(6, 110.0, 2.0)),
        },
        SpawnEntry {
            time: 22.5,
            kind: EnemyKind::Fairy,
            position: Vec2::new(0.0, TOP),
            movement: EnemyMovement::LinearThenStop {
                velocity: Vec2::new(0.0, -110.0),
                stop_after: 1.8,
            },
            emitter: Some(ring_emitter(6, 110.0, 2.0)),
        },
        SpawnEntry {
            time: 23.0,
            kind: EnemyKind::Fairy,
            position: Vec2::new(120.0, TOP),
            movement: EnemyMovement::LinearThenStop {
                velocity: Vec2::new(0.0, -110.0),
                stop_after: 1.8,
            },
            emitter: Some(ring_emitter(6, 110.0, 2.0)),
        },
        // ----------------------------------------------------------------
        // Wave 6: Sine-wave fairies (t = 28 – 31 s)
        // ----------------------------------------------------------------
        SpawnEntry {
            time: 28.0,
            kind: EnemyKind::Fairy,
            position: Vec2::new(-100.0, TOP),
            movement: EnemyMovement::SineWave {
                base_velocity: Vec2::new(0.0, -70.0),
                amplitude: 80.0,
                frequency: 0.5,
            },
            emitter: Some(fairy_emitter()),
        },
        SpawnEntry {
            time: 29.0,
            kind: EnemyKind::Fairy,
            position: Vec2::new(0.0, TOP),
            movement: EnemyMovement::SineWave {
                base_velocity: Vec2::new(0.0, -70.0),
                amplitude: 80.0,
                frequency: 0.5,
            },
            emitter: Some(fairy_emitter()),
        },
        SpawnEntry {
            time: 30.0,
            kind: EnemyKind::Fairy,
            position: Vec2::new(100.0, TOP),
            movement: EnemyMovement::SineWave {
                base_velocity: Vec2::new(0.0, -70.0),
                amplitude: 80.0,
                frequency: 0.5,
            },
            emitter: Some(fairy_emitter()),
        },
        // ----------------------------------------------------------------
        // Wave 7: First TallFairy with escort (t = 35 – 38 s)
        // ----------------------------------------------------------------
        SpawnEntry {
            time: 35.0,
            kind: EnemyKind::TallFairy,
            position: Vec2::new(0.0, TOP),
            movement: EnemyMovement::LinearThenStop {
                velocity: Vec2::new(0.0, -90.0),
                stop_after: 2.2,
            },
            emitter: Some(ring_emitter(8, 120.0, 2.5)),
        },
        SpawnEntry {
            time: 36.0,
            kind: EnemyKind::Fairy,
            position: Vec2::new(-100.0, TOP),
            movement: EnemyMovement::FallDown { speed: 90.0 },
            emitter: Some(fairy_emitter()),
        },
        SpawnEntry {
            time: 36.5,
            kind: EnemyKind::Fairy,
            position: Vec2::new(100.0, TOP),
            movement: EnemyMovement::FallDown { speed: 90.0 },
            emitter: Some(fairy_emitter()),
        },
        // ----------------------------------------------------------------
        // Wave 8: Bat curtain from above (t = 44 – 46 s)
        // ----------------------------------------------------------------
        SpawnEntry {
            time: 44.0,
            kind: EnemyKind::Bat,
            position: Vec2::new(-150.0, TOP),
            movement: EnemyMovement::FallDown { speed: 130.0 },
            emitter: None,
        },
        SpawnEntry {
            time: 44.3,
            kind: EnemyKind::Bat,
            position: Vec2::new(-90.0, TOP),
            movement: EnemyMovement::FallDown { speed: 130.0 },
            emitter: None,
        },
        SpawnEntry {
            time: 44.6,
            kind: EnemyKind::Bat,
            position: Vec2::new(-30.0, TOP),
            movement: EnemyMovement::FallDown { speed: 130.0 },
            emitter: None,
        },
        SpawnEntry {
            time: 44.9,
            kind: EnemyKind::Bat,
            position: Vec2::new(30.0, TOP),
            movement: EnemyMovement::FallDown { speed: 130.0 },
            emitter: None,
        },
        SpawnEntry {
            time: 45.2,
            kind: EnemyKind::Bat,
            position: Vec2::new(90.0, TOP),
            movement: EnemyMovement::FallDown { speed: 130.0 },
            emitter: None,
        },
        SpawnEntry {
            time: 45.5,
            kind: EnemyKind::Bat,
            position: Vec2::new(150.0, TOP),
            movement: EnemyMovement::FallDown { speed: 130.0 },
            emitter: None,
        },
        // ----------------------------------------------------------------
        // Wave 9: Fairies from sides (t = 52 – 55 s)
        // ----------------------------------------------------------------
        SpawnEntry {
            time: 52.0,
            kind: EnemyKind::Fairy,
            position: Vec2::new(LEFT, 130.0),
            movement: EnemyMovement::Linear {
                velocity: Vec2::new(80.0, -60.0),
            },
            emitter: Some(fairy_emitter()),
        },
        SpawnEntry {
            time: 52.5,
            kind: EnemyKind::Fairy,
            position: Vec2::new(LEFT, 80.0),
            movement: EnemyMovement::Linear {
                velocity: Vec2::new(80.0, -60.0),
            },
            emitter: Some(fairy_emitter()),
        },
        SpawnEntry {
            time: 53.0,
            kind: EnemyKind::Fairy,
            position: Vec2::new(RIGHT, 130.0),
            movement: EnemyMovement::Linear {
                velocity: Vec2::new(-80.0, -60.0),
            },
            emitter: Some(fairy_emitter()),
        },
        SpawnEntry {
            time: 53.5,
            kind: EnemyKind::Fairy,
            position: Vec2::new(RIGHT, 80.0),
            movement: EnemyMovement::Linear {
                velocity: Vec2::new(-80.0, -60.0),
            },
            emitter: Some(fairy_emitter()),
        },
        // ----------------------------------------------------------------
        // Wave 10: Faster sine-wave fairies (t = 60 – 63 s)
        // ----------------------------------------------------------------
        SpawnEntry {
            time: 60.0,
            kind: EnemyKind::Fairy,
            position: Vec2::new(-120.0, TOP),
            movement: EnemyMovement::SineWave {
                base_velocity: Vec2::new(30.0, -80.0),
                amplitude: 60.0,
                frequency: 0.8,
            },
            emitter: Some(fairy_emitter()),
        },
        SpawnEntry {
            time: 60.5,
            kind: EnemyKind::Fairy,
            position: Vec2::new(-40.0, TOP),
            movement: EnemyMovement::SineWave {
                base_velocity: Vec2::new(0.0, -80.0),
                amplitude: 60.0,
                frequency: 0.8,
            },
            emitter: Some(fairy_emitter()),
        },
        SpawnEntry {
            time: 61.0,
            kind: EnemyKind::Fairy,
            position: Vec2::new(40.0, TOP),
            movement: EnemyMovement::SineWave {
                base_velocity: Vec2::new(0.0, -80.0),
                amplitude: 60.0,
                frequency: 0.8,
            },
            emitter: Some(fairy_emitter()),
        },
        SpawnEntry {
            time: 61.5,
            kind: EnemyKind::Fairy,
            position: Vec2::new(120.0, TOP),
            movement: EnemyMovement::SineWave {
                base_velocity: Vec2::new(-30.0, -80.0),
                amplitude: 60.0,
                frequency: 0.8,
            },
            emitter: Some(fairy_emitter()),
        },
        // ----------------------------------------------------------------
        // Wave 11: Two TallFairies with bat escorts (t = 68 – 70 s)
        // ----------------------------------------------------------------
        SpawnEntry {
            time: 68.0,
            kind: EnemyKind::TallFairy,
            position: Vec2::new(-80.0, TOP),
            movement: EnemyMovement::LinearThenStop {
                velocity: Vec2::new(30.0, -95.0),
                stop_after: 2.0,
            },
            emitter: Some(ring_emitter(8, 120.0, 2.5)),
        },
        SpawnEntry {
            time: 68.5,
            kind: EnemyKind::TallFairy,
            position: Vec2::new(80.0, TOP),
            movement: EnemyMovement::LinearThenStop {
                velocity: Vec2::new(-30.0, -95.0),
                stop_after: 2.0,
            },
            emitter: Some(ring_emitter(8, 120.0, 2.5)),
        },
        SpawnEntry {
            time: 69.0,
            kind: EnemyKind::Bat,
            position: Vec2::new(LEFT, 80.0),
            movement: EnemyMovement::Linear {
                velocity: Vec2::new(140.0, -20.0),
            },
            emitter: None,
        },
        SpawnEntry {
            time: 69.3,
            kind: EnemyKind::Bat,
            position: Vec2::new(RIGHT, 80.0),
            movement: EnemyMovement::Linear {
                velocity: Vec2::new(-140.0, -20.0),
            },
            emitter: None,
        },
        // ----------------------------------------------------------------
        // Wave 12: Homing fairies (t = 77 – 80 s)
        // ----------------------------------------------------------------
        SpawnEntry {
            time: 77.0,
            kind: EnemyKind::Fairy,
            position: Vec2::new(-150.0, TOP),
            movement: EnemyMovement::ChasePlayer { speed: 60.0 },
            emitter: Some(fairy_emitter()),
        },
        SpawnEntry {
            time: 78.0,
            kind: EnemyKind::Fairy,
            position: Vec2::new(0.0, TOP),
            movement: EnemyMovement::ChasePlayer { speed: 60.0 },
            emitter: Some(fairy_emitter()),
        },
        SpawnEntry {
            time: 79.0,
            kind: EnemyKind::Fairy,
            position: Vec2::new(150.0, TOP),
            movement: EnemyMovement::ChasePlayer { speed: 60.0 },
            emitter: Some(fairy_emitter()),
        },
        // ----------------------------------------------------------------
        // Wave 13: Density surge (t = 85 – 89 s)
        // ----------------------------------------------------------------
        SpawnEntry {
            time: 85.0,
            kind: EnemyKind::Fairy,
            position: Vec2::new(-160.0, TOP),
            movement: EnemyMovement::FallDown { speed: 85.0 },
            emitter: Some(fairy_emitter()),
        },
        SpawnEntry {
            time: 85.5,
            kind: EnemyKind::Fairy,
            position: Vec2::new(-80.0, TOP),
            movement: EnemyMovement::FallDown { speed: 85.0 },
            emitter: Some(fairy_emitter()),
        },
        SpawnEntry {
            time: 86.0,
            kind: EnemyKind::Fairy,
            position: Vec2::new(80.0, TOP),
            movement: EnemyMovement::FallDown { speed: 85.0 },
            emitter: Some(fairy_emitter()),
        },
        SpawnEntry {
            time: 86.5,
            kind: EnemyKind::Fairy,
            position: Vec2::new(160.0, TOP),
            movement: EnemyMovement::FallDown { speed: 85.0 },
            emitter: Some(fairy_emitter()),
        },
        SpawnEntry {
            time: 87.0,
            kind: EnemyKind::Bat,
            position: Vec2::new(LEFT, 140.0),
            movement: EnemyMovement::Linear {
                velocity: Vec2::new(170.0, -50.0),
            },
            emitter: None,
        },
        SpawnEntry {
            time: 87.3,
            kind: EnemyKind::Bat,
            position: Vec2::new(LEFT, 100.0),
            movement: EnemyMovement::Linear {
                velocity: Vec2::new(170.0, -50.0),
            },
            emitter: None,
        },
        SpawnEntry {
            time: 87.6,
            kind: EnemyKind::Bat,
            position: Vec2::new(RIGHT, 140.0),
            movement: EnemyMovement::Linear {
                velocity: Vec2::new(-170.0, -50.0),
            },
            emitter: None,
        },
        SpawnEntry {
            time: 87.9,
            kind: EnemyKind::Bat,
            position: Vec2::new(RIGHT, 100.0),
            movement: EnemyMovement::Linear {
                velocity: Vec2::new(-170.0, -50.0),
            },
            emitter: None,
        },
        // ----------------------------------------------------------------
        // Wave 14: Final wave — TallFairies + closing bats (t = 93 – 97 s)
        // ----------------------------------------------------------------
        SpawnEntry {
            time: 93.0,
            kind: EnemyKind::TallFairy,
            position: Vec2::new(-140.0, TOP),
            movement: EnemyMovement::LinearThenStop {
                velocity: Vec2::new(50.0, -100.0),
                stop_after: 2.0,
            },
            emitter: Some(ring_emitter(10, 130.0, 2.0)),
        },
        SpawnEntry {
            time: 93.5,
            kind: EnemyKind::TallFairy,
            position: Vec2::new(0.0, TOP),
            movement: EnemyMovement::LinearThenStop {
                velocity: Vec2::new(0.0, -100.0),
                stop_after: 2.0,
            },
            emitter: Some(ring_emitter(10, 130.0, 2.0)),
        },
        SpawnEntry {
            time: 94.0,
            kind: EnemyKind::TallFairy,
            position: Vec2::new(140.0, TOP),
            movement: EnemyMovement::LinearThenStop {
                velocity: Vec2::new(-50.0, -100.0),
                stop_after: 2.0,
            },
            emitter: Some(ring_emitter(10, 130.0, 2.0)),
        },
        SpawnEntry {
            time: 95.0,
            kind: EnemyKind::Bat,
            position: Vec2::new(-180.0, TOP),
            movement: EnemyMovement::FallDown { speed: 150.0 },
            emitter: None,
        },
        SpawnEntry {
            time: 95.3,
            kind: EnemyKind::Bat,
            position: Vec2::new(-90.0, TOP),
            movement: EnemyMovement::FallDown { speed: 150.0 },
            emitter: None,
        },
        SpawnEntry {
            time: 95.6,
            kind: EnemyKind::Bat,
            position: Vec2::new(90.0, TOP),
            movement: EnemyMovement::FallDown { speed: 150.0 },
            emitter: None,
        },
        SpawnEntry {
            time: 95.9,
            kind: EnemyKind::Bat,
            position: Vec2::new(180.0, TOP),
            movement: EnemyMovement::FallDown { speed: 150.0 },
            emitter: None,
        },
        // No entries after t = 100 s.
        // Once all enemies are cleared the stage_control_system emits BossSpawnEvent.
    ]
}

// ---------------------------------------------------------------------------
// Stage loader system
// ---------------------------------------------------------------------------

/// Loads the Stage 1 script into [`EnemySpawner`] when gameplay starts.
///
/// Registered as an [`OnEnter`](`bevy::prelude::OnEnter`)(`AppState::Playing`)
/// system. Resets the spawner's index so that restarting from the title screen
/// replays the full script.
pub fn load_stage1_system(mut spawner: ResMut<EnemySpawner>) {
    spawner.script = stage1_script();
    spawner.index = 0;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// The script must contain at least one wave.
    #[test]
    fn script_is_non_empty() {
        assert!(!stage1_script().is_empty());
    }

    /// All entries must be sorted ascending by time.
    #[test]
    fn script_is_sorted_by_time() {
        let script = stage1_script();
        for pair in script.windows(2) {
            assert!(
                pair[0].time <= pair[1].time,
                "entries out of order: t={} before t={}",
                pair[0].time,
                pair[1].time
            );
        }
    }

    /// First entry must not fire at time 0 (give the player a moment to settle).
    #[test]
    fn first_entry_has_grace_period() {
        let script = stage1_script();
        assert!(
            script[0].time >= 1.0,
            "first spawn should be at least 1 second in"
        );
    }

    /// Last entry must be before the 120-second mark.
    #[test]
    fn last_entry_before_two_minutes() {
        let script = stage1_script();
        let last_time = script.last().unwrap().time;
        assert!(
            last_time < 120.0,
            "last spawn at t={last_time} exceeds 2-minute cap"
        );
    }

    /// There must be at least one entry using each enemy kind.
    #[test]
    fn script_contains_all_enemy_kinds() {
        let script = stage1_script();
        assert!(script.iter().any(|e| e.kind == EnemyKind::Fairy));
        assert!(script.iter().any(|e| e.kind == EnemyKind::Bat));
        assert!(script.iter().any(|e| e.kind == EnemyKind::TallFairy));
    }

    /// There must be at least one entry using a non-trivial movement pattern
    /// (i.e. not only FallDown).
    #[test]
    fn script_uses_varied_movement_patterns() {
        let script = stage1_script();
        let has_sine = script
            .iter()
            .any(|e| matches!(e.movement, EnemyMovement::SineWave { .. }));
        let has_chase = script
            .iter()
            .any(|e| matches!(e.movement, EnemyMovement::ChasePlayer { .. }));
        assert!(has_sine, "script must include at least one SineWave entry");
        assert!(
            has_chase,
            "script must include at least one ChasePlayer entry"
        );
    }
}
