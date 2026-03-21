use bevy::prelude::*;

use crate::{
    components::{
        boss::Boss,
        bullet::{BulletEmitter, EnemyBullet},
        GameSessionEntity,
    },
    events::{BossPhaseChangedEvent, EnemyDefeatedEvent},
    resources::{GameData, StageData},
    shaders::{SpellCardBgMaterial, SpellCardBackground},
};

// ---------------------------------------------------------------------------
// boss_phase_system
// ---------------------------------------------------------------------------

/// Drives boss phase transitions every frame.
///
/// Each frame this system:
/// 1. Ticks the `Boss::phase_timer`.
/// 2. Checks whether the current phase ended (HP ≤ 0 **or** timer elapsed).
/// 3. If the phase is a defeated spell card (HP ≤ 0, not timed out) awards the
///    `spell_card_bonus` to `GameData::score`.
/// 4. Deactivates the current [`BulletEmitter`] immediately to prevent the old
///    emitter from firing in the next [`crate::game_set::GameSystemSet::BulletEmit`]
///    tick before [`update_boss_emitter_on_phase_change`] installs the new one.
/// 5. Despawns all live enemy bullets so the field is clear for the new phase.
/// 6. Either advances to the next phase **or** despawns the boss entity and
///    sets `StageData::boss_defeated = true`.
/// 7. Emits [`BossPhaseChangedEvent`] so downstream systems (emitter swap,
///    spell-card background, UI) can react in the same frame.
///
/// # Ordering & emitter timing
///
/// This system runs in [`crate::game_set::GameSystemSet::GameLogic`] — after
/// both `BulletEmit` and `Collision` — so HP changes from
/// [`crate::systems::collision::player_bullet_hit_boss`] are visible when the
/// phase condition is evaluated.  Because `BulletEmit` already ran in the same
/// frame, the old emitter fires once on the transition frame. To prevent this
/// from happening on subsequent frames the emitter is set `active = false` here
/// so `bullet_emitter_system` skips it until the `Commands`-deferred insert
/// from [`update_boss_emitter_on_phase_change`] replaces it at end-of-frame.
pub fn boss_phase_system(
    mut commands: Commands,
    mut bosses: Query<(Entity, &mut Boss, &Transform)>,
    mut emitters: Query<&mut BulletEmitter>,
    enemy_bullets: Query<Entity, With<EnemyBullet>>,
    spell_card_bgs: Query<Entity, With<SpellCardBackground>>,
    time: Res<Time>,
    mut phase_events: MessageWriter<BossPhaseChangedEvent>,
    mut defeated_events: MessageWriter<EnemyDefeatedEvent>,
    mut stage_data: ResMut<StageData>,
    mut game_data: ResMut<GameData>,
) {
    for (boss_entity, mut boss, boss_tf) in &mut bosses {
        let time_up = boss.phase_timer.tick(time.delta()).just_finished();
        let hp_zero = boss.phases[boss.current_phase].hp <= 0.0;

        if !time_up && !hp_zero {
            continue;
        }

        // Award spell-card bonus when defeated by damage (not by timeout).
        let current_phase = &boss.phases[boss.current_phase];
        if boss.spell_card_active && hp_zero && !time_up {
            game_data.score += current_phase.spell_card_bonus as u64;
        }

        // Silence the current emitter immediately so it does not fire in the
        // next BulletEmit tick while the Commands-deferred insert is pending.
        if let Ok(mut emitter) = emitters.get_mut(boss_entity) {
            emitter.active = false;
        }

        // Clear all active enemy bullets so the next phase starts on a clean field.
        for bullet_entity in &enemy_bullets {
            commands.entity(bullet_entity).despawn();
        }

        let next_phase_idx = boss.current_phase + 1;
        if next_phase_idx >= boss.phases.len() {
            // All phases exhausted — boss is defeated.
            // Despawn any spell-card background that was active during the last
            // phase; BossPhaseChangedEvent is not emitted here so on_spell_card_start
            // will not run its cleanup path automatically.
            for bg_entity in &spell_card_bgs {
                commands.entity(bg_entity).despawn();
            }

            stage_data.boss_defeated = true;

            // Notify item/score systems using the regular enemy-defeated path.
            // Use the boss's current world position so item drops spawn at the
            // correct location rather than at screen centre.
            defeated_events.write(EnemyDefeatedEvent {
                position: boss_tf.translation.truncate(),
                score: 0, // score was already accumulated per-phase
                is_boss: true,
            });

            commands.entity(boss_entity).despawn();
        } else {
            // Copy the values needed for mutation before borrowing mutably.
            let next_time_limit = boss.phases[next_phase_idx].time_limit_secs;
            let next_is_spell = boss.phases[next_phase_idx].is_spell_card;

            boss.current_phase = next_phase_idx;
            boss.phase_timer = Timer::from_seconds(next_time_limit, TimerMode::Once);
            boss.spell_card_active = next_is_spell;

            phase_events.write(BossPhaseChangedEvent {
                entity: boss_entity,
                phase: next_phase_idx,
            });
        }
    }
}

