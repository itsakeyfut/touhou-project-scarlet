# Phase 12: UI/HUD 実装

## 目標

プレイ中の全HUD要素が正しく表示・更新される。ボスHPバー、スペルカード名、エクステンドポップアップが動く。

## 完了条件

- [ ] サイドパネル（スコア、ハイスコア、残機、ボム、パワー、グレイズ）
- [ ] ボスHPバー（フェーズ区切り）
- [ ] ボスタイマー（カウントダウン）
- [ ] スペルカード名テキスト
- [ ] エクステンドポップアップ
- [ ] ボムフラッシュエフェクト

---

## タスク詳細

### 1. HUD セットアップ

```rust
// app/ui/src/screens/hud/mod.rs

pub fn setup_hud(mut commands: Commands, assets: Res<ScarletAssets>) {
    // サイドパネル
    commands.spawn((
        HudRoot,
        DespawnOnExit(AppState::Playing),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(384.0),  // プレイエリア右端
            top: Val::Px(0.0),
            width: Val::Px(256.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.01, 0.08, 0.9)),
    )).with_children(|parent| {
        spawn_score_section(parent, &assets);
        spawn_lives_section(parent, &assets);
        spawn_bombs_section(parent, &assets);
        spawn_power_section(parent, &assets);
        spawn_graze_section(parent, &assets);
    });

    // ボスHPバー（初期非表示）
    commands.spawn((
        BossHealthBar,
        DespawnOnExit(AppState::Playing),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            width: Val::Px(384.0),
            height: Val::Px(8.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.8, 0.1, 0.1)),
        Visibility::Hidden,
    ));

    // スペルカード名
    commands.spawn((
        SpellCardName,
        DespawnOnExit(AppState::Playing),
        Text::new(""),
        TextColor(Color::srgb(1.0, 0.8, 0.2)),
        TextFont { font_size: 16.0, ..default() },
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(8.0),
            top: Val::Px(12.0),
            ..default()
        },
        Visibility::Hidden,
    ));
}

fn spawn_score_section(parent: &mut ChildBuilder, assets: &ScarletAssets) {
    parent.spawn(Text::new("ハイスコア"));
    parent.spawn((HudHiScore, Text::new("0000000000")));
    parent.spawn(Text::new("スコア"));
    parent.spawn((HudScore, Text::new("0000000000")));
}

fn spawn_lives_section(parent: &mut ChildBuilder, assets: &ScarletAssets) {
    parent.spawn(Text::new("残機"));
    parent.spawn((HudLives, Node { flex_direction: FlexDirection::Row, ..default() }));
}

fn spawn_bombs_section(parent: &mut ChildBuilder, assets: &ScarletAssets) {
    parent.spawn(Text::new("ボム"));
    parent.spawn((HudBombs, Node { flex_direction: FlexDirection::Row, ..default() }));
}

fn spawn_power_section(parent: &mut ChildBuilder, assets: &ScarletAssets) {
    parent.spawn(Text::new("パワー"));
    parent.spawn((HudPower, Text::new("  0")));
}

fn spawn_graze_section(parent: &mut ChildBuilder, assets: &ScarletAssets) {
    parent.spawn(Text::new("グレイズ"));
    parent.spawn((HudGraze, Text::new("     0")));
}
```

### 2. スコア更新

```rust
// app/ui/src/screens/hud/score.rs

pub fn update_hud_score(
    game_data: Res<GameData>,
    mut score_text: Query<&mut Text, With<HudScore>>,
    mut hiscore_text: Query<&mut Text, (With<HudHiScore>, Without<HudScore>)>,
    mut graze_text: Query<&mut Text, (With<HudGraze>, Without<HudScore>, Without<HudHiScore>)>,
) {
    if !game_data.is_changed() { return; }

    if let Ok(mut t) = score_text.single_mut() {
        **t = format!("{:>10}", game_data.score);
    }
    if let Ok(mut t) = hiscore_text.single_mut() {
        **t = format!("{:>10}", game_data.score.max(game_data.hi_score));
    }
    if let Ok(mut t) = graze_text.single_mut() {
        **t = format!("{:>6}", game_data.graze_count);
    }
}
```

### 3. 残機・ボムアイコン更新

