# 08 クレートアーキテクチャ

## 概要

ワークスペースを構成する5つのクレートの依存関係、公開API、プラグインインターフェースを定義する。

---

## 1. クレート依存グラフ

```
touhou-project-scarlet (binary)
    ├── scarlet-core
    ├── scarlet-ui     → scarlet-core
    ├── scarlet-audio  → scarlet-core
    └── scarlet-assets → (bevy only, no game deps)
```

**設計原則**:
- `scarlet-core` はゲームロジックのすべてを持つ。他のクレートに依存しない
- `scarlet-ui` と `scarlet-audio` は `scarlet-core` の型・イベントを参照する
- `scarlet-assets` はBevy依存のみ。ゲームロジックを持たない
- `touhou-project-scarlet` (binary) はすべてのプラグインを組み合わせる

---

## 2. scarlet-core

### Cargo.toml

```toml
[package]
name = "scarlet-core"
version = "0.1.0"
edition = "2021"

[dependencies]
bevy = { workspace = true }
rand = { workspace = true }
ron = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }

[dev-dependencies]
tempfile = { workspace = true }
```

### 公開API (`lib.rs`)

```rust
pub use self::components::*;
pub use self::events::*;
pub use self::resources::*;
pub use self::states::AppState;
pub use self::types::*;

pub struct ScarletCorePlugin;

impl Plugin for ScarletCorePlugin {
    fn build(&self, app: &mut App);
}
```

### 公開コンポーネント

| コンポーネント | 用途 |
|---|---|
| `Player` | プレイヤーマーカー |
| `PlayerStats` | 速度・HP・当たり判定パラメータ |
| `ShootTimer` | 射撃インターバル管理 |
| `InvincibilityTimer` | 無敵時間 |
| `Enemy` | 敵マーカー、スコア値、ボス判定 |
| `EnemyKind` | 敵種別 |
| `EnemyMovement` | 移動パターン |
| `Boss` | ボス管理（フェーズ、スペルカード） |
| `BossPhaseData` | フェーズごとのHP・パターン |
| `PlayerBullet` | プレイヤー弾（ダメージ値） |
| `EnemyBullet` | 敵弾（ダメージ値） |
| `EnemyBulletKind` | 弾種（当たり判定半径） |
| `LaserBullet` | レーザー |
| `BulletVelocity` | 弾速度ベクタ |
| `BulletEmitter` | 弾幕パターン発射コンポーネント |
| `ItemKind` | アイテム種別 |
| `ItemPhysics` | アイテム物理（落下・引き寄せ） |
| `CircleCollider` | 汎用円形コライダー |
| `DespawnOutOfBounds` | 画面外自動消去マーカー |

### 公開リソース

| リソース | 用途 |
|---|---|
| `GameData` | スコア、残機、ボム、パワー |
| `BombState` | ボム発動中フラグ、タイマー |
| `StageData` | 現在ステージ、スクリプト |
| `EnemySpawner` | タイムラインベースのスポーン管理 |
| `ExtendData` | エクステンド閾値の達成状況 |
| `FragmentTracker` | ライフ/ボムフラグメントカウント |
| `SpatialGrid` | 衝突判定空間グリッド |
| `SelectedCharacter` | 選択キャラクター |
| `SelectedDifficulty` | 選択難易度 |

### 公開イベント

```rust
pub struct PlayerHitEvent;
pub struct GrazeEvent { pub bullet_entity: Entity }
pub struct BombUsedEvent { pub is_counter_bomb: bool }
pub struct EnemyDefeatedEvent { pub enemy_entity: Entity, pub score: u32 }
pub struct BossPhaseChangedEvent { pub entity: Entity, pub phase: usize }
pub struct ItemCollectedEvent { pub kind: ItemKind, pub score: u32 }
pub struct ExtendEvent { pub threshold: u64 }
pub struct StageClearedEvent { pub stage: u8 }
pub struct ShootEvent;
```

