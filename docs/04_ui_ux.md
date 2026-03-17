# 04 UI/UX デザイン

## 概要

東方紅魔郷クローンにおけるUIシステムの設計ドキュメント。HUD、各種画面、メニューの実装方針を定義する。

---

## 1. 画面レイアウト

### 1.1 ウィンドウ解像度

```
ウィンドウ全体: 640 × 480 px (スケール可能)
プレイエリア:   384 × 448 px (左側)
サイドパネル:   256 × 448 px (右側)
上部余白:          0 ×  16 px
下部余白:          0 ×  16 px
```

```
┌─────────────────────────────────────────────────────────┐
│              (上部余白 640×16)                           │
├─────────────────────────────┬───────────────────────────┤
│                             │  ハイスコア: 1,234,567,890 │
│      プレイエリア           │  スコア:     0,123,456,789 │
│      384 × 448 px           │                            │
│                             │  残機: ♥ ♥ ♥              │
│  (弾幕・プレイヤー・敵が   │  ボム: ★ ★ ★             │
│   表示される領域)           │                            │
│                             │  パワー: 128/128           │
│                             │  グレイズ: 0               │
│                             │                            │
│                             │  ──────────────────        │
│                             │  [キャラクター画像]        │
│                             │  霊夢                      │
│                             │                            │
│                             │  [操作説明]                │
│                             │  Z: ショット               │
│                             │  X: ボム                   │
│                             │  Shift: 低速移動           │
├─────────────────────────────┴───────────────────────────┤
│              (下部余白 640×16)                           │
└─────────────────────────────────────────────────────────┘
```

### 1.2 座標系

- Bevy: Y軸上向き正 → プレイエリア中心を (0, 0)
- プレイエリア: X = -192〜192, Y = -224〜224
- サイドパネル: X = 192〜448 (Bevy座標 = +192〜+448から-320オフセット後)

---

## 2. HUD (ゲームプレイ中)

### 2.1 HUDコンポーネント

```rust
// HUDルートエンティティのマーカー
#[derive(Component)] pub struct HudRoot;
#[derive(Component)] pub struct HudScore;
#[derive(Component)] pub struct HudHiScore;
#[derive(Component)] pub struct HudLives;
#[derive(Component)] pub struct HudBombs;
#[derive(Component)] pub struct HudPower;
#[derive(Component)] pub struct HudGraze;
#[derive(Component)] pub struct BossHealthBar;
#[derive(Component)] pub struct BossTimer;
#[derive(Component)] pub struct BossName;
#[derive(Component)] pub struct SpellCardName;
```

### 2.2 スコア表示

```rust
pub fn update_hud_score(
    game_data: Res<GameData>,
    mut score_text: Query<&mut Text, With<HudScore>>,
    mut hiscore_text: Query<&mut Text, (With<HudHiScore>, Without<HudScore>)>,
) {
    if game_data.is_changed() {
        if let Ok(mut text) = score_text.single_mut() {
            **text = format!("{:>12}", game_data.score);
        }
        if let Ok(mut text) = hiscore_text.single_mut() {
            let hi = game_data.score.max(game_data.hi_score);
            **text = format!("{:>12}", hi);
        }
    }
}
```

### 2.3 残機・ボム表示

アイコン方式：残機は霊夢/魔理沙のミニアイコン、ボムは星アイコンを残数分表示。

```rust
pub fn spawn_hud_lives(
    parent: &mut ChildBuilder,
    assets: &ScarletAssets,
    lives: u8,
) {
    parent.spawn(Node {
        flex_direction: FlexDirection::Row,
        ..default()
    }).with_children(|row| {
        for _ in 0..lives {
            row.spawn(ImageNode {
                image: assets.ui_life_icon.clone(),
                ..default()
            });
        }
    });
}
```

### 2.4 パワーゲージ

数値とバーの両方で表示。

```
パワー: [████████████░░░] 96/128
```

```rust
pub fn update_hud_power(
    game_data: Res<GameData>,
    mut power_bar: Query<&mut Node, With<HudPowerBar>>,
    mut power_text: Query<&mut Text, With<HudPower>>,
) {
    if game_data.is_changed() {
        let ratio = game_data.power as f32 / 128.0;
        if let Ok(mut style) = power_bar.single_mut() {
            style.width = Val::Percent(ratio * 100.0);
        }
        if let Ok(mut text) = power_text.single_mut() {
            **text = format!("{}/128", game_data.power);
        }
    }
}
```

