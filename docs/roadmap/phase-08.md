# Phase 08: ボスシステム基本 + スペルカード背景シェーダー

## 目標

ルーミアの基本攻撃（ノーマルフェーズ）が機能する。HPバーがあり、倒せる。スペルカード発動時に `SpellCardBgMaterial` で動的背景が表示される。

## 完了条件

- [ ] `Boss` コンポーネント・フェーズ管理
- [ ] ボスHP・被弾・フェーズ遷移
- [ ] `BossMovement` (Pendulum, Circle) 動作
- [ ] スペルカードシステム（名前表示・ボーナス）
- [ ] `HitFlashMaterial` (`shaders/hit_flash.wgsl`) — ボス被弾フラッシュ
- [ ] `SpellCardBgMaterial` (`shaders/spell_card_bg.wgsl`) — スペルカード背景
- [ ] ルーミア用スウォールパターン (pattern_id=0) 適用
- [ ] ルーミア Phase 1〜2 動作確認

---

## タスク詳細

### 1. Bossコンポーネント

```rust
// app/core/src/components/enemy.rs に追加

#[derive(Component)]
pub struct Boss {
    pub boss_type: BossType,
    pub current_phase: usize,
    pub phases: Vec<BossPhaseData>,
    pub phase_timer: Timer,
    pub spell_card_active: bool,
}

#[derive(Clone)]
pub struct BossPhaseData {
    pub hp: f32,
    pub hp_max: f32,
    pub is_spell_card: bool,
    pub spell_card_name: Option<String>,
    pub time_limit_secs: f32,
    pub pattern: BulletPattern,
    pub movement: BossMovement,
    pub spell_card_bonus: u32,
}

#[derive(Clone)]
pub enum BossMovement {
    Static,
    Pendulum { amplitude: f32, frequency: f32, base_x: f32 },
    Circle { radius: f32, speed_deg: f32, center: Vec2 },
    Teleport { waypoints: Vec<Vec2>, wait_timer: Timer, current: usize },
}

#[derive(Clone, Copy, PartialEq)]
pub enum BossType {
    Rumia,
    Cirno,
    Meiling,
    Patchouli,
    Sakuya,
    Remilia,
    Flandre,
}
```

### 2. ボスフェーズシステム

```rust
// app/core/src/systems/boss/phase.rs

pub fn boss_phase_system(
    mut commands: Commands,
    mut bosses: Query<(Entity, &mut Boss)>,
    time: Res<Time>,
    mut phase_events: EventWriter<BossPhaseChangedEvent>,
    mut stage_data: ResMut<StageData>,
    mut game_data: ResMut<GameData>,
) {
    for (entity, mut boss) in &mut bosses {
        let time_up = boss.phase_timer.tick(time.delta()).just_finished();
        let current = &boss.phases[boss.current_phase];
        let hp_zero = current.hp <= 0.0;

        if !time_up && !hp_zero { continue; }

        // スペルカード撃破ボーナス
        if boss.spell_card_active && hp_zero && !time_up {
            game_data.score += current.spell_card_bonus as u64;
        }

        let next = boss.current_phase + 1;
        if next >= boss.phases.len() {
            // ボス撃破
            stage_data.boss_defeated = true;
            commands.entity(entity).despawn_recursive();
        } else {
            boss.current_phase = next;
            let next_phase = &boss.phases[next];
            boss.phase_timer = Timer::from_seconds(next_phase.time_limit_secs, TimerMode::Once);
            boss.spell_card_active = next_phase.is_spell_card;
            phase_events.write(BossPhaseChangedEvent { entity, phase: next });
        }
    }
}
```

### 3. ボス被弾

```rust
// app/core/src/systems/boss/phase.rs に追加

pub fn player_bullet_hit_boss(
    bullets: Query<(Entity, &Transform, &PlayerBullet)>,
    mut bosses: Query<(&Transform, &mut Boss)>,
    mut commands: Commands,
) {
    const BOSS_RADIUS: f32 = 20.0;
    const BULLET_RADIUS: f32 = 4.0;

    for (bullet_entity, bullet_tf, bullet) in &bullets {
        let bpos = bullet_tf.translation.truncate();
        for (boss_tf, mut boss) in &mut bosses {
            let epos = boss_tf.translation.truncate();
            if check_circle_collision(bpos, BULLET_RADIUS, epos, BOSS_RADIUS) {
                commands.entity(bullet_entity).despawn();
                boss.phases[boss.current_phase].hp -= bullet.damage;
                break;
            }
        }
    }
}
```

