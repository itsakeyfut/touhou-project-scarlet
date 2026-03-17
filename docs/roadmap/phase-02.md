# Phase 02: コアシステム（プレイヤー移動）

## 目標

プレイヤーキャラクターが画面に表示され、矢印キーで移動できる。Shift押し中は低速移動。プレイエリア外に出ない。

## 完了条件

- [ ] プレイヤーが `Playing` 状態でスポーン
- [ ] 矢印キー/WASD で移動
- [ ] Shift 低速移動
- [ ] プレイエリア境界でクランプ
- [ ] `DespawnOnExit(AppState::Playing)` 適用

---

## タスク詳細

### 1. コンポーネント定義

`app/core/src/components/player.rs`:
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerStats {
    pub base_speed: f32,
    pub slow_speed: f32,
    pub hitbox_radius: f32,
    pub graze_radius: f32,
    pub pickup_radius: f32,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            base_speed: 200.0,
            slow_speed: 100.0,
            hitbox_radius: 2.5,
            graze_radius: 16.0,
            pickup_radius: 80.0,
        }
    }
}

#[derive(Component)]
pub struct InvincibilityTimer {
    pub timer: Timer,
}
```

### 2. プレイエリア定数

`app/core/src/lib.rs` または専用 `constants` モジュールには入れず、各ファイルの先頭に private 定数として定義:

```rust
// app/core/src/systems/player.rs の先頭
const PLAY_AREA_HALF_W: f32 = 192.0;
const PLAY_AREA_HALF_H: f32 = 224.0;
const DEFAULT_BASE_SPEED: f32 = 200.0;
const DEFAULT_SLOW_SPEED: f32 = 100.0;
```

### 3. プレイヤースポーンシステム

```rust
// OnEnter(AppState::Playing) で呼ばれる
pub fn spawn_player(
    mut commands: Commands,
    assets: Res<ScarletAssets>,
    selected: Res<SelectedCharacter>,
) {
    let sprite = match selected.character {
        CharacterType::Reimu => assets.player_reimu.clone(),
        CharacterType::Marisa => assets.player_marisa.clone(),
    };

    commands.spawn((
        Player,
        PlayerStats::default(),
        Sprite { image: sprite, ..default() },
        Transform::from_xyz(0.0, -150.0, 1.0), // 初期位置: 画面下寄り
        DespawnOnExit(AppState::Playing),
    ));
}
```

Phase 1〜2 ではアセットがないため、仮の色付き矩形を使う:
```rust
commands.spawn((
    Player,
    PlayerStats::default(),
    Sprite {
        color: Color::srgb(1.0, 0.5, 0.5),
        custom_size: Some(Vec2::splat(16.0)),
        ..default()
    },
    Transform::from_xyz(0.0, -150.0, 1.0),
    DespawnOnExit(AppState::Playing),
));
```

### 4. 移動システム

```rust
pub fn player_movement_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut player: Query<(&mut Transform, &PlayerStats), With<Player>>,
    time: Res<Time>,
) {
    let Ok((mut tf, stats)) = player.single_mut() else { return };

    let slow = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    let speed = if slow { stats.slow_speed } else { stats.base_speed };

    let mut dir = Vec2::ZERO;
    if keys.pressed(KeyCode::ArrowUp)    || keys.pressed(KeyCode::KeyW) { dir.y += 1.0; }
    if keys.pressed(KeyCode::ArrowDown)  || keys.pressed(KeyCode::KeyS) { dir.y -= 1.0; }
    if keys.pressed(KeyCode::ArrowLeft)  || keys.pressed(KeyCode::KeyA) { dir.x -= 1.0; }
    if keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD) { dir.x += 1.0; }

    if dir != Vec2::ZERO {
        let delta = dir.normalize() * speed * time.delta_secs();
        tf.translation += delta.extend(0.0);
    }

    // エリア境界クランプ
    tf.translation.x = tf.translation.x.clamp(-PLAY_AREA_HALF_W, PLAY_AREA_HALF_W);
    tf.translation.y = tf.translation.y.clamp(-PLAY_AREA_HALF_H, PLAY_AREA_HALF_H);
}
```

### 5. カメラセットアップ

`app/ui/src/camera.rs`:
```rust
pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
```

`Startup` システムとして登録。

### 6. SystemSet 設定

```rust
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameSystemSet {
    Input,
    PlayerLogic,
    BulletEmit,
    Movement,
    Collision,
    GameLogic,
    StageControl,
    Effects,
    Cleanup,
}
```

`lib.rs` で `.chain()` を使って順序を保証:
```rust
.configure_sets(Update, (
    GameSystemSet::Input,
    GameSystemSet::PlayerLogic,
    GameSystemSet::BulletEmit,
    GameSystemSet::Movement,
    GameSystemSet::Collision,
    GameSystemSet::GameLogic,
    GameSystemSet::StageControl,
    GameSystemSet::Effects,
    GameSystemSet::Cleanup,
).chain().run_if(in_state(AppState::Playing)))
```

移動システムを `GameSystemSet::Movement` に配置:
```rust
.add_systems(Update, player_movement_system
    .in_set(GameSystemSet::Movement)
    .run_if(in_state(AppState::Playing)))
```

### 7. デバッグ: プレイエリア境界表示

```rust
#[cfg(debug_assertions)]
pub fn draw_play_area(mut gizmos: Gizmos) {
    gizmos.rect_2d(
        Vec2::ZERO,
        Vec2::new(PLAY_AREA_HALF_W * 2.0, PLAY_AREA_HALF_H * 2.0),
        Color::srgba(0.5, 0.5, 1.0, 0.3),
    );
}
```

### 8. テスト

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use bevy::state::app::StatesPlugin;

    #[test]
    fn player_clamps_to_play_area() {
        // プレイヤーがエリア外に移動しようとしてもクランプされる
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        // ... テスト実装
    }
}
```

---

## 参照

- `docs/references/vampire-survivors/app/core/src/systems/player.rs` — 移動システム参照
- `docs/04_ui_ux.md` — 入力マッピング