### SystemSet

```rust
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameSystemSet {
    Input,
    PlayerLogic,
    BulletEmit,
    Movement,
    Collision,
    GameLogic,
    StageControl,
    Effects,
    Cleanup,
}
```

---

## 3. scarlet-ui

### Cargo.toml

```toml
[package]
name = "scarlet-ui"
version = "0.1.0"
edition = "2021"

[dependencies]
bevy = { workspace = true }
scarlet-core = { path = "../core" }
scarlet-assets = { path = "../assets" }
```

### 公開API

```rust
pub struct ScarletUiPlugin;

impl Plugin for ScarletUiPlugin {
    fn build(&self, app: &mut App) {
        app
            // タイトル
            .add_systems(OnEnter(AppState::Title), screens::title::setup_title)
            .add_systems(OnExit(AppState::Title), cleanup::<TitleScreen>)
            .add_systems(Update, screens::title::title_input
                .run_if(in_state(AppState::Title)))

            // キャラ選択
            .add_systems(OnEnter(AppState::CharacterSelect), screens::character_select::setup)
            .add_systems(Update, screens::character_select::character_select_input
                .run_if(in_state(AppState::CharacterSelect)))

            // 難易度選択
            .add_systems(OnEnter(AppState::DifficultySelect), screens::difficulty_select::setup)

            // プレイ中HUD
            .add_systems(OnEnter(AppState::Playing), hud::setup_hud)
            .add_systems(Update, (
                hud::score::update_hud_score,
                hud::lives::update_hud_lives,
                hud::bombs::update_hud_bombs,
                hud::power::update_hud_power,
                hud::boss_bar::update_boss_hp_bar,
                hud::boss_bar::update_boss_timer,
                hud::popups::extend_popup,
                hud::popups::bomb_flash,
            ).run_if(in_state(AppState::Playing)))

            // ポーズ
            .add_systems(OnEnter(AppState::Paused), screens::pause::setup_pause)
            .add_systems(Update, screens::pause::pause_input
                .run_if(in_state(AppState::Paused)))

            // ゲームオーバー
            .add_systems(OnEnter(AppState::GameOver), screens::game_over::setup)
            .add_systems(Update, screens::game_over::game_over_input
                .run_if(in_state(AppState::GameOver)))

            // カメラ
            .add_systems(Startup, camera::setup_camera);
    }
}
```

### UIコンポーネント公開型

```rust
// components.rs で公開
pub struct TitleScreen;
pub struct CharacterSelectScreen;
pub struct DifficultySelectScreen;
pub struct PauseMenu;
pub struct GameOverScreen;
pub struct HudRoot;
pub struct HudScore;
pub struct HudHiScore;
pub struct HudLives;
pub struct HudBombs;
pub struct HudPower;
pub struct HudPowerBar;
pub struct HudGraze;
pub struct BossHealthBar;
pub struct BossTimer;
pub struct BossName;
pub struct SpellCardName;
pub struct FadeOut { pub timer: Timer }
```

---

## 4. scarlet-audio

### Cargo.toml

```toml
[package]
name = "scarlet-audio"
version = "0.1.0"
edition = "2021"

[dependencies]
bevy = { workspace = true }
bevy_kira_audio = { workspace = true }
scarlet-core = { path = "../core" }
```

### 公開API

