# Phase 14: Stage 2〜3 実装（チルノ・紅美鈴）

## 目標

Stage 2（チルノ）と Stage 3（紅美鈴）が通しプレイできる。

## 完了条件

- [ ] チルノ全スペルカード実装（4種）
- [ ] 紅美鈴全スペルカード実装（5種）
- [ ] Stage 2〜3 ザコ波スクリプト
- [ ] 各ボスBGM統合

---

## タスク詳細

### Stage 2: チルノ

**スペルカード一覧**

| フェーズ | 名前 | 説明 |
|---|---|---|
| 通常 | — | 自機狙い + 拡散氷弾 |
| SC1 | 「氷符「アイシクルフォール」」 | 上から大量の氷の欠片が降り注ぐ |
| 通常2 | — | 高速氷弾の螺旋 |
| SC2 | 「冷符「コールドスナップ」」 | 八方向に大きな氷弾 |
| SC3 | 「凍符「パーフェクトフリーズ」」 | 弾が一時的に止まった後全方向へ |
| SC4 | 「雪符「ダイアモンドブリザード」」 (Hard以上) | 超高密度の氷弾嵐 |

```rust
// app/core/src/systems/boss/bosses/cirno.rs

pub fn cirno_phases(difficulty: Difficulty) -> Vec<BossPhaseData> {
    let params = DifficultyParams::for_difficulty(difficulty);
    let mut phases = vec![
        // 通常攻撃: 自機狙い3-way + 氷弾
        BossPhaseData {
            hp: 700.0 * params.enemy_hp_multiplier,
            hp_max: 700.0 * params.enemy_hp_multiplier,
            is_spell_card: false,
            spell_card_name: None,
            time_limit_secs: 30.0,
            pattern: BulletPattern::Aimed { count: 3, spread_deg: 20.0, speed: 110.0 * params.bullet_speed_multiplier },
            movement: BossMovement::Pendulum { amplitude: 80.0, frequency: 0.5, base_x: 0.0 },
            spell_card_bonus: 0,
        },
        // SC1: アイシクルフォール - 画面上部から氷弾が降る
        BossPhaseData {
            hp: 900.0 * params.enemy_hp_multiplier,
            hp_max: 900.0 * params.enemy_hp_multiplier,
            is_spell_card: true,
            spell_card_name: Some("氷符「アイシクルフォール」".to_string()),
            time_limit_secs: 40.0,
            pattern: BulletPattern::Spread {
                count: (8.0 * params.bullet_count_multiplier) as u8,
                spread_deg: 160.0,
                speed: 80.0 * params.bullet_speed_multiplier,
                angle_offset: -90.0, // 下方向
            },
            movement: BossMovement::Pendulum { amplitude: 60.0, frequency: 0.4, base_x: 0.0 },
            spell_card_bonus: 1_000_000,
        },
        // SC2: コールドスナップ - 8方向大弾
        BossPhaseData {
            hp: 1000.0 * params.enemy_hp_multiplier,
            hp_max: 1000.0 * params.enemy_hp_multiplier,
            is_spell_card: true,
            spell_card_name: Some("冷符「コールドスナップ」".to_string()),
            time_limit_secs: 45.0,
            pattern: BulletPattern::Ring {
                count: 8,
                speed: 70.0 * params.bullet_speed_multiplier,
            },
            movement: BossMovement::Static,
            spell_card_bonus: 1_100_000,
        },
        // SC3: パーフェクトフリーズ（将来: 弾停止メカニクス）
        BossPhaseData {
            hp: 1200.0 * params.enemy_hp_multiplier,
            hp_max: 1200.0 * params.enemy_hp_multiplier,
            is_spell_card: true,
            spell_card_name: Some("凍符「パーフェクトフリーズ」".to_string()),
            time_limit_secs: 50.0,
            pattern: BulletPattern::Ring {
                count: (12.0 * params.bullet_count_multiplier) as u8,
                speed: 60.0 * params.bullet_speed_multiplier,
            },
            movement: BossMovement::Circle { radius: 50.0, speed_deg: 25.0, center: Vec2::new(0.0, 60.0) },
            spell_card_bonus: 1_200_000,
        },
    ];

    // Hard以上: ダイアモンドブリザード追加
    if matches!(difficulty, Difficulty::Hard | Difficulty::Lunatic) {
        phases.push(BossPhaseData {
            hp: 1500.0 * params.enemy_hp_multiplier,
            hp_max: 1500.0 * params.enemy_hp_multiplier,
            is_spell_card: true,
            spell_card_name: Some("雪符「ダイアモンドブリザード」".to_string()),
            time_limit_secs: 60.0,
            pattern: BulletPattern::Spiral {
                arms: (3.0 * params.bullet_count_multiplier) as u8,
                speed: 120.0 * params.bullet_speed_multiplier,
                rotation_speed_deg: 120.0,
            },
            movement: BossMovement::Pendulum { amplitude: 100.0, frequency: 0.7, base_x: 0.0 },
            spell_card_bonus: 1_500_000,
        });
    }

    phases
}
```

