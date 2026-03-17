# 03 弾幕システム (Danmaku Systems)

## 概要

東方紅魔郷の核となる弾幕システムの設計ドキュメント。プレイヤー弾、敵弾、グレイズ、ボム、ボスフェーズ、スペルカード、アイテムシステムを含む。

---

## 1. プレイヤー弾システム

### 1.1 ShootTimer コンポーネント

```rust
#[derive(Component)]
pub struct ShootTimer {
    pub timer: Timer,
    pub power_level: u8, // 0-128
}
```

射撃レートはパワーレベルで変化する。

| パワーレベル | 射撃間隔 (ms) |
|---|---|
| 0-15   | 200 |
| 16-31  | 160 |
| 32-47  | 140 |
| 48-63  | 120 |
| 64-95  | 100 |
| 96-127 | 83  |
| 128    | 70  |

### 1.2 霊夢ショットタイプ

**タイプA「霊符」(Homing Amulet)**
- メイン: お札を5方向に発射（ホーミングなし、直線）
- サブ: 巫女霊（追尾ショット）、2発同時追尾
- パワーMAX: お札8枚 + 追尾霊4個

**タイプB「夢符」(Dream Sealing)**
- メイン: お札3方向に発射（広角）
- サブ: 陰陽玉（オービット弾）
- パワーMAX: お札5枚 + 陰陽玉2個

```rust
pub fn spawn_player_bullet_reimu_a(
    commands: &mut Commands,
    player_pos: Vec2,
    power: u8,
    assets: &ScarletAssets,
) {
    let count = bullet_count_from_power(power, 5, 8);
    let spread = spread_angle_from_power(power, 15.0, 5.0);

    for i in 0..count {
        let angle = -spread / 2.0 + spread / (count - 1) as f32 * i as f32;
        let dir = Vec2::from_angle(angle.to_radians()) * Vec2::new(0.0, 1.0);
        commands.spawn(PlayerBulletBundle {
            bullet: PlayerBullet { damage: 12.0 },
            velocity: BulletVelocity(dir * 600.0),
            ..default()
        });
    }
}
```

### 1.3 魔理沙ショットタイプ

**タイプA「魔符」(Magic Missile)**
- メイン: 魔法弾（直線、高速）3方向
- サブ: スターダスト（小さい拡散弾）
- パワーMAX: 魔法弾5発 + スターダスト多数

**タイプB「恋符」(Love Sign)**
- メイン: 集中した直線弾（単発高威力）
- サブ: マスタースパーク予備弾（溜め可能）
- パワーMAX: 高威力単発 + 広角サブ

### 1.4 弾消し（グレイズ/ボム時）

```rust
pub fn despawn_player_bullets_on_bomb(
    mut commands: Commands,
    bullets: Query<Entity, With<PlayerBullet>>,
    mut bomb_events: EventReader<BombUsedEvent>,
) {
    for _ in bomb_events.read() {
        for entity in &bullets {
            commands.entity(entity).despawn();
        }
    }
}
```

---

## 2. 弾幕パターンシステム

### 2.1 BulletEmitter コンポーネント

```rust
#[derive(Component)]
pub struct BulletEmitter {
    pub pattern: BulletPattern,
    pub timer: Timer,
    pub active: bool,
}

pub enum BulletPattern {
    // 基本パターン
    SingleShot { speed: f32, angle: f32 },
    Spread { count: u8, spread_deg: f32, speed: f32 },
    Ring { count: u8, speed: f32 },
    Stack { layers: u8, speed_min: f32, speed_max: f32 },

    // 回転パターン
    Spiral { arms: u8, rotation_speed_deg: f32, speed: f32 },
    RotatingRing { count: u8, speed: f32, rotation_speed_deg: f32 },

    // 追尾パターン
    Aimed { count: u8, spread_deg: f32, speed: f32 },
    AimedRing { count: u8, speed: f32 },

    // 複合パターン
    Burst { pattern: Box<BulletPattern>, burst_count: u8, burst_interval_secs: f32 },
    Wave { pattern: Box<BulletPattern>, amplitude: f32, frequency: f32 },

    // スペルカード専用
    SpellCard(SpellCardPattern),
}
```

