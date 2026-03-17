# Phase 09: パワー・ボムシステム + ボムシェーダー

## 目標

パワーレベルによってショットが変化し、Xキーでボムが発動して敵弾が消える。カウンターボムが機能する。ボム演出は `BombReimuMaterial` / `BombMarisaMaterial` シェーダーで実現する。

## 完了条件

- [ ] パワーレベル 0〜128 全段階でショット変化
- [ ] X キーでボム発動・弾消去
- [ ] カウンターボム（被弾後0.1秒以内）
- [ ] ボム中無敵
- [ ] `BombReimuMaterial` (`shaders/bomb_reimu.wgsl`) — 霊夢ボム「封魔陣」
- [ ] `BombMarisaMaterial` (`shaders/bomb_marisa.wgsl`) — 魔理沙ボム「マスタースパーク」
- [ ] ボム発動時の画面フラッシュ (HitFlashMaterial 流用)

---

## タスク詳細

### 1. BombState リソース

```rust
// app/core/src/resources/bomb.rs

const DEFAULT_BOMB_DURATION: f32 = 3.5;
const DEFAULT_BOMB_INVINCIBLE: f32 = 5.0;
const DEFAULT_COUNTER_BOMB_WINDOW: f32 = 0.1;

#[derive(Resource, Default)]
pub struct BombState {
    pub active: bool,
    pub active_timer: Timer,
    pub invincible_timer: Timer,
    pub counter_bomb_window: f32,
}

impl BombState {
    pub fn is_invincible(&self) -> bool {
        self.active || self.invincible_timer.remaining_secs() > 0.0
    }
}
```

### 2. ボム入力システム

```rust
// app/core/src/systems/bomb.rs

pub fn bomb_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut bomb_state: ResMut<BombState>,
    mut game_data: ResMut<GameData>,
    mut bomb_events: EventWriter<BombUsedEvent>,
    time: Res<Time>,
) {
    // カウンターボムウィンドウ減少
    if bomb_state.counter_bomb_window > 0.0 {
        bomb_state.counter_bomb_window -= time.delta_secs();
    }

    if !keys.just_pressed(KeyCode::KeyX) { return; }
    if bomb_state.active || game_data.bombs == 0 { return; }

    let is_counter = bomb_state.counter_bomb_window > 0.0;

    game_data.bombs -= 1;
    bomb_state.active = true;
    bomb_state.active_timer = Timer::from_seconds(DEFAULT_BOMB_DURATION, TimerMode::Once);
    bomb_state.invincible_timer = Timer::from_seconds(DEFAULT_BOMB_INVINCIBLE, TimerMode::Once);
    bomb_state.counter_bomb_window = 0.0;

    bomb_events.write(BombUsedEvent { is_counter_bomb: is_counter });
}

pub fn bomb_active_system(
    mut bomb_state: ResMut<BombState>,
    time: Res<Time>,
) {
    if !bomb_state.active { return; }

    if bomb_state.active_timer.tick(time.delta()).just_finished() {
        bomb_state.active = false;
    }
    bomb_state.invincible_timer.tick(time.delta());
}
```

### 3. ボム効果システム

```rust
pub fn bomb_effect_system(
    mut commands: Commands,
    enemy_bullets: Query<Entity, With<EnemyBullet>>,
    mut bomb_events: EventReader<BombUsedEvent>,
    mut game_data: ResMut<GameData>,
) {
    for event in bomb_events.read() {
        let mut cleared = 0u32;
        for entity in &enemy_bullets {
            commands.entity(entity).despawn();
            cleared += 1;
        }
        // 消した弾 × 10 点ボーナス
        game_data.score += (cleared as u64) * 10;
    }
}
```

### 4. ボム中の被弾無効化

```rust
// player_hit_detection に追加
pub fn player_hit_detection(
    // ...
    bomb_state: Res<BombState>,
) {
    // ボム中・ボム後無敵中はスキップ
    if bomb_state.is_invincible() { return; }
    // ...
}
```

### 5. パワーレベルによるショット変化

