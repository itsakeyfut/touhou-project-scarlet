# Phase 15: Stage 4〜5 実装（パチュリー・十六夜咲夜）

## 目標

Stage 4（パチュリー・知識）と Stage 5（十六夜咲夜）が通しプレイできる。

## 完了条件

- [ ] パチュリー全スペルカード実装（7種）
- [ ] 十六夜咲夜全スペルカード実装（6種）
- [ ] Stage 4A/4B ルート分岐ロジック
- [ ] 時間停止演出（咲夜専用）
- [ ] Stage 4〜5 ザコ波スクリプト

---

## タスク詳細

### Stage 4A: パチュリー・知識

パチュリーは「五行」属性の弾幕を使う。各属性で弾の色・挙動が変わる。

**スペルカード一覧**

| フェーズ | 名前 | 属性 | 説明 |
|---|---|---|---|
| 通常1 | — | — | 自機狙い + 魔法陣弾 |
| SC1 | 「火符「アグニシャイン」」 | 火 | 大きな火球を扇形に発射 |
| 通常2 | — | — | 水弾の螺旋 |
| SC2 | 「水符「プリンセスウンディネ」」 | 水 | 泡弾で画面を埋める |
| SC3 | 「木符「エーテルフレア」」 | 木 | 放物線を描く木の葉弾 |
| SC4 | 「金符「メタルファティーグ」」 | 金 | 高速のナイフ弾 |
| SC5 | 「土符「レイジィトリリオン」」 | 土 | 重力で落ちる大弾 |
| SC6 | 「火水符「フォグウェザー」」 | 火+水 | 2属性の複合弾幕 |
| SC7 | 「日符「ロイヤルフレア」」 | 日 | 超高密度の全方位弾幕 |

```rust
// app/core/src/systems/boss/bosses/patchouli.rs

pub fn patchouli_phases(difficulty: Difficulty) -> Vec<BossPhaseData> {
    let params = DifficultyParams::for_difficulty(difficulty);

    vec![
        // 通常1: 魔法陣ランダム弾
        BossPhaseData {
            hp: 900.0 * params.enemy_hp_multiplier,
            hp_max: 900.0 * params.enemy_hp_multiplier,
            is_spell_card: false,
            spell_card_name: None,
            time_limit_secs: 30.0,
            pattern: BulletPattern::Ring {
                count: (5.0 * params.bullet_count_multiplier) as u8,
                speed: 110.0 * params.bullet_speed_multiplier,
            },
            movement: BossMovement::Pendulum { amplitude: 60.0, frequency: 0.4, base_x: 0.0 },
            spell_card_bonus: 0,
        },
        // SC1: アグニシャイン
        BossPhaseData {
            hp: 1100.0 * params.enemy_hp_multiplier,
            hp_max: 1100.0 * params.enemy_hp_multiplier,
            is_spell_card: true,
            spell_card_name: Some("火符「アグニシャイン」".to_string()),
            time_limit_secs: 45.0,
            pattern: BulletPattern::Spread {
                count: (5.0 * params.bullet_count_multiplier) as u8,
                spread_deg: 50.0,
                speed: 90.0 * params.bullet_speed_multiplier,
                angle_offset: 0.0,
            },
            movement: BossMovement::Pendulum { amplitude: 80.0, frequency: 0.5, base_x: 0.0 },
            spell_card_bonus: 1_100_000,
        },
        // SC2: プリンセスウンディネ
        BossPhaseData {
            hp: 1200.0 * params.enemy_hp_multiplier,
            hp_max: 1200.0 * params.enemy_hp_multiplier,
            is_spell_card: true,
            spell_card_name: Some("水符「プリンセスウンディネ」".to_string()),
            time_limit_secs: 50.0,
            pattern: BulletPattern::Spiral {
                arms: 3,
                speed: 70.0 * params.bullet_speed_multiplier,
                rotation_speed_deg: 60.0,
            },
            movement: BossMovement::Static,
            spell_card_bonus: 1_200_000,
        },
        // SC7: ロイヤルフレア（最終スペルカード）
        BossPhaseData {
            hp: 2000.0 * params.enemy_hp_multiplier,
            hp_max: 2000.0 * params.enemy_hp_multiplier,
            is_spell_card: true,
            spell_card_name: Some("日符「ロイヤルフレア」".to_string()),
            time_limit_secs: 60.0,
            pattern: BulletPattern::Ring {
                count: (24.0 * params.bullet_count_multiplier) as u8,
                speed: 140.0 * params.bullet_speed_multiplier,
            },
            movement: BossMovement::Circle { radius: 40.0, speed_deg: 60.0, center: Vec2::new(0.0, 80.0) },
            spell_card_bonus: 2_000_000,
        },
        // 他のスペルカードは実装時に追加
    ]
}
```

### Stage 4A/4B ルート分岐

原作では Stage 4A（パチュリー）と Stage 4B（小悪魔 + パチュリー）があるが、本クローンでは常に 4A ルートを採用（シンプル化）。

```rust
// 将来拡張: stage_data に route フィールドを追加可能
// pub route: StageRoute,
// pub enum StageRoute { A, B }
```

---

### Stage 5: 十六夜咲夜

