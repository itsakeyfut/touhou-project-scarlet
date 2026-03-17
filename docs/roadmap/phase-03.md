# Phase 03: ショットシステム

## 目標

Z キーを押すとプレイヤー弾が発射され、画面上方向に飛ぶ。画面外に出た弾は自動的に消える。

## 完了条件

- [ ] Z キーで弾が発射される
- [ ] 弾が上方向に飛ぶ
- [ ] 画面外の弾が自動的に Despawn
- [ ] `ShootTimer` によるレート制限

---

## タスク詳細

### 1. コンポーネント定義

```rust
// app/core/src/components/bullet.rs
#[derive(Component)]
pub struct PlayerBullet {
    pub damage: f32,
}

#[derive(Component)]
pub struct BulletVelocity(pub Vec2);

#[derive(Component)]
pub struct ShootTimer {
    pub timer: Timer,
}

impl Default for ShootTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
        }
    }
}

#[derive(Component)]
pub struct DespawnOutOfBounds;
```

`ShootTimer` は `Player` エンティティに付ける。

### 2. イベント定義

```rust
// app/core/src/events.rs
#[derive(Event)]
pub struct ShootEvent;
```

### 3. 射撃入力システム

```rust
// app/core/src/systems/player.rs
pub fn player_shoot_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut shoot_events: EventWriter<ShootEvent>,
) {
    if keys.pressed(KeyCode::KeyZ) {
        shoot_events.write(ShootEvent);
    }
}
```

### 4. 弾スポーンシステム

```rust
// app/core/src/systems/shoot.rs
const DEFAULT_BULLET_SPEED: f32 = 600.0;
const DEFAULT_SHOOT_INTERVAL: f32 = 0.1;

pub fn spawn_player_bullets(
    mut commands: Commands,
    mut shoot_events: EventReader<ShootEvent>,
    mut player: Query<(&Transform, &mut ShootTimer), With<Player>>,
    game_data: Res<GameData>,
    time: Res<Time>,
) {
    let Ok((player_tf, mut timer)) = player.single_mut() else { return };

    timer.timer.tick(time.delta());
    if !timer.timer.just_finished() {
        return;
    }

    if shoot_events.is_empty() {
        shoot_events.clear();
        return;
    }
    shoot_events.clear();

    let origin = player_tf.translation.truncate();
    let power = game_data.power;

    // Phase 3 では霊夢タイプA（5発直線弾）のみ実装
    let count = bullet_count_from_power(power);
    let spread = 10.0f32;

    for i in 0..count {
        let x_offset = if count > 1 {
            -spread / 2.0 + spread / (count - 1) as f32 * i as f32
        } else {
            0.0
        };

        commands.spawn((
            PlayerBullet { damage: 12.0 },
            BulletVelocity(Vec2::new(x_offset * 5.0, DEFAULT_BULLET_SPEED)),
            Sprite {
                color: Color::srgb(1.0, 0.8, 0.0),
                custom_size: Some(Vec2::new(4.0, 12.0)),
                ..default()
            },
            Transform::from_translation((origin + Vec2::new(x_offset, 16.0)).extend(2.0)),
            DespawnOutOfBounds,
            DespawnOnExit(AppState::Playing),
        ));
    }
}

fn bullet_count_from_power(power: u8) -> u8 {
    match power {
        0..=31  => 1,
        32..=63 => 3,
        64..=95 => 4,
        _       => 5,
    }
}
```

### 5. 弾移動システム

```rust
// app/core/src/systems/bullet.rs
pub fn bullet_movement_system(
    mut bullets: Query<(&mut Transform, &BulletVelocity)>,
    time: Res<Time>,
) {
    for (mut tf, vel) in &mut bullets {
        tf.translation += (vel.0 * time.delta_secs()).extend(0.0);
    }
}
```

### 6. 画面外Despawnシステム

```rust
const DESPAWN_MARGIN: f32 = 32.0;
const DESPAWN_HALF_W: f32 = PLAY_AREA_HALF_W + DESPAWN_MARGIN;
const DESPAWN_HALF_H: f32 = PLAY_AREA_HALF_H + DESPAWN_MARGIN;

pub fn despawn_out_of_bounds(
    mut commands: Commands,
    bullets: Query<(Entity, &Transform), With<DespawnOutOfBounds>>,
) {
    for (entity, tf) in &bullets {
        let pos = tf.translation.truncate();
        if pos.x.abs() > DESPAWN_HALF_W || pos.y.abs() > DESPAWN_HALF_H {
            commands.entity(entity).despawn();
        }
    }
}
```

### 7. システム登録順序

```
Input → PlayerLogic(shoot_input) → BulletEmit(spawn_bullets)
     → Movement(bullet_movement) → Cleanup(despawn_out_of_bounds)
```

```rust
.add_systems(Update, player_shoot_input.in_set(GameSystemSet::Input))
.add_systems(Update, spawn_player_bullets.in_set(GameSystemSet::BulletEmit))
.add_systems(Update, bullet_movement_system.in_set(GameSystemSet::Movement))
.add_systems(Update, despawn_out_of_bounds.in_set(GameSystemSet::Cleanup))
```

### 8. `GameData` リソース（最小限）

```rust
// app/core/src/resources/game_data.rs
#[derive(Resource, Default)]
pub struct GameData {
    pub score: u64,
    pub hi_score: u64,
    pub lives: u8,
    pub bombs: u8,
    pub power: u8,       // 0-128
    pub graze_count: u32,
    pub extend_awarded: std::collections::HashSet<u64>,
}
```

---

## 参照

- `docs/03_danmaku_systems.md` § 1 (プレイヤー弾システム)
- `docs/references/vampire-survivors/app/core/src/systems/` — Bevy 0.17 のシステム登録パターン
