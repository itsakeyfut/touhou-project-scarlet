# Phase 06: アイテム・スコアシステム

## 目標

敵撃破でアイテムが落下し、取得するとスコアが加算される。エクステンドが機能する。

## 完了条件

- [ ] アイテムが落下・吸引・取得される
- [ ] ポイントアイテムの高さ依存スコア計算
- [ ] スコアライン自動収集
- [ ] エクステンド（スコア閾値で残機+1）
- [ ] ライフ/ボムフラグメント（5個で+1）
- [ ] `GameData` スコアが HUD テキストに反映

---

## タスク詳細

### 1. コンポーネント

```rust
// app/core/src/components/item.rs

#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub enum ItemKind {
    PowerSmall,   // +1 power
    PowerLarge,   // +8 power
    PointItem,    // 高さ依存スコア
    LifeFragment, // 5個でライフ+1
    BombFragment, // 5個でボム+1
    FullPower,    // パワーMAX
}

#[derive(Component)]
pub struct ItemPhysics {
    pub velocity: Vec2,
    pub attracted: bool,
    pub fall_speed: f32,
}

impl Default for ItemPhysics {
    fn default() -> Self {
        Self {
            velocity: Vec2::ZERO,
            attracted: false,
            fall_speed: 80.0,
        }
    }
}
```

### 2. GameData リソース（拡張）

```rust
// app/core/src/resources/game_data.rs

#[derive(Resource)]
pub struct GameData {
    pub score: u64,
    pub hi_score: u64,
    pub lives: u8,
    pub bombs: u8,
    pub power: u8,
    pub graze_count: u32,
    pub extend_awarded: std::collections::HashSet<u64>,
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            score: 0,
            hi_score: 0,
            lives: 2,    // 初期残機2（インデックス0ベース、画面上は3）
            bombs: 3,
            power: 0,
            graze_count: 0,
            extend_awarded: Default::default(),
        }
    }
}
```

```rust
// app/core/src/resources/fragment.rs
#[derive(Resource, Default)]
pub struct FragmentTracker {
    pub life_fragments: u8,
    pub bomb_fragments: u8,
}
```

### 3. アイテムスポーン

```rust
// app/core/src/systems/item.rs

pub fn spawn_item(
    commands: &mut Commands,
    pos: Vec2,
    kind: ItemKind,
) {
    let color = match kind {
        ItemKind::PowerSmall | ItemKind::PowerLarge | ItemKind::FullPower
            => Color::srgb(1.0, 0.2, 0.2),
        ItemKind::PointItem
            => Color::srgb(0.2, 0.5, 1.0),
        ItemKind::LifeFragment
            => Color::srgb(0.2, 1.0, 0.4),
        ItemKind::BombFragment
            => Color::srgb(1.0, 0.8, 0.2),
    };

    let size = match kind {
        ItemKind::PowerLarge | ItemKind::FullPower => Vec2::splat(12.0),
        _ => Vec2::splat(8.0),
    };

    commands.spawn((
        kind,
        ItemPhysics::default(),
        Sprite { color, custom_size: Some(size), ..default() },
        Transform::from_translation(pos.extend(1.2)),
        DespawnOnExit(AppState::Playing),
    ));
}

/// 敵撃破時のアイテムドロップ
pub fn on_enemy_defeated(
    mut commands: Commands,
    mut defeated_events: EventReader<EnemyDefeatedEvent>,
    enemies: Query<(&Transform, &Enemy)>,
    mut game_data: ResMut<GameData>,
) {
    for event in defeated_events.read() {
        game_data.score += event.score as u64;

        let Ok((tf, enemy)) = enemies.get(event.enemy_entity) else { continue };
        let pos = tf.translation.truncate();

        // ドロップアイテム (原作準拠: ポイントアイテム + パワーアイテム小)
        spawn_item(&mut commands, pos, ItemKind::PointItem);
        spawn_item(&mut commands, pos + Vec2::new(-8.0, 0.0), ItemKind::PowerSmall);
    }
}
```

### 4. アイテム移動・吸引