咲夜は「時間停止」という特殊メカニクムを持つ。

**スペルカード一覧**

| フェーズ | 名前 | 説明 |
|---|---|---|
| 通常1 | — | ナイフ高速弾 |
| SC1 | 「時符「ミステリアスジャック」」 | 時間停止中にナイフを配置 |
| 通常2 | — | 連続ナイフ投げ |
| SC2 | 「幻符「死体返し」」 | 消えた弾が再出現 |
| SC3 | 「銀符「パーフェクトメイド」」 | 完璧な隙間ナイフ |
| SC4 | 「時符「プライベートスクウェア」」 | 時間停止後に大量ナイフ |
| SC5 | 「時符「咲夜特製ストップウォッチ」」 | 長時間停止後超高密度 |
| SC6 | 「速符「ルミネスリコシェ」」 | 壁に跳ね返るナイフ |

```rust
// app/core/src/systems/boss/bosses/sakuya.rs

/// 時間停止: 敵弾の速度を一時的に0にするコンポーネント
#[derive(Component)]
pub struct FrozenBullet {
    pub original_velocity: Vec2,
    pub frozen_timer: Timer,
}

pub fn time_stop_system(
    mut commands: Commands,
    mut bullets: Query<(Entity, &mut BulletVelocity, Option<&FrozenBullet>), With<EnemyBullet>>,
    bosses: Query<(&Boss, &TimeStopState)>,
    time: Res<Time>,
) {
    let Ok((boss, time_stop)) = bosses.single() else { return };

    if time_stop.active {
        // 全弾を停止
        for (entity, mut vel, frozen) in &mut bullets {
            if frozen.is_none() {
                let original = vel.0;
                vel.0 = Vec2::ZERO;
                commands.entity(entity).insert(FrozenBullet {
                    original_velocity: original,
                    frozen_timer: Timer::from_seconds(time_stop.duration, TimerMode::Once),
                });
            }
        }
    }
}

pub fn unfreeze_bullets(
    mut commands: Commands,
    mut bullets: Query<(Entity, &mut BulletVelocity, &mut FrozenBullet)>,
    time: Res<Time>,
) {
    for (entity, mut vel, mut frozen) in &mut bullets {
        if frozen.frozen_timer.tick(time.delta()).just_finished() {
            vel.0 = frozen.original_velocity;
            commands.entity(entity).remove::<FrozenBullet>();
        }
    }
}

#[derive(Component, Default)]
pub struct TimeStopState {
    pub active: bool,
    pub duration: f32,
    pub cooldown: Timer,
}

pub fn sakuya_phases(difficulty: Difficulty) -> Vec<BossPhaseData> {
    let params = DifficultyParams::for_difficulty(difficulty);

    vec![
        // 通常1: 高速ナイフ
        BossPhaseData {
            hp: 1000.0 * params.enemy_hp_multiplier,
            hp_max: 1000.0 * params.enemy_hp_multiplier,
            is_spell_card: false,
            spell_card_name: None,
            time_limit_secs: 30.0,
            pattern: BulletPattern::Aimed {
                count: 5,
                spread_deg: 30.0,
                speed: 160.0 * params.bullet_speed_multiplier,
            },
            movement: BossMovement::Pendulum { amplitude: 70.0, frequency: 0.6, base_x: 0.0 },
            spell_card_bonus: 0,
        },
        // SC1: ミステリアスジャック（時間停止後ナイフ配置）
        BossPhaseData {
            hp: 1300.0 * params.enemy_hp_multiplier,
            hp_max: 1300.0 * params.enemy_hp_multiplier,
            is_spell_card: true,
            spell_card_name: Some("時符「ミステリアスジャック」".to_string()),
            time_limit_secs: 45.0,
            pattern: BulletPattern::Ring {
                count: (16.0 * params.bullet_count_multiplier) as u8,
                speed: 120.0 * params.bullet_speed_multiplier,
            },
            movement: BossMovement::Teleport {
                waypoints: vec![
                    Vec2::new(-100.0, 80.0),
                    Vec2::new(100.0, 80.0),
                    Vec2::new(0.0, 120.0),
                ],
                wait_timer: Timer::from_seconds(1.5, TimerMode::Repeating),
                current: 0,
            },
            spell_card_bonus: 1_300_000,
        },
        // SC5: 咲夜特製ストップウォッチ
        BossPhaseData {
            hp: 2000.0 * params.enemy_hp_multiplier,
            hp_max: 2000.0 * params.enemy_hp_multiplier,
            is_spell_card: true,
            spell_card_name: Some("時符「咲夜特製ストップウォッチ」".to_string()),
            time_limit_secs: 60.0,
            pattern: BulletPattern::Spiral {
                arms: (4.0 * params.bullet_count_multiplier) as u8,
                speed: 130.0 * params.bullet_speed_multiplier,
                rotation_speed_deg: 90.0,
            },
            movement: BossMovement::Circle { radius: 50.0, speed_deg: 40.0, center: Vec2::new(0.0, 80.0) },
            spell_card_bonus: 2_000_000,
        },
    ]
}
```

---

## 参照

- `docs/specification.md` § Stage 4, Stage 5
- `docs/03_danmaku_systems.md` § 弾幕パターン