### 2.2 基本パターン実装

**Ring (円形発射)**
```rust
pub fn emit_ring(
    commands: &mut Commands,
    origin: Vec2,
    count: u8,
    speed: f32,
    bullet_kind: EnemyBulletKind,
) {
    let angle_step = 360.0 / count as f32;
    for i in 0..count {
        let angle = angle_step * i as f32;
        let dir = Vec2::from_angle(angle.to_radians());
        commands.spawn(EnemyBulletBundle {
            bullet: EnemyBullet { damage: 1 },
            velocity: BulletVelocity(dir * speed),
            kind: bullet_kind,
            ..default()
        });
    }
}
```

**Spiral (螺旋)**
```rust
pub struct SpiralEmitter {
    pub arms: u8,
    pub current_angle: f32,
    pub rotation_speed: f32, // deg/sec
    pub bullet_speed: f32,
    pub timer: Timer,
}

impl SpiralEmitter {
    pub fn emit(&mut self, commands: &mut Commands, origin: Vec2, delta: f32) {
        self.current_angle += self.rotation_speed * delta;
        let arm_gap = 360.0 / self.arms as f32;
        for arm in 0..self.arms {
            let angle = self.current_angle + arm_gap * arm as f32;
            let dir = Vec2::from_angle(angle.to_radians());
            commands.spawn(EnemyBulletBundle {
                velocity: BulletVelocity(dir * self.bullet_speed),
                ..default()
            });
        }
    }
}
```

**Aimed (自機狙い)**
```rust
pub fn aimed_direction(origin: Vec2, player_pos: Vec2) -> Vec2 {
    (player_pos - origin).normalize_or(Vec2::NEG_Y)
}

pub fn emit_aimed_spread(
    commands: &mut Commands,
    origin: Vec2,
    player_pos: Vec2,
    count: u8,
    spread_deg: f32,
    speed: f32,
) {
    let base_dir = aimed_direction(origin, player_pos);
    let base_angle = base_dir.y.atan2(base_dir.x);
    let half = spread_deg.to_radians() / 2.0;
    let step = if count > 1 { spread_deg.to_radians() / (count - 1) as f32 } else { 0.0 };

    for i in 0..count {
        let angle = base_angle - half + step * i as f32;
        let dir = Vec2::from_angle(angle);
        commands.spawn(EnemyBulletBundle {
            velocity: BulletVelocity(dir * speed),
            ..default()
        });
    }
}
```

### 2.3 弾種 (EnemyBulletKind)

```rust
#[derive(Component, Clone, Copy, PartialEq)]
pub enum EnemyBulletKind {
    // 小弾
    SmallRound,    // 小さい丸弾 (r=4px)
    SmallCard,     // 小さいカード弾 (r=3px)

    // 中弾
    MediumRound,   // 中丸弾 (r=7px)
    Rice,          // 米弾 (r=5px 楕円)
    Oval,          // 卵弾 (r=6px)

    // 大弾
    LargeRound,    // 大丸弾 (r=11px)
    Butterfly,     // 蝶弾 (r=10px)

    // 特殊弾
    Laser,         // レーザー (LineCollider)
    CurveLaser,    // 曲線レーザー (曲がるレーザー)
    Amulet,        // お札（霊夢のお札に見た目が似た敵弾）
    Star,          // 星弾 (r=8px)
    Knife,         // ナイフ弾 (r=4px, 速度速い)
    Bubble,        // 泡弾 (r=9px, 遅い)
}

impl EnemyBulletKind {
    pub fn collision_radius(&self) -> f32 {
        match self {
            Self::SmallCard => 3.0,
            Self::SmallRound => 4.0,
            Self::Rice | Self::Knife => 5.0,
            Self::MediumRound | Self::Oval => 7.0,
            Self::Star | Self::Amulet => 8.0,
            Self::Bubble | Self::Butterfly => 9.0,
            Self::LargeRound => 11.0,
            Self::Laser | Self::CurveLaser => 4.0, // 幅の半分
        }
    }
}
```

### 2.4 レーザーシステム

