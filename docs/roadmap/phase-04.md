# Phase 04: 基本弾幕システム + 弾グローシェーダー

## 目標

ダミー敵エンティティが弾幕パターンを発射できる。弾には `BulletGlowMaterial` によるグロー発光を適用する。当たり判定はまだなし。

## 完了条件

- [ ] `BulletEmitter` コンポーネントが機能する
- [ ] Ring（円形）パターン実装
- [ ] Aimed（自機狙い）パターン実装
- [ ] Spread（扇形）パターン実装
- [ ] Spiral（螺旋）パターン実装
- [ ] `BulletGlowMaterial` (`shaders/bullet_glow.wgsl`) の実装・適用
- [ ] `BulletTrailMaterial` (`shaders/bullet_trail.wgsl`) の実装・適用
- [ ] `ScarletShadersPlugin` に両マテリアルを登録
- [ ] `EnemyBulletKind` ごとに異なるグロー色

---

## タスク詳細

### 1. コンポーネント

```rust
// app/core/src/components/bullet.rs に追加

#[derive(Component, Clone, Copy, PartialEq)]
pub enum EnemyBulletKind {
    SmallRound,
    MediumRound,
    LargeRound,
    Rice,
    Knife,
    Star,
    Bubble,
}

impl EnemyBulletKind {
    pub fn collision_radius(&self) -> f32 {
        match self {
            Self::Knife       => 4.0,
            Self::SmallRound  => 4.0,
            Self::Rice        => 5.0,
            Self::MediumRound => 7.0,
            Self::Star        => 8.0,
            Self::Bubble      => 9.0,
            Self::LargeRound  => 11.0,
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Self::SmallRound  => Color::srgb(1.0, 0.2, 0.2),
            Self::MediumRound => Color::srgb(0.2, 0.5, 1.0),
            Self::LargeRound  => Color::srgb(0.8, 0.2, 0.8),
            Self::Rice        => Color::srgb(1.0, 0.8, 0.2),
            Self::Knife       => Color::srgb(0.3, 1.0, 0.3),
            Self::Star        => Color::srgb(1.0, 1.0, 0.2),
            Self::Bubble      => Color::srgb(0.4, 0.9, 1.0),
        }
    }

    pub fn sprite_size(&self) -> Vec2 {
        let r = self.collision_radius();
        Vec2::splat(r * 2.0)
    }
}

#[derive(Component)]
pub struct EnemyBullet {
    pub damage: u8,
}

#[derive(Component)]
pub struct BulletEmitter {
    pub pattern: BulletPattern,
    pub bullet_kind: EnemyBulletKind,
    pub timer: Timer,
    pub active: bool,
}

#[derive(Clone)]
pub enum BulletPattern {
    Ring { count: u8, speed: f32 },
    Aimed { count: u8, spread_deg: f32, speed: f32 },
    Spread { count: u8, spread_deg: f32, speed: f32, angle_offset: f32 },
    Spiral { arms: u8, speed: f32, rotation_speed_deg: f32 },
}
```

### 2. 弾幕パターン実装

```rust
// app/core/src/systems/danmaku/patterns.rs

pub fn emit_ring(
    commands: &mut Commands,
    origin: Vec2,
    count: u8,
    speed: f32,
    kind: EnemyBulletKind,
) {
    let step = 360.0 / count as f32;
    for i in 0..count {
        let angle = step * i as f32;
        let dir = Vec2::from_angle(angle.to_radians());
        spawn_enemy_bullet(commands, origin, dir * speed, kind);
    }
}

pub fn emit_aimed(
    commands: &mut Commands,
    origin: Vec2,
    player_pos: Vec2,
    count: u8,
    spread_deg: f32,
    speed: f32,
    kind: EnemyBulletKind,
) {
    let base_dir = (player_pos - origin).normalize_or(Vec2::NEG_Y);
    let base_angle = base_dir.y.atan2(base_dir.x);
    let half = spread_deg.to_radians() / 2.0;
    let step = if count > 1 {
        spread_deg.to_radians() / (count - 1) as f32
    } else {
        0.0
    };

    for i in 0..count {
        let angle = base_angle - half + step * i as f32;
        let dir = Vec2::from_angle(angle);
        spawn_enemy_bullet(commands, origin, dir * speed, kind);
    }
}

fn spawn_enemy_bullet(
    commands: &mut Commands,
    origin: Vec2,
    velocity: Vec2,
    kind: EnemyBulletKind,
) {
    commands.spawn((
        EnemyBullet { damage: 1 },
        kind,
        BulletVelocity(velocity),
        Sprite {
            color: kind.color(),
            custom_size: Some(kind.sprite_size()),
            ..default()
        },
        Transform::from_translation(origin.extend(1.5)),
        DespawnOutOfBounds,
        DespawnOnExit(AppState::Playing),
    ));
}
```

### 3. Spiral エミッター（ステートフル）

```rust
// app/core/src/systems/danmaku/emitter.rs

#[derive(Component)]
pub struct SpiralState {
    pub current_angle: f32,
}

pub fn update_spiral_emitters(
    mut commands: Commands,
    mut spirals: Query<(&Transform, &BulletEmitter, &mut SpiralState)>,
    player: Query<&Transform, (With<Player>, Without<BulletEmitter>)>,
    time: Res<Time>,
) {
    let Ok(player_tf) = player.single() else { return };
    let player_pos = player_tf.translation.truncate();

    for (tf, emitter, mut spiral_state) in &mut spirals {
        if !emitter.active { continue; }
        let origin = tf.translation.truncate();

        if let BulletPattern::Spiral { arms, speed, rotation_speed_deg } = emitter.pattern {
            spiral_state.current_angle += rotation_speed_deg * time.delta_secs();
            let arm_gap = 360.0 / arms as f32;
            for arm in 0..arms {
                let angle = spiral_state.current_angle + arm_gap * arm as f32;
                let dir = Vec2::from_angle(angle.to_radians());
                spawn_enemy_bullet(&mut commands, origin, dir * speed, emitter.bullet_kind);
            }
        }
    }
}
```