### 2.5 ボスHPバー

ボス出現時に画面上部（プレイエリア上端）に表示するHPバー。フェーズ数だけセグメントに分割される。

```rust
pub fn spawn_boss_hp_bar(
    mut commands: Commands,
    boss_events: EventReader<BossPhaseChangedEvent>,
    bosses: Query<&Boss>,
) {
    // ボスエンティティのフェーズ数でセグメント数を決定
}

pub fn update_boss_hp_bar(
    mut hp_bars: Query<&mut Node, With<BossHealthBar>>,
    bosses: Query<(&Boss, &BossPhaseData)>,
) {
    for (boss, phase_data) in &bosses {
        let ratio = phase_data.hp / phase_data.hp_max;
        // バーの幅をフェーズHP割合に合わせて更新
    }
}
```

### 2.6 ボスタイマー

スペルカード中は残り時間をカウントダウン表示（赤色で強調）。

```rust
pub fn update_boss_timer(
    mut timer_text: Query<(&mut Text, &mut TextColor), With<BossTimer>>,
    bosses: Query<&Boss>,
    time: Res<Time>,
) {
    for boss in &bosses {
        let remaining = boss.phase_timer.remaining_secs();
        if let Ok((mut text, mut color)) = timer_text.single_mut() {
            **text = format!("{:.0}", remaining);
            color.0 = if remaining < 10.0 {
                Color::srgb(1.0, 0.2, 0.2)
            } else {
                Color::WHITE
            };
        }
    }
}
```

### 2.7 スペルカード名表示

```rust
pub fn show_spell_card_name(
    mut commands: Commands,
    mut spell_name: Query<(&mut Text, &mut Visibility), With<SpellCardName>>,
    mut phase_events: EventReader<BossPhaseChangedEvent>,
    bosses: Query<&Boss>,
) {
    for event in phase_events.read() {
        let Ok(boss) = bosses.get(event.entity) else { continue };
        let phase = &boss.phases[event.phase];

        if let Some(name) = &phase.spell_card_name {
            if let Ok((mut text, mut vis)) = spell_name.single_mut() {
                **text = name.clone();
                *vis = Visibility::Visible;
            }
        }
    }
}
```

---

## 3. タイトル画面

### 3.1 レイアウト

```
┌────────────────────────────────────────┐
│                                        │
│     東方紅魔郷                          │
│   ~ the Embodiment of Scarlet Devil ~  │
│                                        │
│         [ゲームスタート]               │
│         [エクストラスタート]           │
│         [終了]                         │
│                                        │
│          (キャラクターイラスト)        │
└────────────────────────────────────────┘
```

### 3.2 実装

```rust
#[derive(Component)] pub struct TitleScreen;
#[derive(Component)] pub struct TitleMenuItem;

pub fn setup_title_screen(
    mut commands: Commands,
    assets: Res<ScarletAssets>,
) {
    commands.spawn((
        TitleScreen,
        DespawnOnExit(AppState::Title),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
    )).with_children(|parent| {
        // タイトルロゴ
        parent.spawn((
            ImageNode { image: assets.title_logo.clone(), ..default() },
        ));

        // メニュー項目
        for (label, action) in [
            ("ゲームスタート", TitleAction::Start),
            ("エクストラスタート", TitleAction::Extra),
            ("終了", TitleAction::Quit),
        ] {
            parent.spawn((
                TitleMenuItem,
                Button,
                Text::new(label),
                // styles...
            )).observe(on_title_button_click(action));
        }
    });
}
```

---

## 4. キャラクター選択画面

### 4.1 レイアウト

```
┌────────────────────────────────────────────────────────┐
│  キャラクター選択                                       │
│                                                        │
│  ┌────────────────┐   ┌────────────────┐              │
│  │                │   │                │              │
│  │  [霊夢画像]   │   │  [魔理沙画像] │              │
│  │                │   │                │              │
│  │  博麗 霊夢     │   │  霧雨 魔理沙   │              │
│  │                │   │                │              │
│  │  ショット A:   │   │  ショット A:   │              │
│  │  霊符          │   │  魔符          │              │
│  │  ショット B:   │   │  ショット B:   │              │
│  │  夢符          │   │  恋符          │              │
│  └────────────────┘   └────────────────┘              │
│                                                        │
│  ← → で選択   Z で決定                                 │
└────────────────────────────────────────────────────────┘
```

