# Phase 07: ザコ敵・ステージスクロール

## 目標

タイムライン駆動でザコ敵の波が出現し、撃破できる。Stage 1 風の演出が動く。

## 完了条件

- [ ] `Enemy` コンポーネントと複数の移動パターン
- [ ] `EnemySpawner` タイムライン駆動
- [ ] 画面外敵の自動カリング
- [ ] Stage 1 の最初の2分間のザコ波スクリプト

---

## タスク詳細

### 1. 敵コンポーネント

```rust
// app/core/src/components/enemy.rs

#[derive(Component)]
pub struct Enemy {
    pub hp: f32,
    pub hp_max: f32,
    pub score_value: u32,
    pub is_boss: bool,
}

#[derive(Component, Clone)]
pub enum EnemyMovement {
    /// 直線移動後停止
    LinearThenStop { velocity: Vec2, stop_after: f32 },
    /// 直線移動（停止なし）
    Linear { velocity: Vec2 },
    /// サインウェーブ
    SineWave { base_velocity: Vec2, amplitude: f32, frequency: f32 },
    /// スポーン位置から下へ直進
    FallDown { speed: f32 },
    /// プレイヤー追尾
    ChasePlayer { speed: f32 },
    /// ウェイポイント移動
    Waypoints { points: Vec<Vec2>, speed: f32, current: usize },
}

#[derive(Component, Clone, Copy)]
pub enum EnemyKind {
    Fairy,    // 妖精: 基本の雑魚
    Bat,      // コウモリ: 小さく速い
    TallFairy, // 大型妖精: HPが高い
}

impl EnemyKind {
    pub fn base_hp(&self) -> f32 {
        match self {
            Self::Fairy => 10.0,
            Self::Bat => 5.0,
            Self::TallFairy => 30.0,
        }
    }

    pub fn score_value(&self) -> u32 {
        match self {
            Self::Fairy => 100,
            Self::Bat => 50,
            Self::TallFairy => 300,
        }
    }

    pub fn collision_radius(&self) -> f32 {
        match self {
            Self::Fairy => 12.0,
            Self::Bat => 8.0,
            Self::TallFairy => 16.0,
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Self::Fairy => Color::srgb(0.5, 0.8, 1.0),
            Self::Bat => Color::srgb(0.3, 0.2, 0.5),
            Self::TallFairy => Color::srgb(0.8, 0.6, 1.0),
        }
    }
}
```

### 2. EnemySpawner リソース

```rust
// app/core/src/resources/spawner.rs

#[derive(Clone)]
pub struct SpawnEntry {
    pub time: f32,          // ステージ開始からの秒数
    pub kind: EnemyKind,
    pub position: Vec2,
    pub movement: EnemyMovement,
    pub emitter: Option<BulletEmitter>,
}

#[derive(Resource)]
pub struct EnemySpawner {
    pub script: Vec<SpawnEntry>,
    pub index: usize,
    pub elapsed: f32,
}

impl Default for EnemySpawner {
    fn default() -> Self {
        Self {
            script: Vec::new(),
            index: 0,
            elapsed: 0.0,
        }
    }
}
```

### 3. StageData リソース

```rust
// app/core/src/resources/stage.rs

#[derive(Resource)]
pub struct StageData {
    pub current_stage: u8,
    pub stage_elapsed: f32,
    pub boss_spawned: bool,
    pub boss_defeated: bool,
}

impl Default for StageData {
    fn default() -> Self {
        Self {
            current_stage: 1,
            stage_elapsed: 0.0,
            boss_spawned: false,
            boss_defeated: false,
        }
    }
}
```

### 4. スポーンシステム

```rust
// app/core/src/systems/enemy/spawn.rs

pub fn enemy_spawner_system(
    mut commands: Commands,
    mut spawner: ResMut<EnemySpawner>,
    time: Res<Time>,
) {
    spawner.elapsed += time.delta_secs();

    while spawner.index < spawner.script.len() {
        let entry = &spawner.script[spawner.index];
        if entry.time > spawner.elapsed { break; }

        spawn_enemy(&mut commands, entry);
        spawner.index += 1;
    }
}

fn spawn_enemy(commands: &mut Commands, entry: &SpawnEntry) {
    let kind = entry.kind;
    let mut entity = commands.spawn((
        Enemy {
            hp: kind.base_hp(),
            hp_max: kind.base_hp(),
            score_value: kind.score_value(),
            is_boss: false,
        },
        kind,
        entry.movement.clone(),
        Sprite {
            color: kind.color(),
            custom_size: Some(Vec2::splat(kind.collision_radius() * 2.0)),
            ..default()
        },
        Transform::from_translation(entry.position.extend(1.0)),
        DespawnOnExit(AppState::Playing),
    ));

    if let Some(emitter) = entry.emitter.clone() {
        entity.insert(emitter);
    }
}
```

### 5. 敵移動システム

