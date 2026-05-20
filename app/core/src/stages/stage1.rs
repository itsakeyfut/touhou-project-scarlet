use bevy::prelude::*;

use crate::{
    components::{
        bullet::{BulletEmitter, BulletPattern, EnemyBulletKind},
        enemy::{EnemyKind, EnemyMovement},
    },
    constants::{PLAY_AREA_HALF_H, PLAY_AREA_HALF_W},
    resources::SpawnEntry,
};

// ---------------------------------------------------------------------------
// Spawn-position constants
// ---------------------------------------------------------------------------

/// Extra vertical margin above the top edge before spawning (px).
const TOP_MARGIN: f32 = 66.0;

/// Extra horizontal margin beyond each side edge before spawning (px).
const SIDE_MARGIN: f32 = 38.0;

/// Spawn Y just above the visible play area.
const TOP: f32 = PLAY_AREA_HALF_H + TOP_MARGIN;

/// Spawn X beyond the left edge of the play area.
const LEFT: f32 = -(PLAY_AREA_HALF_W + SIDE_MARGIN);

/// Spawn X beyond the right edge of the play area.
const RIGHT: f32 = PLAY_AREA_HALF_W + SIDE_MARGIN;

// ---------------------------------------------------------------------------
// Emitter helpers
// ---------------------------------------------------------------------------

/// Standard aimed-spread emitter for a normal fairy (3-way, 20°, 110 px/s).
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

/// Ring-burst emitter used by stop-and-shoot enemies (TallFairy etc.).
fn ring_emitter(count: u8, speed: f32, interval_secs: f32) -> BulletEmitter {
    BulletEmitter {
        pattern: BulletPattern::Ring { count, speed },
        bullet_kind: EnemyBulletKind::SmallRound,
        timer: Timer::from_seconds(interval_secs, TimerMode::Repeating),
        active: true,
    }
}

// ---------------------------------------------------------------------------
// SpawnEntry helpers
// ---------------------------------------------------------------------------

/// Base factory: creates a [`SpawnEntry`] from its raw components.
fn entry(
    time: f32,
    kind: EnemyKind,
    pos: Vec2,
    movement: EnemyMovement,
    emitter: Option<BulletEmitter>,
) -> SpawnEntry {
    SpawnEntry {
        time,
        kind,
        position: pos,
        movement,
        emitter,
    }
}

/// A fairy that falls straight down from `(x, TOP)`.
fn fall_fairy(time: f32, x: f32) -> SpawnEntry {
    entry(
        time,
        EnemyKind::Fairy,
        Vec2::new(x, TOP),
        EnemyMovement::FallDown { speed: 80.0 },
        Some(fairy_emitter()),
    )
}

/// A fairy entering from the left or right side at `(from_x, y)` with
/// velocity `(vx, vy)`.
fn side_fairy(time: f32, from_x: f32, y: f32, vx: f32, vy: f32) -> SpawnEntry {
    entry(
        time,
        EnemyKind::Fairy,
        Vec2::new(from_x, y),
        EnemyMovement::Linear {
            velocity: Vec2::new(vx, vy),
        },
        Some(fairy_emitter()),
    )
}

/// A bat entering from `(from_x, y)` horizontally with speed `vx`.
/// Slight downward drift of -30 px/s is applied to all bats.
fn bat_horizontal(time: f32, from_x: f32, y: f32, vx: f32) -> SpawnEntry {
    entry(
        time,
        EnemyKind::Bat,
        Vec2::new(from_x, y),
        EnemyMovement::Linear {
            velocity: Vec2::new(vx, -30.0),
        },
        None,
    )
}

/// A bat falling straight down from `(x, TOP)`.
fn bat_fall(time: f32, x: f32) -> SpawnEntry {
    entry(
        time,
        EnemyKind::Bat,
        Vec2::new(x, TOP),
        EnemyMovement::FallDown { speed: 130.0 },
        None,
    )
}

/// A fairy that descends from `(x, TOP)`, stops after `stop_after` seconds,
/// then fires a ring burst (`count` bullets at `speed` px/s every `interval` s).
fn stop_ring_fairy(
    time: f32,
    x: f32,
    stop_after: f32,
    count: u8,
    speed: f32,
    interval: f32,
) -> SpawnEntry {
    entry(
        time,
        EnemyKind::Fairy,
        Vec2::new(x, TOP),
        EnemyMovement::LinearThenStop {
            velocity: Vec2::new(0.0, -110.0),
            stop_after,
        },
        Some(ring_emitter(count, speed, interval)),
    )
}

