# Phase 16: Stage 6 実装（レミリア・スカーレット）

## 目標

ラスボス・レミリアが完全実装され、エンディングまで通しプレイできる。

## 完了条件

- [ ] レミリア全スペルカード実装（8種）
- [ ] Stage 6 ザコ波スクリプト
- [ ] エンディングシーケンス（暫定）
- [ ] スタッフロール（暫定）
- [ ] 全難易度でのエンディング確認

---

## タスク詳細

### Stage 6: レミリア・スカーレット

レミリアは「運命操作」と「翼」をテーマにした弾幕を使う。難易度が上がると弾が「運命の流れ」のように曲がる演出が加わる（Phase 18 で実装）。

**スペルカード一覧**

| フェーズ | 名前 | 説明 |
|---|---|---|
| 通常1 | — | 吸血鬼の翼を模した扇形弾幕 |
| SC1 | 「紅符「スカーレットシュート」」 | 紅い弾が画面を覆う |
| 通常2 | — | 自機追尾高速弾 |
| SC2 | 「紅蝙蝠「ヴァンパイアスウォーム」」 | コウモリ形の大弾が乱舞 |
| 通常3 | — | 超速のナイフ弾 |
| SC3 | 「運命「スカーレットデスティニー」」 | 弾が運命に従い曲がる |
| SC4 | 「神鬼「レミリアストレッチ」」 | 巨大な翼を広げて弾を放つ |
| SC5 | 「紅符「ブラッドスウォード」」 | 血剣形の弾幕 |
| SC6 | 「天罰「サンシャインニードル」」 | 十字の光線 |
| SC7 | 「紅魔「スカーレットデビル」」 | 最終前スペルカード |
| SC8 | 「夜符「デーモンキングクレイドル」」 | 最終スペルカード（超高難度） |

```rust
// app/core/src/systems/boss/bosses/remilia.rs

pub fn remilia_phases(difficulty: Difficulty) -> Vec<BossPhaseData> {
    let params = DifficultyParams::for_difficulty(difficulty);

    vec![
        // 通常1: 翼形扇形弾
        BossPhaseData {
            hp: 1500.0 * params.enemy_hp_multiplier,
            hp_max: 1500.0 * params.enemy_hp_multiplier,
            is_spell_card: false,
            spell_card_name: None,
            time_limit_secs: 40.0,
            pattern: BulletPattern::Spread {
                count: (7.0 * params.bullet_count_multiplier) as u8,
                spread_deg: 80.0,
                speed: 130.0 * params.bullet_speed_multiplier,
                angle_offset: 0.0,
            },
            movement: BossMovement::Pendulum { amplitude: 80.0, frequency: 0.5, base_x: 0.0 },
            spell_card_bonus: 0,
        },
        // SC1: スカーレットシュート
        BossPhaseData {
            hp: 1800.0 * params.enemy_hp_multiplier,
            hp_max: 1800.0 * params.enemy_hp_multiplier,
            is_spell_card: true,
            spell_card_name: Some("紅符「スカーレットシュート」".to_string()),
            time_limit_secs: 50.0,
            pattern: BulletPattern::Spiral {
                arms: (3.0 * params.bullet_count_multiplier) as u8,
                speed: 120.0 * params.bullet_speed_multiplier,
                rotation_speed_deg: 80.0,
            },
            movement: BossMovement::Circle { radius: 60.0, speed_deg: 30.0, center: Vec2::new(0.0, 80.0) },
            spell_card_bonus: 1_500_000,
        },
        // SC2: ヴァンパイアスウォーム
        BossPhaseData {
            hp: 2000.0 * params.enemy_hp_multiplier,
            hp_max: 2000.0 * params.enemy_hp_multiplier,
            is_spell_card: true,
            spell_card_name: Some("紅蝙蝠「ヴァンパイアスウォーム」".to_string()),
            time_limit_secs: 55.0,
            pattern: BulletPattern::Ring {
                count: (20.0 * params.bullet_count_multiplier) as u8,
                speed: 100.0 * params.bullet_speed_multiplier,
            },
            movement: BossMovement::Pendulum { amplitude: 100.0, frequency: 0.6, base_x: 0.0 },
            spell_card_bonus: 1_800_000,
        },
        // SC7: スカーレットデビル
        BossPhaseData {
            hp: 2500.0 * params.enemy_hp_multiplier,
            hp_max: 2500.0 * params.enemy_hp_multiplier,
            is_spell_card: true,
            spell_card_name: Some("紅魔「スカーレットデビル」".to_string()),
            time_limit_secs: 60.0,
            pattern: BulletPattern::Spiral {
                arms: (5.0 * params.bullet_count_multiplier) as u8,
                speed: 150.0 * params.bullet_speed_multiplier,
                rotation_speed_deg: 120.0,
            },
            movement: BossMovement::Circle { radius: 80.0, speed_deg: 50.0, center: Vec2::new(0.0, 100.0) },
            spell_card_bonus: 3_000_000,
        },
        // SC8: デーモンキングクレイドル（最終）
        BossPhaseData {
            hp: 3000.0 * params.enemy_hp_multiplier,
            hp_max: 3000.0 * params.enemy_hp_multiplier,
            is_spell_card: true,
            spell_card_name: Some("夜符「デーモンキングクレイドル」".to_string()),
            time_limit_secs: 80.0,
            pattern: BulletPattern::Ring {
                count: (32.0 * params.bullet_count_multiplier) as u8,
                speed: 160.0 * params.bullet_speed_multiplier,
            },
            movement: BossMovement::Pendulum { amplitude: 120.0, frequency: 0.7, base_x: 0.0 },
            spell_card_bonus: 5_000_000,
        },
    ]
}
```

