# Phase 18: シェーダー・エフェクト・ポリッシュ

## 目標

WGSLシェーダーによる美麗な演出が完成する。ブルームポストプロセスが動き、全スペルカードのバランス調整が完了する。

## 完了条件

- [ ] `post_bloom.wgsl` — ブルームポストプロセス（弾幕のグロー滲み）
- [ ] `post_crt.wgsl` — CRTピクセルエフェクト（オプション）
- [ ] 全シェーダーのパラメータ最終調整
- [ ] パーティクルエフェクト（被弾・撃破・グレイズスパーク）
- [ ] スペルカード背景: 全ボス対応（7パターン）
- [ ] 全難易度のバランス調整
- [ ] 60fps安定確認（弾幕ピーク時 + ブルーム有効）
- [ ] 全スペルカード・全難易度の動作確認

---

## タスク詳細

### 1. パーティクルシステム

```rust
// app/core/src/systems/effects/particles.rs

#[derive(Component)]
pub struct Particle {
    pub velocity: Vec2,
    pub lifetime: Timer,
    pub initial_alpha: f32,
}

pub fn particle_system(
    mut commands: Commands,
    mut particles: Query<(Entity, &mut Transform, &mut Sprite, &mut Particle)>,
    time: Res<Time>,
) {
    for (entity, mut tf, mut sprite, mut particle) in &mut particles {
        particle.lifetime.tick(time.delta());
        if particle.lifetime.just_finished() {
            commands.entity(entity).despawn();
            continue;
        }

        tf.translation += (particle.velocity * time.delta_secs()).extend(0.0);
        particle.velocity *= 0.95; // 空気抵抗

        let alpha = particle.initial_alpha * (1.0 - particle.lifetime.fraction());
        sprite.color.set_alpha(alpha);
    }
}

pub fn spawn_explosion(commands: &mut Commands, pos: Vec2, color: Color, count: u8, speed: f32) {
    use rand::Rng;
    let mut rng = rand::rng();

    for _ in 0..count {
        let angle = rng.random::<f32>() * std::f32::consts::TAU;
        let speed = speed * (0.5 + rng.random::<f32>() * 0.5);
        let size = 3.0 + rng.random::<f32>() * 4.0;

        commands.spawn((
            Particle {
                velocity: Vec2::from_angle(angle) * speed,
                lifetime: Timer::from_seconds(0.3 + rng.random::<f32>() * 0.4, TimerMode::Once),
                initial_alpha: 1.0,
            },
            Sprite {
                color,
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_translation(pos.extend(3.0)),
        ));
    }
}
```

### 2. 被弾エフェクト

```rust
// app/core/src/systems/effects/flash.rs

pub fn player_hit_effect(
    mut commands: Commands,
    mut hit_events: EventReader<PlayerHitEvent>,
    player: Query<&Transform, With<Player>>,
) {
    for _ in hit_events.read() {
        if let Ok(tf) = player.single() {
            let pos = tf.translation.truncate();
            spawn_explosion(
                &mut commands, pos,
                Color::srgb(1.0, 0.5, 0.5),
                16, 150.0,
            );
        }
    }
}

pub fn enemy_defeat_effect(
    mut commands: Commands,
    mut defeated_events: EventReader<EnemyDefeatedEvent>,
    enemies: Query<(&Transform, &EnemyKind)>,
) {
    for event in defeated_events.read() {
        let Ok((tf, kind)) = enemies.get(event.enemy_entity) else { continue };
        let pos = tf.translation.truncate();
        let (color, count) = match kind {
            EnemyKind::Fairy => (Color::srgb(0.5, 0.8, 1.0), 8u8),
            EnemyKind::Bat   => (Color::srgb(0.4, 0.2, 0.6), 6),
            EnemyKind::TallFairy => (Color::srgb(0.8, 0.6, 1.0), 12),
        };
        spawn_explosion(&mut commands, pos, color, count, 100.0);
    }
}
```