```rust
// app/ui/src/screens/hud/lives.rs

pub fn update_hud_lives(
    mut commands: Commands,
    game_data: Res<GameData>,
    hud_lives: Query<Entity, With<HudLives>>,
    assets: Res<ScarletAssets>,
) {
    if !game_data.is_changed() { return; }
    let Ok(entity) = hud_lives.single() else { return };

    // 子要素を全削除して再生成
    commands.entity(entity).despawn_descendants();
    commands.entity(entity).with_children(|parent| {
        for _ in 0..game_data.lives {
            parent.spawn(Sprite {
                image: assets.ui_life_icon.clone(),
                custom_size: Some(Vec2::splat(16.0)),
                ..default()
            });
        }
    });
}
```

### 4. パワーゲージ

```rust
// app/ui/src/screens/hud/power.rs

pub fn update_hud_power(
    game_data: Res<GameData>,
    mut power_text: Query<&mut Text, With<HudPower>>,
) {
    if !game_data.is_changed() { return; }
    if let Ok(mut t) = power_text.single_mut() {
        **t = format!("{:>3}", game_data.power);
    }
}
```

### 5. ボスHPバー

```rust
// app/ui/src/screens/hud/boss_bar.rs

pub fn show_boss_hp_bar(
    mut hp_bar: Query<(&mut Node, &mut Visibility, &mut BackgroundColor), With<BossHealthBar>>,
    bosses: Query<&Boss>,
) {
    let Ok((mut node, mut vis, mut color)) = hp_bar.single_mut() else { return };

    if let Ok(boss) = bosses.single() {
        *vis = Visibility::Visible;
        let current = &boss.phases[boss.current_phase];
        let ratio = (current.hp / current.hp_max).clamp(0.0, 1.0);
        node.width = Val::Px(384.0 * ratio);

        // スペルカード中は青、通常は赤
        color.0 = if boss.spell_card_active {
            Color::srgb(0.1, 0.4, 0.9)
        } else {
            Color::srgb(0.9, 0.1, 0.1)
        };
    } else {
        *vis = Visibility::Hidden;
    }
}

pub fn update_boss_timer(
    mut timer_text: Query<(&mut Text, &mut TextColor, &mut Visibility), With<BossTimer>>,
    bosses: Query<&Boss>,
) {
    let Ok((mut text, mut color, mut vis)) = timer_text.single_mut() else { return };

    if let Ok(boss) = bosses.single() {
        *vis = Visibility::Visible;
        let remaining = boss.phase_timer.remaining_secs();
        **text = format!("{:.0}", remaining);
        color.0 = if remaining < 10.0 {
            Color::srgb(1.0, 0.2, 0.2)
        } else {
            Color::WHITE
        };
    } else {
        *vis = Visibility::Hidden;
    }
}
```

### 6. スペルカード名表示

```rust
// app/ui/src/screens/hud/mod.rs

pub fn update_spell_card_name(
    mut spell_name: Query<(&mut Text, &mut Visibility), With<SpellCardName>>,
    mut phase_events: EventReader<BossPhaseChangedEvent>,
    bosses: Query<&Boss>,
) {
    for event in phase_events.read() {
        let Ok(boss) = bosses.get(event.entity) else { continue };
        let phase = &boss.phases[event.phase];
        let Ok((mut text, mut vis)) = spell_name.single_mut() else { continue };

        if let Some(name) = &phase.spell_card_name {
            **text = name.clone();
            *vis = Visibility::Visible;
        } else {
            **text = String::new();
            *vis = Visibility::Hidden;
        }
    }
}
```

### 7. エクステンドポップアップ

```rust
// app/ui/src/screens/hud/popups.rs

pub fn extend_popup(
    mut commands: Commands,
    mut extend_events: EventReader<ExtendEvent>,
) {
    for _ in extend_events.read() {
        commands.spawn((
            Text::new("EXTEND!"),
            TextColor(Color::srgb(1.0, 0.9, 0.0)),
            TextFont { font_size: 48.0, ..default() },
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(100.0),
                top: Val::Px(200.0),
                ..default()
            },
            FadeOut { timer: Timer::from_seconds(2.0, TimerMode::Once) },
            GlobalZIndex(50),
        ));
    }
}
```

### 8. ボムフラッシュ

```rust
pub fn bomb_flash(
    mut commands: Commands,
    mut bomb_events: EventReader<BombUsedEvent>,
) {
    for _ in bomb_events.read() {
        commands.spawn((
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

## 参照

- `docs/04_ui_ux.md` § 2 (HUD)
- `docs/references/vampire-survivors/app/ui/src/screens/hud/`