/// A fairy that moves in a sine wave downward from `(x, TOP)`.
///
/// `base_vx` adds a horizontal drift component (0.0 for straight-down waves).
fn sine_fairy(time: f32, x: f32, base_vx: f32) -> SpawnEntry {
    entry(
        time,
        EnemyKind::Fairy,
        Vec2::new(x, TOP),
        EnemyMovement::SineWave {
            base_velocity: Vec2::new(base_vx, -70.0),
            amplitude: 80.0,
            frequency: 0.5,
        },
        Some(fairy_emitter()),
    )
}

/// A fairy that moves in a faster sine wave downward from `(x, TOP)`.
///
/// Used in the second zigzag wave where fairies move noticeably quicker.
fn fast_sine_fairy(time: f32, x: f32, base_vx: f32) -> SpawnEntry {
    entry(
        time,
        EnemyKind::Fairy,
        Vec2::new(x, TOP),
        EnemyMovement::SineWave {
            base_velocity: Vec2::new(base_vx, -80.0),
            amplitude: 60.0,
            frequency: 0.8,
        },
        Some(fairy_emitter()),
    )
}

/// A fairy that chases the player from `(x, TOP)` at 60 px/s.
fn chase_fairy(time: f32, x: f32) -> SpawnEntry {
    entry(
        time,
        EnemyKind::Fairy,
        Vec2::new(x, TOP),
        EnemyMovement::ChasePlayer { speed: 60.0 },
        Some(fairy_emitter()),
    )
}

/// A TallFairy that descends from `(x, TOP)` with an initial velocity of
/// `(vx, -100 px/s)`, stops after `stop_after` seconds, then fires a ring
/// burst (`count` bullets at `speed` px/s every `interval` s).
fn tall_fairy_stop(
    time: f32,
    x: f32,
    vx: f32,
    stop_after: f32,
    count: u8,
    speed: f32,
    interval: f32,
) -> SpawnEntry {
    entry(
        time,
        EnemyKind::TallFairy,
        Vec2::new(x, TOP),
        EnemyMovement::LinearThenStop {
            velocity: Vec2::new(vx, -100.0),
            stop_after,
        },
        Some(ring_emitter(count, speed, interval)),
    )
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
    [
        wave1_opening(),
        wave2_flanks(),
        wave3_bat_rush_left(),
        wave4_bat_rush_right(),
        wave5_stop_ring_fairies(),
        wave6_zigzag(),
        wave7_tall_fairy_escort(),
        wave8_bat_curtain(),
        wave9_side_fairies(),
        wave10_fast_zigzag(),
        wave11_twin_tall_fairies(),
        wave12_homing(),
        wave13_density_surge(),
        wave14_final_wave(),
    ]
    .into_iter()
    .flatten()
    .collect()
}

// ----------------------------------------------------------------
// Wave 1: Opening — 3 fairies falling from the top (t = 3 – 5 s)
// ----------------------------------------------------------------
fn wave1_opening() -> Vec<SpawnEntry> {
    vec![
        fall_fairy(3.0, -60.0),
        fall_fairy(3.5, 0.0),
        fall_fairy(4.0, 60.0),
    ]
}

// ----------------------------------------------------------------
// Wave 2: Flanks — fairies entering from left and right (t = 7 – 9 s)
// ----------------------------------------------------------------
fn wave2_flanks() -> Vec<SpawnEntry> {
    vec![
        side_fairy(7.0, LEFT, 100.0, 90.0, -50.0),
        side_fairy(7.5, LEFT, 60.0, 90.0, -50.0),
        side_fairy(8.0, RIGHT, 100.0, -90.0, -50.0),
        side_fairy(8.5, RIGHT, 60.0, -90.0, -50.0),
    ]
}

// ----------------------------------------------------------------
// Wave 3: Bat rush from the left (t = 12 – 14 s)
// ----------------------------------------------------------------
fn wave3_bat_rush_left() -> Vec<SpawnEntry> {
    vec![
        bat_horizontal(12.0, LEFT, 120.0, 160.0),
        bat_horizontal(12.4, LEFT, 90.0, 160.0),
        bat_horizontal(12.8, LEFT, 60.0, 160.0),
        bat_horizontal(13.2, LEFT, 30.0, 160.0),
        bat_horizontal(13.6, LEFT, 0.0, 160.0),
    ]
}