```rust
const ITEM_ATTRACT_SPEED: f32 = 400.0;
const SCORE_LINE_Y: f32 = 192.0;

pub fn item_movement_system(
    mut items: Query<(&mut Transform, &mut ItemPhysics, &ItemKind)>,
    player: Query<(&Transform, &PlayerStats), (With<Player>, Without<ItemPhysics>)>,
    time: Res<Time>,
) {
    let Ok((player_tf, stats)) = player.single() else { return };
    let player_pos = player_tf.translation.truncate();

    for (mut tf, mut physics, _) in &mut items {
        let item_pos = tf.translation.truncate();
        let dist = (item_pos - player_pos).length();

        // スコアライン以上 → 全アイテム引き寄せ
        let auto_attract = player_pos.y >= SCORE_LINE_Y;

        if physics.attracted || dist < stats.pickup_radius || auto_attract {
            physics.attracted = true;
            let dir = (player_pos - item_pos).normalize_or(Vec2::Y);
            physics.velocity = dir * ITEM_ATTRACT_SPEED;
        } else {
            // 重力落下
            physics.velocity.y -= physics.fall_speed * time.delta_secs();
            physics.velocity.y = physics.velocity.y.max(-200.0); // 落下速度上限
        }

        tf.translation += (physics.velocity * time.delta_secs()).extend(0.0);

        // 画面外に落ちたら消去
        if tf.translation.y < -260.0 {
            // DespawnOutOfBounds で代替可
        }
    }
}
```

### 5. アイテム収集

```rust
const POI_BASE_VALUE: u32 = 10_000;
const POI_MIN_VALUE: u32 = 100;
const PLAY_AREA_HALF_H: f32 = 224.0;

pub fn calc_point_item_value(player_y: f32) -> u32 {
    // スコアライン(192px)以上なら最大値
    if player_y >= SCORE_LINE_Y {
        return POI_BASE_VALUE;
    }
    // 線形補間
    let t = (player_y + PLAY_AREA_HALF_H) / (SCORE_LINE_Y + PLAY_AREA_HALF_H);
    let t = t.clamp(0.0, 1.0);
    (POI_MIN_VALUE as f32 + (POI_BASE_VALUE - POI_MIN_VALUE) as f32 * t) as u32
}

pub fn item_collection_system(
    mut commands: Commands,
    items: Query<(Entity, &Transform, &ItemKind)>,
    player: Query<(&Transform, &PlayerStats), With<Player>>,
    mut game_data: ResMut<GameData>,
    mut tracker: ResMut<FragmentTracker>,
    mut item_events: EventWriter<ItemCollectedEvent>,
) {
    let Ok((player_tf, stats)) = player.single() else { return };
    let player_pos = player_tf.translation.truncate();

    for (entity, tf, kind) in &items {
        let item_pos = tf.translation.truncate();
        let dist = (item_pos - player_pos).length();
        if dist > 8.0 { continue; } // 収集判定半径 8px

        let score = apply_item(&mut game_data, &mut tracker, *kind, player_pos.y);
        item_events.write(ItemCollectedEvent { kind: *kind, score });
        commands.entity(entity).despawn();
    }
}

fn apply_item(
    game_data: &mut GameData,
    tracker: &mut FragmentTracker,
    kind: ItemKind,
    player_y: f32,
) -> u32 {
    match kind {
        ItemKind::PowerSmall => { game_data.power = (game_data.power + 1).min(128); 0 }
        ItemKind::PowerLarge => { game_data.power = (game_data.power + 8).min(128); 0 }
        ItemKind::FullPower  => { game_data.power = 128; 0 }
        ItemKind::PointItem  => {
            let v = calc_point_item_value(player_y);
            game_data.score += v as u64;
            v
        }
        ItemKind::LifeFragment => {
            tracker.life_fragments += 1;
            if tracker.life_fragments >= 5 {
                tracker.life_fragments = 0;
                game_data.lives += 1;
            }
            0
        }
        ItemKind::BombFragment => {
            tracker.bomb_fragments += 1;
            if tracker.bomb_fragments >= 5 {
                tracker.bomb_fragments = 0;
                game_data.bombs = (game_data.bombs + 1).min(3);
            }
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_item_value_at_score_line() {
        assert_eq!(calc_point_item_value(SCORE_LINE_Y), POI_BASE_VALUE);
    }

    #[test]
    fn point_item_value_at_bottom() {
        assert_eq!(calc_point_item_value(-PLAY_AREA_HALF_H), POI_MIN_VALUE);
    }

    #[test]
    fn point_item_value_midpoint() {
        let mid = (SCORE_LINE_Y + (-PLAY_AREA_HALF_H)) / 2.0;
        let v = calc_point_item_value(mid);
        assert!(v > POI_MIN_VALUE && v < POI_BASE_VALUE);
    }
}
```

### 6. エクステンド

```rust
// app/core/src/systems/score.rs
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

---

## 参照

- `docs/03_danmaku_systems.md` § 6〜7 (アイテム・スコアシステム)