### 4. ボス移動システム

```rust
// app/core/src/systems/boss/movement.rs

pub fn boss_movement_system(
    mut bosses: Query<(&mut Transform, &Boss)>,
    player: Query<&Transform, (With<Player>, Without<Boss>)>,
    time: Res<Time>,
) {
    let player_pos = player.single().ok()
        .map(|tf| tf.translation.truncate())
        .unwrap_or(Vec2::ZERO);

    for (mut boss_tf, boss) in &mut bosses {
        let phase = &boss.phases[boss.current_phase];
        match &phase.movement {
            BossMovement::Static => {}
            BossMovement::Pendulum { amplitude, frequency, base_x } => {
                let x = base_x + amplitude * (time.elapsed_secs() * frequency * std::f32::consts::TAU).sin();
                boss_tf.translation.x = x;
            }
            BossMovement::Circle { radius, speed_deg, center } => {
                let angle = time.elapsed_secs() * speed_deg.to_radians();
                boss_tf.translation.x = center.x + angle.cos() * radius;
                boss_tf.translation.y = center.y + angle.sin() * radius;
            }
            BossMovement::Teleport { waypoints, wait_timer, current } => {
                // Teleport はフェーズ開始時に設定
            }
        }
    }
}
```

### 5. ボスエミッターシステム

ボスは `BulletEmitter` コンポーネントを使うが、フェーズ遷移時に差し替える:

```rust
pub fn update_boss_emitter_on_phase_change(
    mut commands: Commands,
    mut phase_events: EventReader<BossPhaseChangedEvent>,
    bosses: Query<&Boss>,
) {
    for event in phase_events.read() {
        let Ok(boss) = bosses.get(event.entity) else { continue };
        let phase = &boss.phases[event.phase];

        // 旧エミッターを削除して新パターンを設定
        commands.entity(event.entity).remove::<BulletEmitter>();
        commands.entity(event.entity).insert(BulletEmitter {
            pattern: phase.pattern.clone(),
            bullet_kind: EnemyBulletKind::MediumRound,
            timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            active: true,
        });
    }
}
```

### 6. ルーミア実装

```rust
// app/core/src/systems/boss/bosses/rumia.rs

pub fn spawn_rumia(
    commands: &mut Commands,
    difficulty: Difficulty,
) {
    let params = DifficultyParams::for_difficulty(difficulty);

    let phases = vec![
        // Phase 1: 通常攻撃 - 自機狙い3-way
        BossPhaseData {
            hp: 600.0 * params.enemy_hp_multiplier,
            hp_max: 600.0 * params.enemy_hp_multiplier,
            is_spell_card: false,
            spell_card_name: None,
            time_limit_secs: 30.0 * params.boss_time_multiplier,
            pattern: BulletPattern::Aimed {
                count: 3,
                spread_deg: 20.0,
                speed: 100.0 * params.bullet_speed_multiplier,
            },
            movement: BossMovement::Pendulum {
                amplitude: 80.0,
                frequency: 0.4,
                base_x: 0.0,
            },
            spell_card_bonus: 0,
        },
        // Phase 2: スペルカード「闇符「ディマーケイション」」
        BossPhaseData {
            hp: 800.0 * params.enemy_hp_multiplier,
            hp_max: 800.0 * params.enemy_hp_multiplier,
            is_spell_card: true,
            spell_card_name: Some("闇符「ディマーケイション」".to_string()),
            time_limit_secs: 40.0 * params.boss_time_multiplier,
            pattern: BulletPattern::Ring {
                count: (12.0 * params.bullet_count_multiplier) as u8,
                speed: 90.0 * params.bullet_speed_multiplier,
            },
            movement: BossMovement::Circle {
                radius: 60.0,
                speed_deg: 30.0,
                center: Vec2::new(0.0, 80.0),
            },
            spell_card_bonus: 1_000_000,
        },
    ];

    commands.spawn((
        Boss {
            boss_type: BossType::Rumia,
            current_phase: 0,
            phases: phases.clone(),
            phase_timer: Timer::from_seconds(phases[0].time_limit_secs, TimerMode::Once),
            spell_card_active: false,
        },
        Enemy {
            hp: phases[0].hp,
            hp_max: phases[0].hp_max,
            score_value: 10_000,
            is_boss: true,
        },
        BulletEmitter {
            pattern: phases[0].pattern.clone(),
            bullet_kind: EnemyBulletKind::SmallRound,
            timer: Timer::from_seconds(0.8, TimerMode::Repeating),
            active: true,
        },
        Sprite {
            color: Color::srgb(0.2, 0.1, 0.3),
            custom_size: Some(Vec2::splat(40.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 120.0, 1.0),
        DespawnOnExit(AppState::Playing),
    ));
}
```