// ----------------------------------------------------------------
// Wave 4: Bat rush from the right (t = 16 – 18 s)
// ----------------------------------------------------------------
fn wave4_bat_rush_right() -> Vec<SpawnEntry> {
    vec![
        bat_horizontal(16.0, RIGHT, 120.0, -160.0),
        bat_horizontal(16.4, RIGHT, 90.0, -160.0),
        bat_horizontal(16.8, RIGHT, 60.0, -160.0),
        bat_horizontal(17.2, RIGHT, 30.0, -160.0),
        bat_horizontal(17.6, RIGHT, 0.0, -160.0),
    ]
}

// ----------------------------------------------------------------
// Wave 5: Stop-and-shoot fairies (t = 22 – 24 s)
// ----------------------------------------------------------------
fn wave5_stop_ring_fairies() -> Vec<SpawnEntry> {
    vec![
        stop_ring_fairy(22.0, -120.0, 1.8, 6, 110.0, 2.0),
        stop_ring_fairy(22.5, 0.0, 1.8, 6, 110.0, 2.0),
        stop_ring_fairy(23.0, 120.0, 1.8, 6, 110.0, 2.0),
    ]
}

// ----------------------------------------------------------------
// Wave 6: Sine-wave fairies (t = 28 – 31 s)
// ----------------------------------------------------------------
fn wave6_zigzag() -> Vec<SpawnEntry> {
    vec![
        sine_fairy(28.0, -100.0, 0.0),
        sine_fairy(29.0, 0.0, 0.0),
        sine_fairy(30.0, 100.0, 0.0),
    ]
}

// ----------------------------------------------------------------
// Wave 7: First TallFairy with escort fairies (t = 35 – 38 s)
// ----------------------------------------------------------------
fn wave7_tall_fairy_escort() -> Vec<SpawnEntry> {
    vec![
        tall_fairy_stop(35.0, 0.0, 0.0, 2.2, 8, 120.0, 2.5),
        fall_fairy(36.0, -100.0),
        // Right escort — slightly faster fall speed, so use entry() directly.
        entry(
            36.5,
            EnemyKind::Fairy,
            Vec2::new(100.0, TOP),
            EnemyMovement::FallDown { speed: 90.0 },
            Some(fairy_emitter()),
        ),
    ]
}

// ----------------------------------------------------------------
// Wave 8: Bat curtain from above (t = 44 – 46 s)
// ----------------------------------------------------------------
fn wave8_bat_curtain() -> Vec<SpawnEntry> {
    vec![
        bat_fall(44.0, -150.0),
        bat_fall(44.3, -90.0),
        bat_fall(44.6, -30.0),
        bat_fall(44.9, 30.0),
        bat_fall(45.2, 90.0),
        bat_fall(45.5, 150.0),
    ]
}

// ----------------------------------------------------------------
// Wave 9: Fairies from the sides (t = 52 – 55 s)
// ----------------------------------------------------------------
fn wave9_side_fairies() -> Vec<SpawnEntry> {
    vec![
        side_fairy(52.0, LEFT, 130.0, 80.0, -60.0),
        side_fairy(52.5, LEFT, 80.0, 80.0, -60.0),
        side_fairy(53.0, RIGHT, 130.0, -80.0, -60.0),
        side_fairy(53.5, RIGHT, 80.0, -80.0, -60.0),
    ]
}

// ----------------------------------------------------------------
// Wave 10: Faster sine-wave fairies (t = 60 – 63 s)
// ----------------------------------------------------------------
fn wave10_fast_zigzag() -> Vec<SpawnEntry> {
    vec![
        fast_sine_fairy(60.0, -120.0, 30.0),
        fast_sine_fairy(60.5, -40.0, 0.0),
        fast_sine_fairy(61.0, 40.0, 0.0),
        fast_sine_fairy(61.5, 120.0, -30.0),
    ]
}

// ----------------------------------------------------------------
// Wave 11: Two TallFairies with bat escorts (t = 68 – 70 s)
// ----------------------------------------------------------------
fn wave11_twin_tall_fairies() -> Vec<SpawnEntry> {
    vec![
        tall_fairy_stop(68.0, -80.0, 30.0, 2.0, 8, 120.0, 2.5),
        tall_fairy_stop(68.5, 80.0, -30.0, 2.0, 8, 120.0, 2.5),
        bat_horizontal(69.0, LEFT, 80.0, 140.0),
        bat_horizontal(69.3, RIGHT, 80.0, -140.0),
    ]
}