```rust
pub struct ScarletAudioPlugin;

impl Plugin for ScarletAudioPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(AudioPlugin)
            .add_audio_channel::<BgmChannel>()
            .add_audio_channel::<SfxChannel>()
            .init_resource::<AudioHandles>()
            .init_resource::<AudioSettings>()
            .add_event::<PlayBgmEvent>()
            .add_event::<StopBgmEvent>()
            .add_event::<PlaySfxEvent>()
            .add_systems(Startup, set_channel_volumes)
            .add_systems(Update, (
                bgm::handle_bgm_transitions,
                sfx::handle_sfx_events,
                apply_audio_settings,
            ))
            // State-driven BGM
            .add_systems(OnEnter(AppState::Title), bgm::on_enter_title)
            .add_systems(OnEnter(AppState::Playing), bgm::on_enter_playing)
            .add_systems(OnEnter(AppState::Paused), bgm::on_pause)
            .add_systems(OnExit(AppState::Paused), bgm::on_resume)
            .add_systems(OnEnter(AppState::GameOver), bgm::on_game_over)
            // SFX triggers
            .add_systems(Update, (
                sfx::triggers::sfx_on_shoot,
                sfx::triggers::sfx_on_player_hit,
                sfx::triggers::sfx_on_graze,
                sfx::triggers::sfx_on_enemy_die,
                sfx::triggers::sfx_on_item_collect,
                sfx::triggers::sfx_on_extend,
                sfx::triggers::sfx_on_spell_card,
                sfx::triggers::sfx_on_bomb,
            ).run_if(in_state(AppState::Playing)));
    }
}
```

### 公開型

```rust
pub struct BgmChannel;
pub struct SfxChannel;
pub struct AudioHandles { /* ... */ }
pub struct AudioSettings { pub master_volume: f32, pub bgm_volume: f32, pub sfx_volume: f32 }
pub struct PlayBgmEvent { pub track: BgmTrack, pub fade_in_secs: f32 }
pub struct StopBgmEvent { pub fade_out_secs: f32 }
pub struct PlaySfxEvent { pub sfx: SfxKind, pub volume: f32 }
pub enum BgmTrack { Title, Stage(u8), Boss(BossType), GameOver, Ending, StaffRoll }
pub enum SfxKind { /* ... */ }
```

---

## 5. scarlet-assets

### Cargo.toml

```toml
[package]
name = "scarlet-assets"
version = "0.1.0"
edition = "2021"

[dependencies]
bevy = { workspace = true }
```

### 公開API

```rust
pub struct ScarletAssetsPlugin;

impl Plugin for ScarletAssetsPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ScarletAssets>()
            .init_resource::<AssetsLoading>()
            .add_systems(OnEnter(AppState::Loading), load_assets)
            .add_systems(Update, check_assets_loaded
                .run_if(in_state(AppState::Loading)));
    }
}
```

```rust
#[derive(Resource, Default)]
pub struct ScarletAssets {
    // スプライト
    pub player_reimu: Handle<Image>,
    pub player_marisa: Handle<Image>,
    pub enemy_fairy: Handle<Image>,
    pub bullet_sheet: Handle<Image>,
    pub bullet_sheet_layout: Handle<TextureAtlasLayout>,
    pub item_sheet: Handle<Image>,
    pub item_sheet_layout: Handle<TextureAtlasLayout>,
    pub boss_rumia: Handle<Image>,
    // ... 全ボスキャラクター

    // UI
    pub title_logo: Handle<Image>,
    pub ui_life_icon: Handle<Image>,
    pub ui_bomb_icon: Handle<Image>,

    // フォント
    pub font_main: Handle<Font>,
    pub font_japanese: Handle<Font>,
    pub font_score: Handle<Font>,
}

#[derive(Resource, Default)]
pub struct AssetsLoading {
    pub handles: Vec<UntypedHandle>,
}
```

### アセット読み込みフロー

```rust
fn load_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading: ResMut<AssetsLoading>,
) {
    let mut assets = ScarletAssets::default();

    assets.player_reimu = asset_server.load("sprites/player/reimu.png");
    assets.player_marisa = asset_server.load("sprites/player/marisa.png");
    assets.bullet_sheet = asset_server.load("sprites/bullets/bullet_sheet.png");
    // ...

    loading.handles.push(assets.player_reimu.clone().untyped());
    // ...

    commands.insert_resource(assets);
}

fn check_assets_loaded(
    loading: Res<AssetsLoading>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let all_loaded = loading.handles.iter().all(|h| {
        asset_server.is_loaded_with_dependencies(h.id())
    });

    if all_loaded {
        next_state.set(AppState::Playing);
    }
}
```