```rust
#[derive(Component)]
pub struct LaserBullet {
    pub width: f32,
    pub length: f32,
    pub angle: f32,
    pub active: bool,
    pub charge_timer: Timer,  // 発射前の予備動作
    pub active_timer: Timer,  // 発射持続時間
}

/// レーザーとプレイヤーの衝突チェック (線分-点 判定)
pub fn check_laser_collision(
    laser_origin: Vec2,
    laser_angle: f32,
    laser_length: f32,
    laser_width: f32,
    player_pos: Vec2,
    player_hitbox: f32,
) -> bool {
    let dir = Vec2::from_angle(laser_angle);
    let to_player = player_pos - laser_origin;
    let proj = to_player.dot(dir);
    if proj < 0.0 || proj > laser_length {
        return false;
    }
    let perp_dist = (to_player - dir * proj).length();
    perp_dist < (laser_width / 2.0 + player_hitbox)
}
```

---

## 3. グレイズシステム

### 3.1 グレイズ判定

```rust
const PLAYER_HITBOX_RADIUS: f32 = 2.5;
const GRAZE_RADIUS: f32 = 16.0;

/// グレイズ: 弾が「ヒットボックスの外側、グレイズ圏内」を通過したとき
pub fn graze_detection_system(
    mut commands: Commands,
    player: Query<(Entity, &Transform), With<Player>>,
    bullets: Query<(Entity, &Transform, &EnemyBulletKind), With<EnemyBullet>>,
    mut grazed: Local<HashSet<Entity>>,
    mut graze_events: EventWriter<GrazeEvent>,
    mut game_data: ResMut<GameData>,
) {
    let Ok((player_entity, player_tf)) = player.single() else { return };
    let player_pos = player_tf.translation.truncate();

    // 前フレームでグレイズした弾を削除（新規グレイズのみカウント）
    let mut current_frame_grazed = HashSet::new();

    for (bullet_entity, bullet_tf, bullet_kind) in &bullets {
        let bullet_pos = bullet_tf.translation.truncate();
        let dist = (bullet_pos - player_pos).length();
        let bullet_r = bullet_kind.collision_radius();

        let in_graze_zone = dist < (GRAZE_RADIUS + bullet_r);
        let not_hit = dist >= (PLAYER_HITBOX_RADIUS + bullet_r);

        if in_graze_zone && not_hit && !grazed.contains(&bullet_entity) {
            current_frame_grazed.insert(bullet_entity);
            game_data.graze_count += 1;
            game_data.score += 500; // グレイズ1回で+500点
            graze_events.write(GrazeEvent { bullet_entity });
        }
    }

    // グレイズセットを更新（存在しなくなった弾を除去）
    let bullet_entities: HashSet<Entity> = bullets.iter().map(|(e, _, _)| e).collect();
    grazed.retain(|e| bullet_entities.contains(e));
    grazed.extend(current_frame_grazed);
}
```

### 3.2 グレイズ演出

グレイズ発生時にGrazeEventを受け取り、SparkエフェクトをSpawnする。

```rust
pub fn graze_effect_system(
    mut commands: Commands,
    mut graze_events: EventReader<GrazeEvent>,
    bullets: Query<&Transform, With<EnemyBullet>>,
    player: Query<&Transform, With<Player>>,
    assets: Res<ScarletAssets>,
) {
    let Ok(player_tf) = player.single() else { return };
    for event in graze_events.read() {
        if let Ok(bullet_tf) = bullets.get(event.bullet_entity) {
            // プレイヤーとその周りにスパーク
            commands.spawn(GrazeSparkBundle {
                position: bullet_tf.translation.truncate(),
                ..default()
            });
        }
    }
}
```

---

## 4. ボムシステム

### 4.1 ボム発動

```rust
#[derive(Resource, Default)]
pub struct BombState {
    pub bombs_remaining: u8,       // 残りボム数
    pub active: bool,              // ボム発動中
    pub counter_bomb_window: f32,  // カウンターボム受付時間（秒）
    pub active_timer: Timer,       // ボム持続タイマー
    pub invincible_timer: Timer,   // 無敵時間タイマー
}

const BOMB_DURATION: f32 = 3.5;
const COUNTER_BOMB_WINDOW: f32 = 0.1; // 被弾後0.1秒以内ならカウンターボム
const BOMB_INVINCIBLE_DURATION: f32 = 5.0;
const MAX_BOMBS: u8 = 3;
```