// ----------------------------------------------------------------
// Wave 12: Homing fairies (t = 77 – 80 s)
// ----------------------------------------------------------------
fn wave12_homing() -> Vec<SpawnEntry> {
    vec![
        chase_fairy(77.0, -150.0),
        chase_fairy(78.0, 0.0),
        chase_fairy(79.0, 150.0),
    ]
}

// ----------------------------------------------------------------
// Wave 13: Density surge — fairies + bats (t = 85 – 89 s)
// ----------------------------------------------------------------
fn wave13_density_surge() -> Vec<SpawnEntry> {
    vec![
        fall_fairy(85.0, -160.0),
        fall_fairy(85.5, -80.0),
        fall_fairy(86.0, 80.0),
        fall_fairy(86.5, 160.0),
        bat_horizontal(87.0, LEFT, 140.0, 170.0),
        bat_horizontal(87.3, LEFT, 100.0, 170.0),
        bat_horizontal(87.6, RIGHT, 140.0, -170.0),
        bat_horizontal(87.9, RIGHT, 100.0, -170.0),
    ]
}

// ----------------------------------------------------------------
// Wave 14: Final wave — TallFairies + closing bats (t = 93 – 97 s)
// ----------------------------------------------------------------
fn wave14_final_wave() -> Vec<SpawnEntry> {
    vec![
        tall_fairy_stop(93.0, -140.0, 50.0, 2.0, 10, 130.0, 2.0),
        tall_fairy_stop(93.5, 0.0, 0.0, 2.0, 10, 130.0, 2.0),
        tall_fairy_stop(94.0, 140.0, -50.0, 2.0, 10, 130.0, 2.0),
        bat_fall(95.0, -180.0),
        bat_fall(95.3, -90.0),
        bat_fall(95.6, 90.0),
        bat_fall(95.9, 180.0),
    ]
}
// No entries after t = 100 s.
// Once all enemies are cleared, stage_control_system emits BossSpawnEvent.

// ---------------------------------------------------------------------------
// Stage loader system
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

    /// Last entry must be at or before the 100-second stage window.
    #[test]
    fn last_entry_stays_within_stage_window() {
        let script = stage1_script();
        let last_time = script.last().unwrap().time;
        assert!(
            last_time <= 100.0,
            "last spawn at t={last_time} exceeds the 100-second stage window"
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

    /// There must be at least one entry using a non-trivial movement pattern.
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

    /// Helper: fall_fairy must produce a FallDown fairy at the correct x.
    #[test]
    fn fall_fairy_helper_is_correct() {
        let e = fall_fairy(5.0, 42.0);
        assert_eq!(e.time, 5.0);
        assert_eq!(e.kind, EnemyKind::Fairy);
        assert_eq!(e.position.x, 42.0);
        assert_eq!(e.position.y, TOP);
        assert!(matches!(e.movement, EnemyMovement::FallDown { .. }));
        assert!(e.emitter.is_some());
    }

    /// Helper: bat_horizontal must produce a bat with correct velocity sign.
    #[test]
    fn bat_horizontal_helper_direction() {
        let left = bat_horizontal(1.0, LEFT, 50.0, 160.0);
        let right = bat_horizontal(1.0, RIGHT, 50.0, -160.0);
        match left.movement {
            EnemyMovement::Linear { velocity } => assert!(velocity.x > 0.0),
            _ => panic!("expected Linear movement"),
        }
        match right.movement {
            EnemyMovement::Linear { velocity } => assert!(velocity.x < 0.0),
            _ => panic!("expected Linear movement"),
        }
    }

    /// Helper: tall_fairy_stop must produce a TallFairy with a ring emitter.
    #[test]
    fn tall_fairy_stop_helper_has_ring_emitter() {
        let e = tall_fairy_stop(10.0, 0.0, 0.0, 2.0, 8, 120.0, 2.5);
        assert_eq!(e.kind, EnemyKind::TallFairy);
        let emitter = e.emitter.expect("TallFairy must have an emitter");
        assert!(matches!(emitter.pattern, BulletPattern::Ring { .. }));
    }
}
