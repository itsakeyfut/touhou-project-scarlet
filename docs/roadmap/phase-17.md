# Phase 17: Extraステージ実装（フランドール・スカーレット）

## 目標

Extraステージが解放・プレイ可能になる。フランドールの全スペルカードが実装される。

## 完了条件

- [ ] Extraステージ解放条件の実装
- [ ] Extra雑魚敵スクリプト
- [ ] フランドール全スペルカード実装（8種）
- [ ] Extra BGM統合
- [ ] Extra クリアエンディング（暫定）

---

## タスク詳細

### 1. Extraステージ解放条件

いずれかの難易度でノーマルクリアすると解放。

```rust
// app/core/src/resources/game_data.rs に追加

#[derive(Resource, serde::Serialize, serde::Deserialize, Default)]
pub struct ProgressData {
    pub cleared_difficulties: std::collections::HashSet<String>, // "Normal", "Hard" etc.
    pub extra_unlocked: bool,
    pub hi_score: u64,
}

pub fn check_extra_unlock(
    game_data: Res<GameData>,
    mut progress: ResMut<ProgressData>,
    selected: Res<SelectedDifficulty>,
) {
    let diff_name = format!("{:?}", selected.difficulty);
    progress.cleared_difficulties.insert(diff_name);
    if !progress.extra_unlocked && !progress.cleared_difficulties.is_empty() {
        progress.extra_unlocked = true;
    }
}
```

### 2. タイトル画面のExtra項目

```rust
// app/ui/src/screens/title.rs

pub fn title_input(
    keys: Res<ButtonInput<KeyCode>>,
    progress: Res<ProgressData>,
    mut next_state: ResMut<NextState<AppState>>,
    mut selected: Local<usize>,
    mut app_exit: EventWriter<AppExit>,
) {
    let menu_items = if progress.extra_unlocked { 3 } else { 2 }; // Extra 項目の有効化

    if keys.just_pressed(KeyCode::KeyZ) {
        match *selected {
            0 => next_state.set(AppState::CharacterSelect),
            1 if progress.extra_unlocked => {
                // Extra モード: 難易度選択なしで直接ロードへ
                next_state.set(AppState::Loading);
            }
            _ => app_exit.write(AppExit::Success),
        }
    }
}
```

### 3. Extra ステージ識別

```rust
// StageData に extra フィールドを追加

#[derive(Resource)]
pub struct StageData {
    pub current_stage: u8,
    pub stage_elapsed: f32,
    pub boss_spawned: bool,
    pub boss_defeated: bool,
    pub is_extra: bool,  // 追加
}
```

### 4. フランドール・スカーレット

フランドールは「すべてを壊す力」をテーマにした超高密度の弾幕を使う。弾の量・速度ともにレミリアを大幅に上回る。

**スペルカード一覧**

| フェーズ | 名前 | 説明 |
|---|---|---|
| 通常1 | — | 全方位拡散弾 + 自機狙い |
| SC1 | 「禁忌「フォービドゥンフルーツ」」 | リンゴ形の弾幕 |
| 通常2 | — | 四方向高速弾幕 |
| SC2 | 「禁忌「レーヴァテイン」」 | 炎の剣弾幕（レーザー的） |
| 通常3 | — | 超高速螺旋 |
| SC3 | 「禁忌「カゴメカゴメ」」 | 籠目紋様の弾幕 |
| SC4 | 「禁忌「ロンリーウェアウルフ」」 | 狼の遠吠えを模した全方位 |
| SC5 | 「秘弾「そして誰もいなくなるか？」」 | 時間差で全画面を埋める |
| SC6 | 「禁忌「恋の迷路」」 | 回転する壁弾幕 |
| SC7 | 「超弦「スターボウブレイク」」 | 星型超高密度弾幕 |
| SC8 | 「「おーぷんざゲート」」 | 最終スペルカード |