**カウンターボム判定**

```rust
pub fn bomb_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut bomb_state: ResMut<BombState>,
    mut game_data: ResMut<GameData>,
    mut bomb_events: EventWriter<BombUsedEvent>,
    time: Res<Time>,
) {
    if !keys.just_pressed(KeyCode::KeyX) {
        return;
    }
    if bomb_state.active || game_data.bombs == 0 {
        return;
    }

    let is_counter = bomb_state.counter_bomb_window > 0.0;

    game_data.bombs -= 1;
    bomb_state.active = true;
    bomb_state.active_timer = Timer::from_seconds(BOMB_DURATION, TimerMode::Once);
    bomb_state.invincible_timer = Timer::from_seconds(BOMB_INVINCIBLE_DURATION, TimerMode::Once);
    bomb_state.counter_bomb_window = 0.0;

    bomb_events.write(BombUsedEvent { is_counter_bomb: is_counter });
}
```

**ボム効果**

```rust
pub fn bomb_effect_system(
    mut commands: Commands,
    bomb_state: Res<BombState>,
    enemy_bullets: Query<Entity, With<EnemyBullet>>,
    enemies: Query<(Entity, &Enemy)>,
    mut game_data: ResMut<GameData>,
    mut bomb_events: EventReader<BombUsedEvent>,
) {
    for event in bomb_events.read() {
        // 全敵弾消去
        for entity in &enemy_bullets {
            // 弾をアイテムに変換（点アイテムとして）
            commands.entity(entity).despawn();
        }

        // 雑魚敵にダメージ
        for (entity, enemy) in &enemies {
            if !enemy.is_boss {
                commands.entity(entity).despawn_recursive();
                // スコア加算
                game_data.score += enemy.score_value;
            }
        }

        // ボーナス点（消した弾数×10）
    }
}
```

### 4.2 霊夢ボム「夢符「封魔陣」」

- 自機中心に大きな結界（半径150px）を展開
- 結界内の敵弾を消去
- 結界が敵に触れるとダメージ

### 4.3 魔理沙ボム「恋符「マスタースパーク」」

- 画面上方向に向かって太いレーザー（幅80px）を発射
- 持続時間3.5秒間、範囲内の敵に継続ダメージ

```rust
pub enum BombVisual {
    Reimu {
        barrier_radius: f32,
        expand_speed: f32,
    },
    Marisa {
        spark_width: f32,
        spark_length: f32,
    },
}
```

---

## 5. ボスシステム

### 5.1 ボスフェーズ管理

```rust
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
    pub is_spell_card: bool,
    pub spell_card_name: Option<String>,
    pub time_limit_secs: f32,
    pub pattern: BulletPattern,
    pub movement: BossMovement,
    pub spell_card_bonus: u32,  // スペルカードボーナス
}
```

**フェーズ遷移**

```rust
pub fn boss_phase_system(
    mut commands: Commands,
    mut bosses: Query<(Entity, &mut Boss, &mut BossPhaseData)>,
    time: Res<Time>,
    mut phase_events: EventWriter<BossPhaseChangedEvent>,
    mut game_data: ResMut<GameData>,
) {
    for (entity, mut boss, mut phase_data) in &mut bosses {
        // タイムアップまたはHP0でフェーズ終了
        let time_up = boss.phase_timer.tick(time.delta()).just_finished();
        let hp_zero = phase_data.hp <= 0.0;

        if time_up || hp_zero {
            let spell_bonus = if boss.spell_card_active && hp_zero && !time_up {
                phase_data.spell_card_bonus
            } else {
                0
            };

            game_data.score += spell_bonus as u64;

            let next_phase = boss.current_phase + 1;
            if next_phase >= boss.phases.len() {
                // ボス撃破
                commands.entity(entity).despawn_recursive();
            } else {
                boss.current_phase = next_phase;
                let next = &boss.phases[next_phase];
                boss.phase_timer = Timer::from_seconds(next.time_limit_secs, TimerMode::Once);
                boss.spell_card_active = next.is_spell_card;
                phase_events.write(BossPhaseChangedEvent { entity, phase: next_phase });
            }
        }
    }
}
```