### 3. グレイズスパーク

```rust
pub fn graze_spark_effect(
    mut commands: Commands,
    mut graze_events: EventReader<GrazeEvent>,
    bullets: Query<&Transform, With<EnemyBullet>>,
) {
    for event in graze_events.read() {
        let Ok(bullet_tf) = bullets.get(event.bullet_entity) else { continue };
        let pos = bullet_tf.translation.truncate();

        // 小さなスパーク（2〜3個）
        spawn_explosion(
            &mut commands, pos,
            Color::srgb(1.0, 1.0, 0.5),
            3, 60.0,
        );
    }
}
```

### 4. スペルカード背景演出

```rust
// app/core/src/systems/effects/spell_card_bg.rs

#[derive(Component)]
pub struct SpellCardBackground {
    pub color: Color,
    pub pattern_timer: Timer,
}

pub fn spawn_spell_card_bg(
    mut commands: Commands,
    mut phase_events: EventReader<BossPhaseChangedEvent>,
    bosses: Query<&Boss>,
) {
    for event in phase_events.read() {
        let Ok(boss) = bosses.get(event.entity) else { continue };
        let phase = &boss.phases[event.phase];

        if phase.is_spell_card {
            // スペルカード用の暗い背景オーバーレイ
            commands.spawn((
                SpellCardBackground {
                    color: Color::srgba(0.0, 0.0, 0.2, 0.6),
                    pattern_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                },
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Px(384.0),
                    height: Val::Px(448.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.2, 0.4)),
                GlobalZIndex(-1),
                DespawnOnExit(AppState::Playing),
            ));
        }
    }
}

pub fn despawn_spell_card_bg_on_phase_end(
    mut commands: Commands,
    bgs: Query<Entity, With<SpellCardBackground>>,
    mut phase_events: EventReader<BossPhaseChangedEvent>,
    bosses: Query<&Boss>,
) {
    for event in phase_events.read() {
        let Ok(boss) = bosses.get(event.entity) else { continue };
        let prev_phase_idx = event.phase.saturating_sub(1);
        if prev_phase_idx < boss.phases.len() && boss.phases[prev_phase_idx].is_spell_card {
            for entity in &bgs {
                commands.entity(entity).despawn();
            }
        }
    }
}
```

### 5. 画面揺れ（ボス撃破時）

```rust
// app/core/src/systems/effects/shake.rs

#[derive(Resource, Default)]
pub struct ScreenShake {
    pub intensity: f32,
    pub duration: Timer,
}

pub fn screen_shake_system(
    mut shake: ResMut<ScreenShake>,
    mut camera: Query<&mut Transform, With<Camera2d>>,
    time: Res<Time>,
) {
    if !shake.duration.tick(time.delta()).finished() {
        if let Ok(mut cam_tf) = camera.single_mut() {
            use rand::Rng;
            let mut rng = rand::rng();
            let offset = Vec2::new(
                (rng.random::<f32>() - 0.5) * shake.intensity * 2.0,
                (rng.random::<f32>() - 0.5) * shake.intensity * 2.0,
            );
            // カメラをプレイエリア中心 + ランダムオフセット
            cam_tf.translation.x = offset.x;
            cam_tf.translation.y = offset.y;
        }
    } else {
        if let Ok(mut cam_tf) = camera.single_mut() {
            cam_tf.translation.x = 0.0;
            cam_tf.translation.y = 0.0;
        }
    }
}

pub fn trigger_shake_on_player_hit(
    mut hit_events: EventReader<PlayerHitEvent>,
    mut shake: ResMut<ScreenShake>,
) {
    for _ in hit_events.read() {
        shake.intensity = 6.0;
        shake.duration = Timer::from_seconds(0.3, TimerMode::Once);
    }
}
```

### 6. バランス調整チェックリスト

各ステージ・難易度ごとに以下を確認:

```
[ ] Stage 1 Easy   - 初心者が20分以内でクリアできる
[ ] Stage 1 Normal - 中級者が30分程度でクリアできる
[ ] Stage 1 Hard   - 上級者向け難易度
[ ] Stage 1 Lunatic - 弾幕STG上級者向け
[ ] Stage 2 各難易度
[ ] Stage 3 各難易度
[ ] Stage 4 各難易度
[ ] Stage 5 各難易度
[ ] Stage 6 各難易度
[ ] Extra
```

**調整パラメータ**（RON設定または定数）:

```rust
// 敵弾速度乗数
// 敵弾数乗数
// ボスHP乗数
// ボム持続時間
// 無敵時間
// スペルカードボーナス初期値・減少率
```

### 7. パフォーマンス最適化

```bash
# プロファイリング実行
cargo run -p touhou-project-scarlet --release --features bevy/trace_tracy
```

確認ポイント:
- `despawn_out_of_bounds` システムの CPU 時間
- `graze_detection_system` の HashSet 操作コスト
- 弾幕ピーク時（弾 500+ エンティティ）の ECS クエリ性能

最適化手法:
- `DespawnOutOfBounds` の判定を AABB で高速化（`length_squared` 使用済み）
- グレイズ HashSet を `EntityHashSet` に変更（`bevy::utils` 提供）
- ボス弾幕のスポーンを分散（`Commands::spawn_batch` 活用）

### ブルームポストプロセス統合

Bevy 0.17 標準の `BloomSettings` を使ってカメラにブルームを設定する:

```rust
// app/ui/src/camera.rs

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            hdr: true,  // HDR必須
            ..default()
        },
        Bloom {
            intensity: 0.25,
            low_frequency_boost: 0.5,
            low_frequency_boost_curvature: 0.5,
            high_pass_frequency: 1.0,
            prefilter: BloomPrefilter {
                threshold: 0.6,
                threshold_softness: 0.2,
            },
            composite_mode: BloomCompositeMode::Additive,
        },
    ));
}
```

弾のグロー色を `LinearRgba` の HDR 値（1.0以上）で設定することでブルームが発光する:

```rust
impl EnemyBulletKind {
    pub fn glow_color(&self) -> LinearRgba {
        match self {
            Self::SmallRound  => LinearRgba::new(2.0, 0.3, 0.3, 1.0),  // 赤い発光弾
            Self::MediumRound => LinearRgba::new(0.3, 0.8, 3.0, 1.0),  // 青い発光弾
            Self::LargeRound  => LinearRgba::new(2.0, 0.3, 2.5, 1.0),  // 紫
            Self::Star        => LinearRgba::new(2.5, 2.5, 0.3, 1.0),  // 黄
            Self::Knife       => LinearRgba::new(0.5, 3.0, 0.5, 1.0),  // 緑
            _                 => LinearRgba::new(1.5, 1.5, 1.5, 1.0),
        }
    }
}
```

### シェーダーパラメータ最終調整チェックリスト

```
[ ] bullet_glow.wgsl: glow_intensity の弾種別調整
[ ] bullet_trail.wgsl: trail_length の速度依存調整
[ ] graze_field.wgsl: 低速/通常モードの視認性バランス
[ ] hit_flash.wgsl: フラッシュ持続時間の調整
[ ] spell_card_bg.wgsl: 全7パターンの色・速度調整
[ ] bomb_reimu.wgsl: 結界の展開速度・彩度
[ ] bomb_marisa.wgsl: スパーク幅・虹色の鮮やかさ
[ ] pixel_outline.wgsl: アウトライン幅 (1px vs 2px)
[ ] Camera Bloom: intensity / threshold の最終値
```

---

## 参照

- `docs/03_danmaku_systems.md` § 9 (判定可視化)
- `docs/10_shaders_wgsl.md` — 全シェーダー設計
- `docs/references/vampire-survivors/app/core/src/systems/effects/`