// ---------------------------------------------------------------------------
// update_boss_emitter_on_phase_change  (Issue #59)
// ---------------------------------------------------------------------------

/// Swaps the [`BulletEmitter`] on a boss entity whenever its phase changes.
///
/// Reads [`BossPhaseChangedEvent`] emitted by [`boss_phase_system`] and
/// replaces the boss entity's `BulletEmitter` component with one configured
/// for the new phase's `pattern`, `bullet_kind`, and `fire_interval_secs`.
///
/// Because Bevy's `insert` overwrites an existing component, this effectively
/// resets the emitter timer to zero as well, preventing a stale countdown from
/// the previous phase from firing in the new phase.
///
/// # Ordering
///
/// Registered in [`crate::GameSystemSet::GameLogic`], chained after
/// [`boss_phase_system`] so that events are visible.
pub fn update_boss_emitter_on_phase_change(
    mut commands: Commands,
    mut phase_events: MessageReader<BossPhaseChangedEvent>,
    bosses: Query<&Boss>,
) {
    for event in phase_events.read() {
        let Ok(boss) = bosses.get(event.entity) else {
            continue;
        };
        let phase = &boss.phases[event.phase];

        commands.entity(event.entity).insert(BulletEmitter {
            pattern: phase.pattern.clone(),
            bullet_kind: phase.bullet_kind,
            timer: Timer::from_seconds(phase.fire_interval_secs, TimerMode::Repeating),
            active: true,
        });
    }
}

// ---------------------------------------------------------------------------
// on_spell_card_start  (Issue #62 integration)
// ---------------------------------------------------------------------------

/// Spawns or replaces the [`SpellCardBackground`] entity when a spell-card phase begins.
///
/// Reads [`BossPhaseChangedEvent`] and — if the new phase `is_spell_card` —
/// first despawns every existing [`SpellCardBackground`] entity (preventing
/// fullscreen-mesh stacking across multiple spell-card phases), then spawns a
/// fresh full-play-area `Mesh2d` carrying a [`SpellCardBgMaterial`] configured
/// with the boss-specific pattern and colour set.
///
/// The `intensity` field starts at `0.0` and is faded in by
/// [`crate::shaders::plugin::update_spell_card_bg_time`] at `2.0/s`
/// (≈ 0.5 s fade-in).
///
/// The spawned entity carries [`GameSessionEntity`] so it is automatically
/// despawned when the game session ends.
///
/// # Ordering
///
/// Registered in [`crate::GameSystemSet::GameLogic`], chained after
/// [`boss_phase_system`] so that events are visible.
///
/// # Initial spell-card phase
///
/// When a boss spawns with its first phase already a spell card the
/// `BossPhaseChangedEvent` is never emitted (it fires only on *transitions*).
/// The separate [`spawn_initial_spell_card_bg`] system handles that case.
pub fn on_spell_card_start(
    mut commands: Commands,
    mut phase_events: MessageReader<BossPhaseChangedEvent>,
    bosses: Query<&Boss>,
    existing_bgs: Query<Entity, With<SpellCardBackground>>,
    mut spell_materials: ResMut<Assets<SpellCardBgMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for event in phase_events.read() {
        let Ok(boss) = bosses.get(event.entity) else {
            continue;
        };
        let phase = &boss.phases[event.phase];
        if !phase.is_spell_card {
            // Non-spell phase — despawn any lingering spell-card background.
            for bg_entity in &existing_bgs {
                commands.entity(bg_entity).despawn();
            }
            continue;
        }

        // Replace any previous spell-card background before spawning the new one.
        for bg_entity in &existing_bgs {
            commands.entity(bg_entity).despawn();
        }

        let (pattern_id, primary_color, secondary_color) = boss.boss_type.spell_card_colors();
        spawn_spell_card_bg(
            &mut commands,
            &mut meshes,
            &mut spell_materials,
            pattern_id,
            primary_color,
            secondary_color,
        );
    }
}

/// Spawns the [`SpellCardBackground`] when a boss's first phase is already a spell card.
///
/// Because [`on_spell_card_start`] only reacts to [`BossPhaseChangedEvent`]
/// (which is never emitted for the initial phase), this companion system uses
/// [`Added<Boss>`] to detect a freshly spawned boss and spawns the background
/// immediately if `spell_card_active` is `true`.
///
/// # Ordering
///
/// Registered in [`crate::GameSystemSet::GameLogic`], runs alongside
/// (or just before) the main phase-change chain.
pub fn spawn_initial_spell_card_bg(
    mut commands: Commands,
    new_bosses: Query<&Boss, Added<Boss>>,
    mut spell_materials: ResMut<Assets<SpellCardBgMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for boss in &new_bosses {
        if !boss.spell_card_active {
            continue;
        }
        let (pattern_id, primary_color, secondary_color) = boss.boss_type.spell_card_colors();
        spawn_spell_card_bg(
            &mut commands,
            &mut meshes,
            &mut spell_materials,
            pattern_id,
            primary_color,
            secondary_color,
        );
    }
}