### 5.2 スペルカードシステム

スペルカードは名前付きの特殊攻撃フェーズ。ボーナス点あり。

```rust
pub struct SpellCardBonus {
    pub initial_bonus: u32,      // 初期ボーナス値
    pub time_deduction: u32,     // 1秒ごとに減算
    pub min_bonus: u32,          // 最低ボーナス値（0になる前）
}

/// スペルカードボーナス計算
pub fn calc_spell_card_bonus(initial: u32, elapsed: f32, deduction_per_sec: u32, min: u32) -> u32 {
    let deducted = (elapsed as u32) * deduction_per_sec;
    initial.saturating_sub(deducted).max(min)
}
```

スペルカード演出（背景変化、テキスト表示）：

```rust
#[derive(Component)]
pub struct SpellCardBackground {
    pub color: Color,
    pub pattern: BackgroundPattern,
}

pub enum BackgroundPattern {
    Solid(Color),
    Stars { density: f32, color: Color },
    Swirl { speed: f32, color: Color },
}
```

### 5.3 ボス移動パターン

```rust
pub enum BossMovement {
    Static,
    Pendulum { amplitude: f32, frequency: f32 },
    Circle { radius: f32, speed_deg: f32 },
    Teleport { waypoints: Vec<Vec2>, wait_secs: f32 },
    Chase { speed: f32, distance: f32 },
}

pub fn boss_movement_system(
    mut bosses: Query<(&mut Transform, &Boss)>,
    player: Query<&Transform, (With<Player>, Without<Boss>)>,
    time: Res<Time>,
) {
    let Ok(player_tf) = player.single() else { return };

    for (mut boss_tf, boss) in &mut bosses {
        let phase = &boss.phases[boss.current_phase];
        match &phase.movement {
            BossMovement::Pendulum { amplitude, frequency } => {
                let x = amplitude * (time.elapsed_secs() * frequency * TAU).sin();
                boss_tf.translation.x = x;
            }
            BossMovement::Circle { radius, speed_deg } => {
                let angle = time.elapsed_secs() * speed_deg.to_radians();
                boss_tf.translation.x = angle.cos() * radius;
                boss_tf.translation.y = angle.sin() * radius + 100.0; // Y中心オフセット
            }
            _ => {}
        }
    }
}
```

---

## 6. アイテムシステム

### 6.1 アイテム種類と取得判定

```rust
#[derive(Component, Clone, Copy)]
pub enum ItemKind {
    PowerSmall,   // 赤い小さい星 (+1 power)
    PowerLarge,   // 赤い大きい星 (+8 power)
    PointItem,    // 青い「P」 (点アイテム、高さ依存スコア)
    LifeFragment, // 1/5ライフ 5個でライフ+1
    BombFragment, // 1/5ボム 5個でボム+1
    Star,         // 金星（グレイズ加算）
    Cherry,       // 桜（スペルカードボーナス）
    FullPower,    // 赤い「P」（パワーをMAXに）
}
```

### 6.2 ポイントアイテム高さ依存スコア

ポイントアイテムの価値は、収集時のプレイヤーのY座標（高さ）に依存する。

```rust
const POI_BASE_VALUE: u32 = 10_000;      // 画面上端での基本値
const POI_MIN_VALUE: u32 = 100;           // 画面下端での最小値
const POI_HEIGHT_BOUNDARY: f32 = 128.0;  // この高さ以上なら最大値
const PLAY_AREA_HEIGHT: f32 = 448.0;

pub fn calc_point_item_value(player_y: f32) -> u32 {
    // Y座標が高い（画面上部）ほど高得点
    // プレイエリア上端=最大、POI_HEIGHT_BOUNDARY以上で最大
    let normalized_y = (player_y + PLAY_AREA_HEIGHT / 2.0) / PLAY_AREA_HEIGHT;
    let clamped = normalized_y.clamp(0.0, 1.0);

    if clamped >= (1.0 - POI_HEIGHT_BOUNDARY / PLAY_AREA_HEIGHT) {
        POI_BASE_VALUE
    } else {
        let t = clamped / (1.0 - POI_HEIGHT_BOUNDARY / PLAY_AREA_HEIGHT);
        (POI_MIN_VALUE as f32 + (POI_BASE_VALUE - POI_MIN_VALUE) as f32 * t) as u32
    }
}
```