---

## 6. touhou-project-scarlet (binary)

### Cargo.toml

```toml
[package]
name = "touhou-project-scarlet"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "touhou-project-scarlet"
path = "src/main.rs"

[dependencies]
bevy = { workspace = true }
scarlet-core = { path = "../core" }
scarlet-ui = { path = "../ui" }
scarlet-audio = { path = "../audio" }
scarlet-assets = { path = "../assets" }
```

### `main.rs`

```rust
use bevy::prelude::*;
use scarlet_assets::ScarletAssetsPlugin;
use scarlet_audio::ScarletAudioPlugin;
use scarlet_core::ScarletCorePlugin;
use scarlet_ui::ScarletUiPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "東方紅魔郷 ~ Embodiment of Scarlet Devil".into(),
                        resolution: (640.0, 480.0).into(),
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()), // ピクセルアートのため
            ScarletAssetsPlugin,
            ScarletCorePlugin,
            ScarletUiPlugin,
            ScarletAudioPlugin,
        ))
        .run();
}
```

---

## 7. クレート間の型共有パターン

`scarlet-ui` から `scarlet-core` の型を参照する例：

```rust
// scarlet-ui/src/screens/hud/score.rs
use scarlet_core::{GameData, HudScore, HudHiScore};

pub fn update_hud_score(
    game_data: Res<GameData>,
    mut score_text: Query<&mut Text, With<HudScore>>,
) {
    // ...
}
```

`scarlet-audio` からゲームイベントを購読する例：

```rust
// scarlet-audio/src/sfx/triggers.rs
use scarlet_core::{GrazeEvent, PlayerHitEvent, ExtendEvent};
use crate::{PlaySfxEvent, SfxKind};

pub fn sfx_on_graze(
    mut graze_events: EventReader<GrazeEvent>,
    mut sfx: EventWriter<PlaySfxEvent>,
) {
    for _ in graze_events.read() {
        sfx.write(PlaySfxEvent { sfx: SfxKind::Graze, volume: 0.4 });
    }
}
```

---

## 8. テスト戦略

### ユニットテスト (scarlet-core)

純粋関数はインラインテストで検証：

```rust
// systems/collision.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graze_zone() {
        let player = Vec2::ZERO;
        let bullet_in_graze = Vec2::new(15.0, 0.0);
        let bullet_outside = Vec2::new(30.0, 0.0);
        let bullet_in_hitbox = Vec2::new(2.0, 0.0);

        let bullet_r = 4.0;

        // グレイズ圏内
        let dist = (bullet_in_graze - player).length();
        assert!(dist < GRAZE_RADIUS + bullet_r);
        assert!(dist >= PLAYER_HITBOX_RADIUS + bullet_r);

        // 当たり判定外
        let dist = (bullet_outside - player).length();
        assert!(dist >= GRAZE_RADIUS + bullet_r);
    }
}
```

### 統合テスト (MinimalPlugins)

```rust
// app/core/tests/integration_test.rs

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use scarlet_core::{ScarletCorePlugin, AppState, Player};

#[test]
fn player_spawns_on_enter_playing() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin, ScarletCorePlugin));
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Playing);
    app.update();
    app.update(); // OnEnter runs after state change

    let player_count = app.world().query::<&Player>().iter(app.world()).count();
    assert_eq!(player_count, 1);
}
```

---

## 9. フィーチャーフラグ

```toml
# scarlet-core/Cargo.toml
[features]
default = []
debug-hitbox = []  # デバッグヒットボックス表示
```

```rust
// デバッグヒットボックス
#[cfg(feature = "debug-hitbox")]
app.add_systems(Update, debug_hitbox_system);
```

開発時の実行:
```bash
cargo run -p touhou-project-scarlet --features scarlet-core/debug-hitbox
```