### 4.2 実装

```rust
#[derive(Component)] pub struct CharacterSelectScreen;
#[derive(Component)] pub struct CharacterCard(pub CharacterType);
#[derive(Component)] pub struct CharacterCardSelected;

pub fn character_select_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut selected: ResMut<SelectedCharacter>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::KeyA) {
        // 前のキャラクターへ
    }
    if keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::KeyD) {
        // 次のキャラクターへ
    }
    if keys.just_pressed(KeyCode::KeyZ) || keys.just_pressed(KeyCode::Return) {
        next_state.set(AppState::DifficultySelect);
    }
}
```

---

## 5. 難易度選択画面

### 5.1 レイアウト

```
┌──────────────────────────────────────┐
│  難易度選択                          │
│                                      │
│  ● Easy   (弾が少ない、ゆっくり)   │
│  ● Normal (標準)                    │
│  ● Hard   (弾が速く多い)           │
│  ● Lunatic (最高難易度)             │
│                                      │
│  ← → で選択   Z で決定             │
└──────────────────────────────────────┘
```

---

## 6. ポーズ画面

### 6.1 ポーズメニュー

ESCキーでポーズ。ゲームの更新を止め（`TimeScale = 0`相当）、半透明オーバーレイを表示。

```rust
#[derive(Component)] pub struct PauseMenu;

pub fn setup_pause_menu(mut commands: Commands) {
    commands.spawn((
        PauseMenu,
        DespawnOnExit(AppState::Paused),
        // 半透明黒背景
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        // 最前面に表示
        GlobalZIndex(100),
    )).with_children(|parent| {
        // ポーズメニュー本体
        parent.spawn(Node {
            flex_direction: FlexDirection::Column,
            ..default()
        }).with_children(|col| {
            for label in ["ゲームに戻る", "タイトルへ"] {
                col.spawn((Button, Text::new(label)));
            }
        });
    });
}
```

---

## 7. ゲームオーバー画面

### 7.1 レイアウト

```
┌──────────────────────────────────────┐
│                                      │
│         GAME OVER                    │
│                                      │
│  スコア:  1,234,567                  │
│  グレイズ: 42                        │
│                                      │
│  [コンティニュー (残り3回)]         │
│  [タイトルへ]                        │
│                                      │
└──────────────────────────────────────┘
```

コンティニューは3回まで可能（ノーコンボーナス消滅）。

```rust
#[derive(Resource, Default)]
pub struct ContinueData {
    pub continues_used: u8,
    pub max_continues: u8, // 3
}
```

---

## 8. ステージクリア・エンディング画面

### 8.1 ステージクリア

各ステージクリア後に表示。

```
┌──────────────────────────────────────┐
│  STAGE 1 CLEAR                       │
│                                      │
│  ステージスコア: 234,567             │
│  グレイズボーナス: 21,000            │
│  残機ボーナス: 150,000               │
│  ボムボーナス: 15,000                │
│                                      │
│  合計: 420,567                       │
└──────────────────────────────────────┘
```

### 8.2 クリア後ダイアログ（ボス撃破時）

ボス撃破後にストーリーダイアログを表示する（省略可）。

```rust
#[derive(Component)] pub struct DialogBox;
#[derive(Component)] pub struct DialogText;
#[derive(Component)] pub struct DialogPortrait;

pub struct DialogLine {
    pub speaker: String,
    pub text: String,
    pub portrait: Handle<Image>,
}
```

---

## 9. エフェクトUI

### 9.1 エクステンド表示

残機が増えた際に画面中央にフラッシュ表示。

```rust
pub fn spawn_extend_popup(
    mut commands: Commands,
    mut extend_events: EventReader<ExtendEvent>,
    assets: Res<ScarletAssets>,
) {
    for _ in extend_events.read() {
        commands.spawn((
            Text::new("EXTEND!"),
            TextColor(Color::srgb(1.0, 0.9, 0.0)),
            TextFont { font_size: 48.0, ..default() },
            // フェードアウトアニメーション
            FadeOut { timer: Timer::from_seconds(2.0, TimerMode::Once) },
        ));
    }
}
```