### 6.3 アイテム吸引（引き寄せ）

```rust
#[derive(Component)]
pub struct ItemPhysics {
    pub velocity: Vec2,
    pub attracted: bool,     // 引き寄せ中
    pub fall_speed: f32,     // 重力落下速度
}

const ITEM_ATTRACT_RADIUS: f32 = 80.0; // 通常引き寄せ半径
const ITEM_ATTRACT_SPEED: f32 = 400.0;

pub fn item_attract_system(
    mut items: Query<(&mut ItemPhysics, &Transform)>,
    player: Query<(&Transform, &PlayerStats), With<Player>>,
    game_data: Res<GameData>,
    time: Res<Time>,
) {
    let Ok((player_tf, stats)) = player.single() else { return };
    let player_pos = player_tf.translation.truncate();
    let pickup_radius = stats.pickup_radius;

    // Cキー押し中 or アイテム吸引状態 (画面上端到達時など)
    for (mut physics, tf) in &mut items {
        let item_pos = tf.translation.truncate();
        let dist = (item_pos - player_pos).length();

        if dist < pickup_radius || physics.attracted {
            physics.attracted = true;
            let dir = (player_pos - item_pos).normalize_or(Vec2::Y);
            physics.velocity = dir * ITEM_ATTRACT_SPEED;
        }
    }
}
```

### 6.4 画面上端到達時のアイテム自動収集

スコアラインと呼ばれる高さ（Y = 192px = 画面上から約43%）以上にプレイヤーがいる場合、画面上のポイントアイテムはすべて最大値で自動収集される。

```rust
const SCORE_LINE_Y: f32 = 192.0; // プレイエリア座標での高さ

pub fn auto_collect_at_score_line(
    mut commands: Commands,
    items: Query<(Entity, &ItemKind, &Transform)>,
    player: Query<&Transform, With<Player>>,
    mut game_data: ResMut<GameData>,
) {
    let Ok(player_tf) = player.single() else { return };
    if player_tf.translation.y < SCORE_LINE_Y {
        return;
    }

    // プレイヤーがスコアライン以上にいる場合、ポイントアイテムを最大値で収集
    for (entity, kind, _) in &items {
        if matches!(kind, ItemKind::PointItem) {
            game_data.score += POI_BASE_VALUE as u64;
            commands.entity(entity).despawn();
        }
    }
}
```

---

## 7. スコアシステム詳細

### 7.1 スコア加算一覧

| イベント | 点数 |
|---|---|
| 雑魚敵撃破 | 100〜1,000 点（敵種別） |
| グレイズ1回 | 500 点 |
| ポイントアイテム（最大値） | 10,000 点 |
| ポイントアイテム（最小値） | 100 点 |
| スペルカードボーナス（最大） | 1,000,000〜5,000,000 点 |
| 残機ボーナス（ステージクリア） | 残機×50,000 点 |
| ボムボーナス（ステージクリア） | ボム×5,000 点 |

### 7.2 延命（エクステンド）

```rust
const EXTEND_THRESHOLDS: &[u64] = &[10_000_000, 20_000_000, 40_000_000, 60_000_000];

pub fn check_extend(
    mut game_data: ResMut<GameData>,
    mut extend_events: EventWriter<ExtendEvent>,
) {
    for &threshold in EXTEND_THRESHOLDS {
        if game_data.score >= threshold && !game_data.extend_awarded.contains(&threshold) {
            game_data.extend_awarded.insert(threshold);
            game_data.lives += 1;
            extend_events.write(ExtendEvent { threshold });
        }
    }
}
```

### 7.3 ライフフラグメント・ボムフラグメント