```rust
// app/core/src/systems/shoot.rs

pub fn spawn_player_bullets_reimu_a(
    mut commands: Commands,
    mut shoot_events: EventReader<ShootEvent>,
    mut player: Query<(&Transform, &mut ShootTimer), With<Player>>,
    game_data: Res<GameData>,
    time: Res<Time>,
) {
    let Ok((player_tf, mut timer)) = player.single_mut() else { return };
    timer.timer.tick(time.delta());

    if shoot_events.is_empty() { shoot_events.clear(); return; }
    shoot_events.clear();
    if !timer.timer.just_finished() { return; }

    let power = game_data.power;
    let origin = player_tf.translation.truncate();

    // メインショット: パワーによって弾数変化
    let (main_count, spread, damage) = match power {
        0..=15  => (1, 0.0,  10.0),
        16..=31 => (2, 8.0,  11.0),
        32..=47 => (3, 10.0, 12.0),
        48..=63 => (3, 10.0, 13.0),
        64..=79 => (4, 12.0, 14.0),
        80..=95 => (4, 12.0, 15.0),
        96..=111 => (5, 14.0, 16.0),
        112..=127 => (5, 14.0, 17.0),
        _ => (5, 14.0, 18.0), // 128 (MAX)
    };

    for i in 0..main_count {
        let x = if main_count > 1 {
            -spread / 2.0 + spread / (main_count - 1) as f32 * i as f32
        } else { 0.0 };

        commands.spawn((
            PlayerBullet { damage },
            BulletVelocity(Vec2::new(x * 5.0, 600.0)),
            Sprite {
                color: Color::srgb(1.0, 0.8, 0.0),
                custom_size: Some(Vec2::new(4.0, 12.0)),
                ..default()
            },
            Transform::from_translation((origin + Vec2::new(x, 16.0)).extend(2.0)),
            DespawnOutOfBounds,
            DespawnOnExit(AppState::Playing),
        ));
    }

    // パワー32以上: サブショット（追尾霊）
    if power >= 32 {
        // Phase 3 では省略、後で実装
    }
}
```

### 6. ShootTimer をパワーレベルで更新

```rust
pub fn update_shoot_timer_from_power(
    mut players: Query<(&mut ShootTimer, &GameData), With<Player>>,
    game_data: Res<GameData>,
) {
    // GameData はリソース、ShootTimer はコンポーネント
    // 実際は power を PlayerStats にキャッシュするか、別途変換システムを持つ
}

fn shoot_interval_from_power(power: u8) -> f32 {
    match power {
        0..=15  => 0.200,
        16..=31 => 0.160,
        32..=47 => 0.140,
        48..=63 => 0.120,
        64..=95 => 0.100,
        96..=127 => 0.083,
        _ => 0.070,
    }
}
```

### 7. 霊夢ボムビジュアル（仮）

```rust
// app/core/src/systems/effects/flash.rs

pub fn spawn_reimu_bomb_effect(
    mut commands: Commands,
    mut bomb_events: EventReader<BombUsedEvent>,
    player: Query<&Transform, With<Player>>,
    selected: Res<SelectedCharacter>,
) {
    if !matches!(selected.character, CharacterType::Reimu) { return; }

    for _ in bomb_events.read() {
        if let Ok(player_tf) = player.single() {
            // 円形バリア（仮グラフィック: 大きい白い円）
            commands.spawn((
                Sprite {
                    color: Color::srgba(1.0, 0.8, 0.9, 0.5),
                    custom_size: Some(Vec2::splat(300.0)),
                    ..default()
                },
                Transform::from_translation(player_tf.translation),
                FadeOut { timer: Timer::from_seconds(3.5, TimerMode::Once) },
                DespawnOnExit(AppState::Playing),
            ));
        }
    }
}
```

### 8. FadeOut コンポーネント

```rust
// app/ui/src/components.rs

#[derive(Component)]
pub struct FadeOut {
    pub timer: Timer,
}

pub fn fade_out_system(
    mut commands: Commands,
    mut fading: Query<(Entity, &mut Sprite, &mut FadeOut)>,
    time: Res<Time>,
) {
    for (entity, mut sprite, mut fade) in &mut fading {
        fade.timer.tick(time.delta());
        let alpha = 1.0 - fade.timer.fraction();
        sprite.color.set_alpha(alpha);
        if fade.timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}
```

### シェーダー: 霊夢ボム・魔理沙ボム

**霊夢「封魔陣」**: 全画面Mesh2dに `BombReimuMaterial` を適用。`expand_radius` を時間で0→1に変化させて結界展開を演出:

```rust
pub fn spawn_reimu_bomb_effect(
    mut commands: Commands,
    mut bomb_events: EventReader<BombUsedEvent>,
    player: Query<&Transform, With<Player>>,
    mut bomb_materials: ResMut<Assets<BombReimuMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    selected: Res<SelectedCharacter>,
) {
    if !matches!(selected.character, CharacterType::Reimu) { return; }
    for _ in bomb_events.read() {
        let Ok(player_tf) = player.single() else { continue };
        commands.spawn((
            BombVisualMarker,
            Mesh2d(meshes.add(Circle::new(300.0))),
            MeshMaterial2d(bomb_materials.add(BombReimuMaterial {
                time: 0.0,
                expand_radius: 0.0,
                _padding: Vec2::ZERO,
            })),
            Transform::from_translation(player_tf.translation),
            FadeOut { timer: Timer::from_seconds(3.5, TimerMode::Once) },
            DespawnOnExit(AppState::Playing),
        ));
    }
}
```

**魔理沙「マスタースパーク」**: プレイヤーから上方向へ伸びる細長い矩形Mesh2dに `BombMarisaMaterial` を適用。

---

## 参照

- `docs/03_danmaku_systems.md` § 4 (ボムシステム)
- `docs/01_specification.md` § パワーシステム・ボムシステム
- `docs/10_shaders_wgsl.md` § 8 (ボム演出シェーダー)