### 9.2 スコアポップアップ

アイテム取得・敵撃破時にスコアがポップアップ表示される（オプション）。

```rust
#[derive(Component)]
pub struct ScorePopup {
    pub value: u32,
    pub timer: Timer,
    pub velocity: Vec2, // 上方向に浮かぶ
}
```

### 9.3 ボム演出（フラッシュ）

ボム使用時に画面全体を一瞬白くフラッシュ。

```rust
pub fn bomb_flash_effect(
    mut commands: Commands,
    mut bomb_events: EventReader<BombUsedEvent>,
) {
    for _ in bomb_events.read() {
        commands.spawn((
            // 全画面白いオーバーレイ
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            BackgroundColor(Color::WHITE),
            GlobalZIndex(99),
            FadeOut { timer: Timer::from_seconds(0.3, TimerMode::Once) },
        ));
    }
}
```

---

## 10. フォントとスタイル

### 10.1 フォント定義

```rust
pub struct UiFonts {
    pub main: Handle<Font>,       // メインフォント（英数字）
    pub japanese: Handle<Font>,   // 日本語フォント
    pub score: Handle<Font>,      // スコア表示（等幅フォント）
}
```

### 10.2 カラーパレット

```rust
pub mod colors {
    use bevy::color::Color;

    pub const TEXT_WHITE:  Color = Color::srgb(1.0, 1.0, 1.0);
    pub const TEXT_YELLOW: Color = Color::srgb(1.0, 0.9, 0.0);
    pub const TEXT_RED:    Color = Color::srgb(1.0, 0.2, 0.2);
    pub const TEXT_CYAN:   Color = Color::srgb(0.3, 0.9, 1.0);

    pub const UI_BG:       Color = Color::srgba(0.05, 0.02, 0.1, 0.95);
    pub const UI_BORDER:   Color = Color::srgb(0.4, 0.2, 0.6);
    pub const HP_BAR_GREEN: Color = Color::srgb(0.2, 0.9, 0.2);
    pub const HP_BAR_RED:   Color = Color::srgb(0.9, 0.1, 0.1);

    pub const SPELL_CARD_BG: Color = Color::srgba(0.0, 0.0, 0.2, 0.8);
}
```

### 10.3 スタイル定数

```rust
pub mod styles {
    use bevy::ui::*;

    pub fn menu_button() -> Node {
        Node {
            width: Val::Px(280.0),
            height: Val::Px(48.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            margin: UiRect::vertical(Val::Px(4.0)),
            ..default()
        }
    }

    pub fn side_panel() -> Node {
        Node {
            width: Val::Px(256.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        }
    }
}
```

---

## 11. 入力マッピング

| キー | アクション |
|---|---|
| Z | ショット / 選択決定 |
| X | ボム |
| Shift | 低速移動（ヒットボックス表示） |
| ESC | ポーズ / キャンセル |
| 矢印 / WASD | 移動 |
| Enter | 選択決定（メニュー） |
| Alt+Enter | フルスクリーン切り替え |

```rust
pub fn player_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut player: Query<(&mut Transform, &PlayerStats), With<Player>>,
    mut shoot_events: EventWriter<ShootEvent>,
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

    let movement = dir.normalize_or_zero() * speed * time.delta_secs();
    tf.translation += movement.extend(0.0);

    // プレイエリア内に制限
    tf.translation.x = tf.translation.x.clamp(-192.0, 192.0);
    tf.translation.y = tf.translation.y.clamp(-224.0, 224.0);

    if keys.pressed(KeyCode::KeyZ) {
        shoot_events.write(ShootEvent);
    }
}
```

---

## 12. ローカライズ

Phase 1〜2 では日本語のみ対応。英語対応は後フェーズで検討。

```rust
// 将来拡張用
pub enum Language { Japanese, English }

pub struct Strings {
    pub game_start: &'static str,
    pub game_over: &'static str,
    // ...
}

impl Strings {
    pub fn for_lang(lang: Language) -> &'static Self {
        match lang {
            Language::Japanese => &JP_STRINGS,
            Language::English => &EN_STRINGS,
        }
    }
}
```