### 4. 汎用エミッターシステム

```rust
pub fn update_bullet_emitters(
    mut commands: Commands,
    mut emitters: Query<(&Transform, &mut BulletEmitter), Without<SpiralState>>,
    player: Query<&Transform, (With<Player>, Without<BulletEmitter>)>,
    time: Res<Time>,
) {
    let Ok(player_tf) = player.single() else { return };
    let player_pos = player_tf.translation.truncate();

    for (tf, mut emitter) in &mut emitters {
        if !emitter.active { continue; }
        if !emitter.timer.tick(time.delta()).just_finished() { continue; }

        let origin = tf.translation.truncate();
        match emitter.pattern.clone() {
            BulletPattern::Ring { count, speed } => {
                emit_ring(&mut commands, origin, count, speed, emitter.bullet_kind);
            }
            BulletPattern::Aimed { count, spread_deg, speed } => {
                emit_aimed(&mut commands, origin, player_pos, count, spread_deg, speed, emitter.bullet_kind);
            }
            BulletPattern::Spread { count, spread_deg, speed, angle_offset } => {
                // 固定角度の扇形（自機狙いなし）
                let step = spread_deg / (count.max(2) - 1) as f32;
                for i in 0..count {
                    let angle = -spread_deg / 2.0 + step * i as f32 + angle_offset;
                    let dir = Vec2::from_angle(angle.to_radians() - std::f32::consts::FRAC_PI_2);
                    spawn_enemy_bullet(&mut commands, origin, dir * speed, emitter.bullet_kind);
                }
            }
            BulletPattern::Spiral { .. } => { /* SpiralState を持つエンティティで処理 */ }
        }
    }
}
```

### 5. ダミー敵でテスト

デバッグ用に固定位置のダミー敵をスポーンして弾幕パターンを確認:

```rust
// テスト用 (Playing OnEnter に追加)
pub fn spawn_debug_enemies(mut commands: Commands) {
    // Ring パターン
    commands.spawn((
        BulletEmitter {
            pattern: BulletPattern::Ring { count: 8, speed: 120.0 },
            bullet_kind: EnemyBulletKind::SmallRound,
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            active: true,
        },
        SpiralState { current_angle: 0.0 },
        Sprite {
            color: Color::srgb(0.5, 0.5, 0.5),
            custom_size: Some(Vec2::splat(24.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 100.0, 1.0),
        DespawnOnExit(AppState::Playing),
    ));
}
```

### 6. デバッグ: 当たり判定可視化

```rust
#[cfg(debug_assertions)]
pub fn debug_bullet_hitbox(
    mut gizmos: Gizmos,
    bullets: Query<(&Transform, &EnemyBulletKind), With<EnemyBullet>>,
) {
    for (tf, kind) in &bullets {
        gizmos.circle_2d(
            tf.translation.truncate(),
            kind.collision_radius(),
            Color::srgba(0.0, 1.0, 0.0, 0.4),
        );
    }
}
```

### シェーダー設計決定（確定）

| 項目 | 決定内容 |
|---|---|
| カメラ | `hdr: true` + `BloomPlugin` を有効化 |
| BulletGlowMaterial | 手続き的な光る円（テクスチャなし）。Phase 19 で差替予定 |
| BulletGlowMaterial パラメータ | `color: LinearRgba`, `glow_intensity: f32`, `time: f32` |
| BulletTrailMaterial | UV フェードアウト実装 |
| BulletTrailMaterial パラメータ | `color: LinearRgba`, `length: f32`, `alpha_falloff: f32`, `time: f32` |
| ScarletShadersPlugin 配置 | `scarlet-core/src/shaders/` モジュール（新クレートなし） |
| main.rs 登録 | Phase 04 の PR に含める |

### シェーダー: 弾グロー追加

`ScarletShadersPlugin` の初期実装を行う:

```rust
// app/core/src/shaders/plugin.rs

pub struct ScarletShadersPlugin;

impl Plugin for ScarletShadersPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(Material2dPlugin::<BulletGlowMaterial>::default())
            .add_plugins(Material2dPlugin::<BulletTrailMaterial>::default())
            .add_systems(Update, update_bullet_glow_time
                .run_if(in_state(AppState::Playing)));
    }
}
```

弾スポーン時にMesh2d + Material2dを使用（Spriteの代わり）:

```rust
// spawn_enemy_bullet_with_shader の利用例
commands.spawn((
    EnemyBullet { damage: 1 },
    kind,
    BulletVelocity(velocity),
    Mesh2d(meshes.add(Circle::new(kind.visual_radius()))),
    MeshMaterial2d(glow_materials.add(BulletGlowMaterial {
        color: kind.glow_color(),
        glow_intensity: 1.5,
        time: 0.0,
        texture: Handle::default(), // Phase 19でスプライトシートに差し替え
    })),
    Transform::from_translation(origin.extend(1.5)),
    DespawnOutOfBounds,
    DespawnOnExit(AppState::Playing),
));
```

---

## 参照

- `docs/03_danmaku_systems.md` § 2 (弾幕パターンシステム)
- `docs/10_shaders_wgsl.md` § 3 (弾グロー), § 4 (弾残像)