```rust
// app/core/src/systems/boss/bosses/flandre.rs

pub fn flandre_phases(difficulty: Difficulty) -> Vec<BossPhaseData> {
    // Extra は固定難易度（最高難度相当）
    let params = DifficultyParams::for_difficulty(Difficulty::Extra);

    vec![
        // 通常1: 全方位 + 自機狙い複合
        BossPhaseData {
            hp: 2000.0,
            hp_max: 2000.0,
            is_spell_card: false,
            spell_card_name: None,
            time_limit_secs: 40.0,
            pattern: BulletPattern::Ring { count: 16, speed: 140.0 },
            movement: BossMovement::Pendulum { amplitude: 100.0, frequency: 0.6, base_x: 0.0 },
            spell_card_bonus: 0,
        },
        // SC1: フォービドゥンフルーツ
        BossPhaseData {
            hp: 2500.0,
            hp_max: 2500.0,
            is_spell_card: true,
            spell_card_name: Some("禁忌「フォービドゥンフルーツ」".to_string()),
            time_limit_secs: 50.0,
            pattern: BulletPattern::Spiral {
                arms: 5,
                speed: 130.0,
                rotation_speed_deg: 100.0,
            },
            movement: BossMovement::Circle { radius: 70.0, speed_deg: 45.0, center: Vec2::new(0.0, 80.0) },
            spell_card_bonus: 2_000_000,
        },
        // SC2: レーヴァテイン（炎剣 = 太いレーザー的演出）
        BossPhaseData {
            hp: 2800.0,
            hp_max: 2800.0,
            is_spell_card: true,
            spell_card_name: Some("禁忌「レーヴァテイン」".to_string()),
            time_limit_secs: 55.0,
            pattern: BulletPattern::Spread {
                count: 12,
                spread_deg: 60.0,
                speed: 160.0,
                angle_offset: 0.0,
            },
            movement: BossMovement::Teleport {
                waypoints: vec![
                    Vec2::new(-120.0, 60.0),
                    Vec2::new(120.0, 60.0),
                    Vec2::new(0.0, 120.0),
                    Vec2::new(-60.0, 40.0),
                    Vec2::new(60.0, 40.0),
                ],
                wait_timer: Timer::from_seconds(2.0, TimerMode::Repeating),
                current: 0,
            },
            spell_card_bonus: 2_500_000,
        },
        // SC7: スターボウブレイク
        BossPhaseData {
            hp: 4000.0,
            hp_max: 4000.0,
            is_spell_card: true,
            spell_card_name: Some("超弦「スターボウブレイク」".to_string()),
            time_limit_secs: 70.0,
            pattern: BulletPattern::Spiral {
                arms: 8,
                speed: 170.0,
                rotation_speed_deg: 150.0,
            },
            movement: BossMovement::Circle { radius: 90.0, speed_deg: 60.0, center: Vec2::new(0.0, 90.0) },
            spell_card_bonus: 5_000_000,
        },
        // SC8: おーぷんざゲート（最終）
        BossPhaseData {
            hp: 5000.0,
            hp_max: 5000.0,
            is_spell_card: true,
            spell_card_name: Some("「おーぷんざゲート」".to_string()),
            time_limit_secs: 90.0,
            pattern: BulletPattern::Ring { count: 40, speed: 180.0 },
            movement: BossMovement::Pendulum { amplitude: 130.0, frequency: 0.8, base_x: 0.0 },
            spell_card_bonus: 10_000_000,
        },
    ]
}
```

### 5. Extra ステージスクリプト

```rust
// app/core/src/systems/stage/stage_extra.rs

pub fn extra_script() -> Vec<SpawnEntry> {
    // 通常ステージより強力な雑魚
    // レミリアの眷属（強化版）
    // 長めのザコ波（ボス前10分程度）
    vec![
        // 詳細は実装時
    ]
}
```

### 6. Extra クリアエンディング

```rust
// Extra クリア時は別エンディング（フランドールとの会話）
pub fn setup_extra_ending(mut commands: Commands) {
    commands.spawn((
        DespawnOnExit(AppState::Ending),
        Text::new("エクストラステージクリア！\nおめでとうございます！\n\n（フランドールエンディングは今後実装予定）"),
    ));
}
```

---

## 参照

- `docs/specification.md` § Extraステージ / フランドール