### Stage 2 スクリプト（概要）

```rust
// app/core/src/systems/stage/stage2.rs

pub fn stage2_script() -> Vec<SpawnEntry> {
    let mut script = vec![
        // 氷の妖精が大量に出現
        // コウモリの群れ
        // 大型妖精（HP高め）
    ];
    // 詳細は実装時に拡充
    script
}
```

---

### Stage 3: 紅美鈴

**スペルカード一覧**

| フェーズ | 名前 | 説明 |
|---|---|---|
| 通常1 | — | 五色の玉（扇形） |
| SC1 | 「彩符「彩雨」」 | 五色の玉が上から降る |
| 通常2 | — | 格闘系（当たり判定近い） |
| SC2 | 「闘符「彩光乱舞」」 | 高速の小弾多発 |
| SC3 | 「虹符「彩光乱舞」」 (Hard) | さらに高密度版 |
| SC4 | 「気符「星脈地転」」 | 星形弾幕 |
| SC5 | 「彩符「五行射」」 | 五方向に異なる色の弾 |

```rust
// app/core/src/systems/boss/bosses/meiling.rs

pub fn meiling_phases(difficulty: Difficulty) -> Vec<BossPhaseData> {
    let params = DifficultyParams::for_difficulty(difficulty);

    vec![
        BossPhaseData {
            hp: 800.0 * params.enemy_hp_multiplier,
            hp_max: 800.0 * params.enemy_hp_multiplier,
            is_spell_card: false,
            spell_card_name: None,
            time_limit_secs: 30.0,
            pattern: BulletPattern::Spread {
                count: 5,
                spread_deg: 60.0,
                speed: 120.0 * params.bullet_speed_multiplier,
                angle_offset: 0.0,
            },
            movement: BossMovement::Pendulum { amplitude: 70.0, frequency: 0.45, base_x: 0.0 },
            spell_card_bonus: 0,
        },
        BossPhaseData {
            hp: 1000.0 * params.enemy_hp_multiplier,
            hp_max: 1000.0 * params.enemy_hp_multiplier,
            is_spell_card: true,
            spell_card_name: Some("彩符「彩雨」".to_string()),
            time_limit_secs: 40.0,
            pattern: BulletPattern::Spread {
                count: (6.0 * params.bullet_count_multiplier) as u8,
                spread_deg: 120.0,
                speed: 100.0 * params.bullet_speed_multiplier,
                angle_offset: -90.0,
            },
            movement: BossMovement::Pendulum { amplitude: 90.0, frequency: 0.5, base_x: 0.0 },
            spell_card_bonus: 1_000_000,
        },
        BossPhaseData {
            hp: 1200.0 * params.enemy_hp_multiplier,
            hp_max: 1200.0 * params.enemy_hp_multiplier,
            is_spell_card: true,
            spell_card_name: Some("気符「星脈地転」".to_string()),
            time_limit_secs: 50.0,
            pattern: BulletPattern::Ring {
                count: (10.0 * params.bullet_count_multiplier) as u8,
                speed: 100.0 * params.bullet_speed_multiplier,
            },
            movement: BossMovement::Circle { radius: 70.0, speed_deg: 35.0, center: Vec2::new(0.0, 70.0) },
            spell_card_bonus: 1_200_000,
        },
        // 残りのスペルカードは実装時に追加
    ]
}
```

### Stage 3 スクリプト（概要）

```rust
// app/core/src/systems/stage/stage3.rs

pub fn stage3_script() -> Vec<SpawnEntry> {
    // 紅魔館の門番関連の敵
    // 妖精と門番見習い
    vec![]
}
```

---

## Stage 2〜3 への遷移

```rust
// stage_control_system で next_stage を処理
// OnEnter(AppState::Playing) で stage_data.current_stage に応じてスクリプト選択

pub fn setup_stage(
    mut spawner: ResMut<EnemySpawner>,
    stage_data: Res<StageData>,
) {
    spawner.script = match stage_data.current_stage {
        1 => stage1::stage1_script(),
        2 => stage2::stage2_script(),
        3 => stage3::stage3_script(),
        _ => vec![],
    };
    spawner.index = 0;
    spawner.elapsed = 0.0;
}

/// ステージクリア後にボスをスポーン
pub fn spawn_boss_for_stage(
    commands: &mut Commands,
    stage: u8,
    difficulty: Difficulty,
) {
    match stage {
        1 => rumia::spawn_rumia(commands, difficulty),
        2 => cirno::spawn_cirno(commands, difficulty),
        3 => meiling::spawn_meiling(commands, difficulty),
        _ => {}
    }
}
```

---

## 参照

- `docs/specification.md` § Stage 2, Stage 3
- `docs/03_danmaku_systems.md` § 5 (ボスシステム)