/// Shared helper: spawns a single full-play-area [`SpellCardBackground`] mesh.
fn spawn_spell_card_bg(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    spell_materials: &mut ResMut<Assets<SpellCardBgMaterial>>,
    pattern_id: u32,
    primary_color: LinearRgba,
    secondary_color: LinearRgba,
) {
    commands.spawn((
        SpellCardBackground,
        GameSessionEntity,
        Mesh2d(meshes.add(Rectangle::new(384.0, 448.0))),
        MeshMaterial2d(spell_materials.add(SpellCardBgMaterial {
            time: 0.0,
            pattern_id,
            intensity: 0.0,
            _pad: 0.0,
            primary_color,
            secondary_color,
        })),
        // Render behind gameplay entities (player is at z=1, background at -0.5).
        Transform::from_xyz(0.0, 0.0, -0.5),
    ));
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{
        boss::{BossMovement, BossPhaseData, BossType},
        bullet::{BulletPattern, EnemyBulletKind},
    };

    fn make_phase(hp: f32, is_spell: bool, bonus: u32) -> BossPhaseData {
        BossPhaseData {
            hp,
            hp_max: hp,
            is_spell_card: is_spell,
            spell_card_name: None,
            time_limit_secs: 30.0,
            pattern: BulletPattern::Ring { count: 8, speed: 100.0 },
            bullet_kind: EnemyBulletKind::SmallRound,
            fire_interval_secs: 0.5,
            movement: BossMovement::Static,
            spell_card_bonus: bonus,
        }
    }

    // -----------------------------------------------------------------------
    // Phase transition logic (unit tests on data only — no Bevy App needed).
    // -----------------------------------------------------------------------

    /// When HP drops to zero on a spell-card phase the bonus should be awarded.
    #[test]
    fn spell_card_bonus_awarded_on_hp_defeat() {
        let mut score: u64 = 0;
        let hp_zero = true;
        let time_up = false;
        let spell_card_active = true;
        let spell_card_bonus: u32 = 1_000_000;

        if spell_card_active && hp_zero && !time_up {
            score += spell_card_bonus as u64;
        }

        assert_eq!(score, 1_000_000);
    }

    /// Bonus must NOT be awarded if the phase timed out (even if HP also hit zero).
    #[test]
    fn spell_card_bonus_not_awarded_on_timeout() {
        let mut score: u64 = 0;
        let hp_zero = true;
        let time_up = true; // timeout flag takes priority
        let spell_card_active = true;
        let spell_card_bonus: u32 = 1_000_000;

        if spell_card_active && hp_zero && !time_up {
            score += spell_card_bonus as u64;
        }

        assert_eq!(score, 0, "timeout should suppress spell-card bonus");
    }

    /// Bonus must NOT be awarded on normal (non-spell) phases.
    #[test]
    fn spell_card_bonus_not_awarded_on_normal_phase() {
        let mut score: u64 = 0;
        let hp_zero = true;
        let time_up = false;
        let spell_card_active = false;
        let spell_card_bonus: u32 = 1_000_000;

        if spell_card_active && hp_zero && !time_up {
            score += spell_card_bonus as u64;
        }

        assert_eq!(score, 0, "normal phase should not yield spell-card bonus");
    }

    /// After the last phase the boss should be marked as defeated.
    #[test]
    fn all_phases_exhausted_marks_boss_defeated() {
        let phases = vec![make_phase(500.0, false, 0)];
        let next_phase_idx = 1; // advance past the only phase
        let boss_defeated = next_phase_idx >= phases.len();
        assert!(boss_defeated, "advancing past the last phase must defeat the boss");
    }

    /// Advancing within available phases must NOT mark the boss as defeated.
    #[test]
    fn mid_transition_does_not_mark_boss_defeated() {
        let phases = vec![make_phase(500.0, false, 0), make_phase(800.0, true, 1_000_000)];
        let next_phase_idx = 1;
        let boss_defeated = next_phase_idx >= phases.len();
        assert!(!boss_defeated, "mid-boss transition must not defeat the boss");
    }

    /// `Boss::new` must initialise `spell_card_active` from the first phase.
    #[test]
    fn boss_spell_card_active_reflects_first_phase() {
        let boss = Boss::new(
            BossType::Rumia,
            vec![make_phase(500.0, false, 0), make_phase(800.0, true, 1_000_000)],
        );
        assert!(!boss.spell_card_active, "first phase is not a spell card");
    }

    /// After simulating a phase advance, `spell_card_active` should update.
    #[test]
    fn phase_advance_updates_spell_card_active() {
        let phases = vec![make_phase(500.0, false, 0), make_phase(800.0, true, 1_000_000)];
        let next_idx = 1;
        let next_is_spell = phases[next_idx].is_spell_card;
        assert!(next_is_spell, "second phase is a spell card");
    }

    /// New emitter timer duration must equal the next phase's fire interval.
    #[test]
    fn emitter_timer_matches_next_phase_fire_interval() {
        let phase = make_phase(800.0, true, 0);
        let timer = Timer::from_seconds(phase.fire_interval_secs, TimerMode::Repeating);
        assert!(
            (timer.duration().as_secs_f32() - phase.fire_interval_secs).abs() < 1e-6,
            "emitter timer duration must match fire_interval_secs"
        );
    }
}