### Stage 6 スクリプト

```rust
// app/core/src/systems/stage/stage6.rs

pub fn stage6_script() -> Vec<SpawnEntry> {
    // 紅魔館の最深部
    // 吸血鬼の眷属が多数
    // 強力なコウモリ型敵
    vec![
        // 実装時に詳細追加
    ]
}
```

### エンディング（暫定）

```rust
// app/ui/src/screens/ending.rs

pub fn setup_ending(mut commands: Commands, assets: Res<ScarletAssets>) {
    commands.spawn((
        DespawnOnExit(AppState::Ending),
        Node { .. },
    )).with_children(|parent| {
        parent.spawn(Text::new("おめでとうございます！\n東方紅魔郷をクリアしました。\n\n（エンディングは今後実装予定）"));
    });
}

pub fn ending_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::KeyZ) || keys.just_pressed(KeyCode::Return) {
        next_state.set(AppState::StaffRoll);
    }
}
```

### スタッフロール（暫定）

```rust
// app/ui/src/screens/ending.rs に追加

pub fn setup_staff_roll(mut commands: Commands) {
    commands.spawn((
        DespawnOnExit(AppState::StaffRoll),
        Text::new("開発: touhou-project-scarlet\nエンジン: Bevy 0.17.3\n\nBased on 東方紅魔郷\n© 上海アリス幻樂団"),
    ));
}

pub fn staff_roll_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::KeyZ) || keys.just_pressed(KeyCode::Return) {
        next_state.set(AppState::Title);
    }
}
```

### ハイスコア永続化

クリア時にハイスコアをJSONで保存:

```rust
// app/core/src/systems/score.rs

const HISCORE_PATH: &str = "hiscore.json";

pub fn save_hiscore(game_data: Res<GameData>) {
    let hi = game_data.score.max(game_data.hi_score);
    let json = serde_json::to_string(&hi).unwrap_or_default();
    let path = std::path::Path::new(HISCORE_PATH);
    let _ = std::fs::write(path, json);
}

pub fn load_hiscore(mut game_data: ResMut<GameData>) {
    let path = std::path::Path::new(HISCORE_PATH);
    if let Ok(json) = std::fs::read_to_string(path) {
        if let Ok(hi) = serde_json::from_str::<u64>(&json) {
            game_data.hi_score = hi;
        }
    }
}
```

---

## 参照

- `docs/01_specification.md` § Stage 6 / レミリア
- `docs/references/vampire-survivors/app/core/src/persistence.rs` — save/load JSON パターン
