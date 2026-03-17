# Phase 11: ゲームフロー

## 目標

タイトルからゲームオーバーまでの全フローが繋がる。

## 完了条件

- [ ] タイトル → キャラクター選択 → 難易度選択 → ロード → プレイ
- [ ] ESCキーでポーズ → 再開/タイトルへ
- [ ] ゲームオーバー → コンティニュー(3回) / タイトルへ
- [ ] ステージクリア画面（スコア集計）
- [ ] `AppState` 全遷移の動作確認

---

## タスク詳細

### 1. タイトル画面

```rust
// app/ui/src/screens/title.rs

#[derive(Component)] pub struct TitleScreen;

pub fn setup_title(
    mut commands: Commands,
    assets: Res<ScarletAssets>,
) {
    commands.spawn((
        TitleScreen,
        DespawnOnExit(AppState::Title),
        Node { width: Val::Percent(100.0), height: Val::Percent(100.0), .. },
    )).with_children(|parent| {
        // タイトルロゴ
        parent.spawn(Text::new("東方紅魔郷\n~ Embodiment of Scarlet Devil ~"));

        // メニュー項目
        spawn_menu_item(parent, "ゲームスタート");
        spawn_menu_item(parent, "エクストラスタート");
        spawn_menu_item(parent, "終了");
    });
}

pub fn title_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut selected_menu: Local<usize>,
    mut app_exit: EventWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::ArrowDown) { *selected_menu = (*selected_menu + 1) % 3; }
    if keys.just_pressed(KeyCode::ArrowUp)   { *selected_menu = selected_menu.wrapping_sub(1) % 3; }

    if keys.just_pressed(KeyCode::KeyZ) || keys.just_pressed(KeyCode::Return) {
        match *selected_menu {
            0 => next_state.set(AppState::CharacterSelect),
            1 => { /* Extra: Phase 17 */ }
            2 => { app_exit.write(AppExit::Success); }
            _ => {}
        }
    }
}
```

### 2. キャラクター選択

```rust
// app/ui/src/screens/character_select.rs

pub fn setup_character_select(mut commands: Commands, assets: Res<ScarletAssets>) {
    // 霊夢 / 魔理沙 の選択UI
}

pub fn character_select_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut selected: ResMut<SelectedCharacter>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::ArrowLeft) {
        selected.character = CharacterType::Reimu;
    }
    if keys.just_pressed(KeyCode::ArrowRight) {
        selected.character = CharacterType::Marisa;
    }
    if keys.just_pressed(KeyCode::KeyZ) || keys.just_pressed(KeyCode::Return) {
        next_state.set(AppState::DifficultySelect);
    }
    if keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::KeyX) {
        next_state.set(AppState::Title);
    }
}
```

### 3. 難易度選択

```rust
// app/ui/src/screens/difficulty_select.rs

pub fn difficulty_select_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut selected: ResMut<SelectedDifficulty>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let difficulties = [Difficulty::Easy, Difficulty::Normal, Difficulty::Hard, Difficulty::Lunatic];
    let current_idx = difficulties.iter().position(|d| *d == selected.difficulty).unwrap_or(1);

    if keys.just_pressed(KeyCode::ArrowUp) && current_idx > 0 {
        selected.difficulty = difficulties[current_idx - 1];
    }
    if keys.just_pressed(KeyCode::ArrowDown) && current_idx < difficulties.len() - 1 {
        selected.difficulty = difficulties[current_idx + 1];
    }
    if keys.just_pressed(KeyCode::KeyZ) || keys.just_pressed(KeyCode::Return) {
        next_state.set(AppState::Loading);
    }
    if keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::KeyX) {
        next_state.set(AppState::CharacterSelect);
    }
}
```

### 4. ロード画面

```rust
// app/ui/src/screens/loading.rs
// アセットが全てロード済みなら即 Playing に遷移

pub fn setup_loading(mut commands: Commands) {
    commands.spawn((
        DespawnOnExit(AppState::Loading),
        Text::new("Loading..."),
    ));
}

pub fn check_loading_complete(
    loading: Res<AssetsLoading>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<AppState>>,
    mut game_data: ResMut<GameData>,
) {
    let all_ready = loading.handles.iter()
        .all(|h| asset_server.is_loaded_with_dependencies(h.id()));

    if all_ready {
        // ゲームデータリセット
        *game_data = GameData::default();
        next_state.set(AppState::Playing);
    }
}
```

### 5. ポーズシステム