```rust
// app/core/src/systems/enemy/movement.rs

pub fn enemy_movement_system(
    mut enemies: Query<(&mut Transform, &mut EnemyMovement), With<Enemy>>,
    player: Query<&Transform, (With<Player>, Without<Enemy>)>,
    time: Res<Time>,
) {
    let player_pos = player.single().ok()
        .map(|tf| tf.translation.truncate())
        .unwrap_or(Vec2::ZERO);

    for (mut tf, mut movement) in &mut enemies {
        let delta = time.delta_secs();
        match &mut *movement {
            EnemyMovement::Linear { velocity } => {
                tf.translation += velocity.extend(0.0) * delta;
            }
            EnemyMovement::LinearThenStop { velocity, stop_after } => {
                if *stop_after > 0.0 {
                    tf.translation += velocity.extend(0.0) * delta;
                    *stop_after -= delta;
                }
            }
            EnemyMovement::FallDown { speed } => {
                tf.translation.y -= speed * delta;
            }
            EnemyMovement::SineWave { base_velocity, amplitude, frequency } => {
                let t = time.elapsed_secs();
                let x_offset = amplitude * (t * *frequency * std::f32::consts::TAU).sin();
                tf.translation.x += (base_velocity.x + x_offset) * delta;
                tf.translation.y += base_velocity.y * delta;
            }
            EnemyMovement::ChasePlayer { speed } => {
                let pos = tf.translation.truncate();
                let dir = (player_pos - pos).normalize_or(Vec2::NEG_Y);
                tf.translation += (dir * *speed * delta).extend(0.0);
            }
            EnemyMovement::Waypoints { points, speed, current } => {
                if *current < points.len() {
                    let target = points[*current];
                    let pos = tf.translation.truncate();
                    let dir = (target - pos).normalize_or(Vec2::Y);
                    let move_dist = *speed * delta;
                    if pos.distance(target) < move_dist {
                        tf.translation = target.extend(tf.translation.z);
                        *current += 1;
                    } else {
                        tf.translation += (dir * move_dist).extend(0.0);
                    }
                }
            }
        }
    }
}
```

### 6. 敵カリングシステム

```rust
// app/core/src/systems/enemy/cull.rs
const CULL_MARGIN: f32 = 80.0;
const CULL_HALF_W: f32 = 192.0 + CULL_MARGIN;
const CULL_HALF_H: f32 = 224.0 + CULL_MARGIN;

pub fn enemy_cull_system(
    mut commands: Commands,
    enemies: Query<(Entity, &Transform), (With<Enemy>, Without<Boss>)>,
) {
    for (entity, tf) in &enemies {
        let pos = tf.translation.truncate();
        if pos.x.abs() > CULL_HALF_W || pos.y < -CULL_HALF_H || pos.y > CULL_HALF_H + 100.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}
```

### 7. Stage 1 スクリプト

```rust
// app/core/src/systems/boss/bosses/stage1_script.rs

pub fn stage1_script() -> Vec<SpawnEntry> {
    vec![
        // 開幕: 妖精が上から降ってくる
        SpawnEntry {
            time: 2.0,
            kind: EnemyKind::Fairy,
            position: Vec2::new(-60.0, 260.0),
            movement: EnemyMovement::FallDown { speed: 80.0 },
            emitter: Some(BulletEmitter {
                pattern: BulletPattern::Aimed { count: 3, spread_deg: 20.0, speed: 100.0 },
                bullet_kind: EnemyBulletKind::SmallRound,
                timer: Timer::from_seconds(2.0, TimerMode::Repeating),
                active: true,
            }),
        },
        SpawnEntry {
            time: 2.5,
            kind: EnemyKind::Fairy,
            position: Vec2::new(60.0, 260.0),
            movement: EnemyMovement::FallDown { speed: 80.0 },
            emitter: Some(BulletEmitter {
                pattern: BulletPattern::Aimed { count: 3, spread_deg: 20.0, speed: 100.0 },
                bullet_kind: EnemyBulletKind::SmallRound,
                timer: Timer::from_seconds(2.0, TimerMode::Repeating),
                active: true,
            }),
        },
        // Wave 2: 左右から
        SpawnEntry {
            time: 8.0,
            kind: EnemyKind::Fairy,
            position: Vec2::new(-200.0, 100.0),
            movement: EnemyMovement::Linear { velocity: Vec2::new(60.0, -30.0) },
            emitter: Some(BulletEmitter {
                pattern: BulletPattern::Ring { count: 6, speed: 100.0 },
                bullet_kind: EnemyBulletKind::SmallRound,
                timer: Timer::from_seconds(1.5, TimerMode::Repeating),
                active: true,
            }),
        },
        // ... 2分間分のウェーブ
    ]
}
```

### 8. ステージ管理システム

```rust
pub fn stage_control_system(
    mut stage_data: ResMut<StageData>,
    mut spawner: ResMut<EnemySpawner>,
    enemies: Query<(), (With<Enemy>, Without<Boss>)>,
    mut next_state: ResMut<NextState<AppState>>,
    time: Res<Time>,
) {
    stage_data.stage_elapsed += time.delta_secs();

    // 全ザコ撃破 & ボス未出現 → ボスをスポーン
    if !stage_data.boss_spawned
        && spawner.index >= spawner.script.len()
        && enemies.is_empty()
    {
        stage_data.boss_spawned = true;
        // Phase 8 でボスをスポーン
    }

    // ボス撃破 → ステージクリア
    if stage_data.boss_spawned && stage_data.boss_defeated {
        next_state.set(AppState::StageClear);
    }
}
```

---

## 参照

- `docs/03_danmaku_systems.md` § 弾幕パターン
- `docs/references/vampire-survivors/app/core/src/systems/enemy_spawn.rs`