```rust
#[derive(Resource, Default)]
pub struct FragmentTracker {
    pub life_fragments: u8,  // 5個でライフ+1
    pub bomb_fragments: u8,  // 5個でボム+1
}

pub fn collect_fragment(
    fragment: ItemKind,
    tracker: &mut FragmentTracker,
    game_data: &mut GameData,
) {
    match fragment {
        ItemKind::LifeFragment => {
            tracker.life_fragments += 1;
            if tracker.life_fragments >= 5 {
                tracker.life_fragments = 0;
                game_data.lives += 1;
            }
        }
        ItemKind::BombFragment => {
            tracker.bomb_fragments += 1;
            if tracker.bomb_fragments >= 5 {
                tracker.bomb_fragments = 0;
                game_data.bombs = (game_data.bombs + 1).min(MAX_BOMBS);
            }
        }
        _ => {}
    }
}
```

---

## 8. 難易度スケーリング

### 8.1 難易度パラメータ

```rust
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
    Lunatic,
    Extra,
}

pub struct DifficultyParams {
    pub bullet_speed_multiplier: f32,
    pub bullet_count_multiplier: f32,
    pub enemy_hp_multiplier: f32,
    pub spawn_rate_multiplier: f32,
    pub boss_time_multiplier: f32,
}

impl DifficultyParams {
    pub fn for_difficulty(diff: Difficulty) -> Self {
        match diff {
            Difficulty::Easy    => Self { bullet_speed_multiplier: 0.6, bullet_count_multiplier: 0.7, enemy_hp_multiplier: 0.8,  spawn_rate_multiplier: 0.8, boss_time_multiplier: 1.2 },
            Difficulty::Normal  => Self { bullet_speed_multiplier: 1.0, bullet_count_multiplier: 1.0, enemy_hp_multiplier: 1.0,  spawn_rate_multiplier: 1.0, boss_time_multiplier: 1.0 },
            Difficulty::Hard    => Self { bullet_speed_multiplier: 1.3, bullet_count_multiplier: 1.4, enemy_hp_multiplier: 1.2,  spawn_rate_multiplier: 1.2, boss_time_multiplier: 0.85 },
            Difficulty::Lunatic => Self { bullet_speed_multiplier: 1.6, bullet_count_multiplier: 1.8, enemy_hp_multiplier: 1.5,  spawn_rate_multiplier: 1.4, boss_time_multiplier: 0.7 },
            Difficulty::Extra   => Self { bullet_speed_multiplier: 1.8, bullet_count_multiplier: 2.0, enemy_hp_multiplier: 2.0,  spawn_rate_multiplier: 1.5, boss_time_multiplier: 0.7 },
        }
    }
}
```

### 8.2 ランクシステム（内部難易度）

原作に倣ったランクシステム（省略可。将来実装）：

```rust
// 将来実装予定
pub struct RankSystem {
    pub rank: f32,          // 0.0 ~ 1.0
    pub rank_up_rate: f32,  // 時間経過で上昇
    pub rank_down_rate: f32, // 被弾で低下
}
```

---

## 9. 判定可視化（デバッグ）

開発中の判定確認用システム。`SCARLET_DEBUG_HITBOX=1` 環境変数またはデバッグビルドで有効化。

```rust
#[cfg(debug_assertions)]
pub fn debug_hitbox_system(
    mut gizmos: Gizmos,
    player: Query<&Transform, With<Player>>,
    bullets: Query<(&Transform, &EnemyBulletKind), With<EnemyBullet>>,
) {
    if let Ok(tf) = player.single() {
        let pos = tf.translation.truncate();
        // ヒットボックス（赤）
        gizmos.circle_2d(pos, PLAYER_HITBOX_RADIUS, Color::srgb(1.0, 0.0, 0.0));
        // グレイズ圏（黄）
        gizmos.circle_2d(pos, GRAZE_RADIUS, Color::srgb(1.0, 1.0, 0.0));
    }

    for (tf, kind) in &bullets {
        let pos = tf.translation.truncate();
        let r = kind.collision_radius();
        gizmos.circle_2d(pos, r, Color::srgba(0.0, 1.0, 0.0, 0.5));
    }
}
```

---

## 10. システム依存関係まとめ

```
BombInput → BombEffect → DespawnBullets
            ↓
PlayerInput → PlayerMovement → GrazeDetection
                               ↓
BulletEmitter → BulletMovement → CollisionDetection → HitPlayer / HitEnemy
                                                       ↓
                                               ItemSpawn → ItemAttract → ItemCollect
                                                                          ↓
                                                                    ScoreUpdate → ExtendCheck
```