```rust
// app/ui/src/screens/pause.rs

pub fn pause_toggle(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if !keys.just_pressed(KeyCode::Escape) { return; }
    match state.get() {
        AppState::Playing => next_state.set(AppState::Paused),
        AppState::Paused  => next_state.set(AppState::Playing),
        _ => {}
    }
}

pub fn pause_menu_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut selected: Local<usize>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::ArrowDown) { *selected = (*selected + 1) % 2; }
    if keys.just_pressed(KeyCode::ArrowUp)   { *selected = selected.wrapping_sub(1) % 2; }

    if keys.just_pressed(KeyCode::KeyZ) || keys.just_pressed(KeyCode::Return) {
        match *selected {
            0 => next_state.set(AppState::Playing),
            1 => next_state.set(AppState::Title),
            _ => {}
        }
    }
}
```

### 6. ゲームオーバー

```rust
// app/ui/src/screens/game_over.rs

#[derive(Resource, Default)]
pub struct ContinueData {
    pub continues_used: u8,
    pub max_continues: u8,
}

impl Default for ContinueData {
    fn default() -> Self {
        Self { continues_used: 0, max_continues: 3 }
    }
}

pub fn game_over_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut selected: Local<usize>,
    mut continue_data: ResMut<ContinueData>,
    mut game_data: ResMut<GameData>,
    mut next_state: ResMut<NextState<AppState>>,
    mut stage_data: ResMut<StageData>,
) {
    let can_continue = continue_data.continues_used < continue_data.max_continues;
    let menu_len = if can_continue { 2 } else { 1 };

    if keys.just_pressed(KeyCode::ArrowDown) { *selected = (*selected + 1) % menu_len; }
    if keys.just_pressed(KeyCode::ArrowUp)   { *selected = selected.wrapping_sub(1) % menu_len; }

    if keys.just_pressed(KeyCode::KeyZ) || keys.just_pressed(KeyCode::Return) {
        match *selected {
            0 if can_continue => {
                continue_data.continues_used += 1;
                game_data.lives = 2; // コンティニュー: 残機リセット
                game_data.bombs = 3;
                game_data.power = 0;
                game_data.score = 0; // ノーコンボーナス消滅
                next_state.set(AppState::Playing);
            }
            _ => {
                *continue_data = ContinueData::default();
                next_state.set(AppState::Title);
            }
        }
    }
}
```

### 7. ステージクリア集計

```rust
// app/ui/src/screens/stage_clear.rs

pub fn setup_stage_clear(
    mut commands: Commands,
    game_data: Res<GameData>,
    stage_data: Res<StageData>,
) {
    let lives_bonus = game_data.lives as u64 * 50_000;
    let bombs_bonus = game_data.bombs as u64 * 5_000;

    commands.spawn((
        DespawnOnExit(AppState::StageClear),
        Node { .. },
    )).with_children(|parent| {
        parent.spawn(Text::new(format!("STAGE {} CLEAR", stage_data.current_stage)));
        parent.spawn(Text::new(format!("残機ボーナス: {:>8}", lives_bonus)));
        parent.spawn(Text::new(format!("ボムボーナス:  {:>8}", bombs_bonus)));
    });
}

pub fn stage_clear_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut stage_data: ResMut<StageData>,
    mut game_data: ResMut<GameData>,
    mut spawner: ResMut<EnemySpawner>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::KeyZ) || keys.just_pressed(KeyCode::Return) {
        // ボーナス加算
        game_data.score += game_data.lives as u64 * 50_000;
        game_data.score += game_data.bombs as u64 * 5_000;

        // 次ステージへ
        stage_data.current_stage += 1;
        stage_data.stage_elapsed = 0.0;
        stage_data.boss_spawned = false;
        stage_data.boss_defeated = false;
        spawner.index = 0;
        spawner.elapsed = 0.0;

        if stage_data.current_stage > 6 {
            next_state.set(AppState::Ending);
        } else {
            next_state.set(AppState::Playing);
        }
    }
}
```

### 8. SelectedCharacter / SelectedDifficulty リソース

```rust
// app/core/src/resources/selected.rs

#[derive(Resource)]
pub struct SelectedCharacter {
    pub character: CharacterType,
}

impl Default for SelectedCharacter {
    fn default() -> Self {
        Self { character: CharacterType::Reimu }
    }
}

#[derive(Resource)]
pub struct SelectedDifficulty {
    pub difficulty: Difficulty,
}

impl Default for SelectedDifficulty {
    fn default() -> Self {
        Self { difficulty: Difficulty::Normal }
    }
}
```

---

## 参照

- `docs/04_ui_ux.md` § タイトル〜ゲームオーバー各画面
- `docs/references/vampire-survivors/app/ui/src/screens/`