### シェーダー: スペルカード背景

スペルカード発動時（`BossPhaseChangedEvent` で `is_spell_card=true`）にプレイエリアを覆う全画面Mesh2dを生成し、`SpellCardBgMaterial` を適用する:

```rust
pub fn on_spell_card_start(
    mut commands: Commands,
    mut phase_events: EventReader<BossPhaseChangedEvent>,
    bosses: Query<(&Boss, &BossType)>,
    mut spell_materials: ResMut<Assets<SpellCardBgMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for event in phase_events.read() {
        let Ok((boss, boss_type)) = bosses.get(event.entity) else { continue };
        let phase = &boss.phases[event.phase];
        if !phase.is_spell_card { continue; }

        let (pattern_id, primary, secondary) = boss_type.spell_card_colors();

        commands.spawn((
            SpellCardBackground,
            Mesh2d(meshes.add(Rectangle::new(384.0, 448.0))),
            MeshMaterial2d(spell_materials.add(SpellCardBgMaterial {
                time: 0.0,
                pattern_id,
                primary_color: primary,
                secondary_color: secondary,
                intensity: 0.0, // フェードイン
                _padding: Vec3::ZERO,
            })),
            Transform::from_xyz(0.0, 0.0, -0.5),
            DespawnOnExit(AppState::Playing),
        ));
    }
}
```

ボス種別ごとの色設定:

```rust
impl BossType {
    pub fn spell_card_colors(&self) -> (u32, LinearRgba, LinearRgba) {
        match self {
            Self::Rumia     => (0, LinearRgba::new(0.15, 0.05, 0.25, 0.7), LinearRgba::new(0.02, 0.01, 0.05, 0.9)),
            Self::Cirno     => (1, LinearRgba::new(0.5, 0.9, 1.0, 0.6),   LinearRgba::new(0.8, 0.95, 1.0, 0.4)),
            Self::Meiling   => (2, LinearRgba::new(1.0, 0.5, 0.2, 0.6),   LinearRgba::new(0.2, 0.8, 0.4, 0.5)),
            Self::Patchouli => (3, LinearRgba::new(0.5, 0.2, 0.8, 0.7),   LinearRgba::new(0.8, 0.7, 0.1, 0.5)),
            Self::Sakuya    => (4, LinearRgba::new(0.9, 0.9, 0.95, 0.5),  LinearRgba::new(0.6, 0.7, 0.8, 0.4)),
            Self::Remilia   => (5, LinearRgba::new(0.7, 0.05, 0.1, 0.7),  LinearRgba::new(0.1, 0.02, 0.02, 0.9)),
            Self::Flandre   => (6, LinearRgba::new(1.0, 0.3, 0.0, 0.6),   LinearRgba::new(0.5, 0.0, 0.5, 0.5)),
        }
    }
}
```

---

## 参照

- `docs/03_danmaku_systems.md` § 5 (ボスシステム)
- `docs/specification.md` § ルーミアのスペルカード一覧
- `docs/10_shaders_wgsl.md` § 7 (スペルカード背景), § 6 (被弾フラッシュ)
