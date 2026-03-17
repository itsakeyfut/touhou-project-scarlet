# 07 プロジェクト構造

## ディレクトリツリー

```
touhou-project-scarlet/
├── Cargo.toml                    # ワークスペース設定
├── justfile                      # タスクランナー
├── .gitignore
├── README.md
│
├── app/
│   ├── core/                     # scarlet-core クレート
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # ScarletCorePlugin 定義
│   │       ├── states.rs         # AppState enum
│   │       │
│   │       ├── shaders/          # WGSLシェーダーMaterial2d定義
│   │       │   ├── mod.rs
│   │       │   ├── plugin.rs         # ScarletShadersPlugin
│   │       │   ├── bullet_glow.rs    # BulletGlowMaterial
│   │       │   ├── bullet_trail.rs   # BulletTrailMaterial
│   │       │   ├── graze_field.rs    # GrazeMaterial
│   │       │   ├── hit_flash.rs      # HitFlashMaterial
│   │       │   ├── spell_card_bg.rs  # SpellCardBgMaterial
│   │       │   ├── bomb_reimu.rs     # BombReimuMaterial
│   │       │   ├── bomb_marisa.rs    # BombMarisaMaterial
│   │       │   └── pixel_outline.rs  # PixelOutlineMaterial
│   │       │
│   │       ├── types/            # Bevy依存なし純粋なドメイン型
│   │       │   ├── mod.rs
│   │       │   ├── character.rs  # CharacterType, ShotType
│   │       │   ├── enemy.rs      # EnemyType, BossType, AIType
│   │       │   ├── weapon.rs     # BulletPattern, BulletKind
│   │       │   └── game.rs       # Difficulty, ItemKind
│   │       │
│   │       ├── components/       # ECS コンポーネント
│   │       │   ├── mod.rs
│   │       │   ├── player.rs     # Player, PlayerStats, ShootTimer, GrazeBox
│   │       │   ├── enemy.rs      # Enemy, EnemyKind, EnemyMovement, Boss, BossPhase
│   │       │   ├── bullet.rs     # PlayerBullet, EnemyBullet, LaserBullet, BulletEmitter
│   │       │   ├── item.rs       # ItemKind, ItemPhysics
│   │       │   └── physics.rs    # CircleCollider, DespawnOutOfBounds
│   │       │
│   │       ├── resources/        # ECS リソース
│   │       │   ├── mod.rs
│   │       │   ├── game_data.rs  # GameData (スコア、残機、ボム、パワー)
│   │       │   ├── bomb.rs       # BombState
│   │       │   ├── stage.rs      # StageData, StageScript
│   │       │   ├── spawner.rs    # EnemySpawner
│   │       │   ├── extend.rs     # ExtendData
│   │       │   ├── fragment.rs   # FragmentTracker
│   │       │   ├── spatial.rs    # SpatialGrid
│   │       │   └── selected.rs   # SelectedCharacter, SelectedDifficulty
│   │       │
│   │       ├── events.rs         # 全カスタムイベント定義
│   │       │
│   │       └── systems/          # ゲームロジックシステム
│   │           ├── mod.rs
│   │           ├── player.rs     # spawn_player, player_movement, player_hit
│   │           ├── shoot.rs      # player_shoot, spawn_player_bullet
│   │           ├── bullet.rs     # bullet_movement, despawn_out_of_bounds
│   │           ├── danmaku/      # 弾幕パターン
│   │           │   ├── mod.rs
│   │           │   ├── emitter.rs    # BulletEmitter 処理
│   │           │   ├── patterns.rs   # emit_ring, emit_aimed, emit_spiral 等
│   │           │   └── laser.rs      # LaserBullet 処理
│   │           ├── collision.rs  # check_circle_collision, player_hit_detection
│   │           ├── graze.rs      # graze_detection_system
│   │           ├── bomb.rs       # bomb_input, bomb_effect
│   │           ├── enemy/        # 敵関連
│   │           │   ├── mod.rs
│   │           │   ├── spawn.rs      # spawn_enemy
│   │           │   ├── movement.rs   # enemy_movement_system
│   │           │   ├── hit.rs        # enemy_hit_system
│   │           │   └── cull.rs       # enemy_cull_system
│   │           ├── boss/         # ボス関連
│   │           │   ├── mod.rs
│   │           │   ├── phase.rs      # boss_phase_system
│   │           │   ├── movement.rs   # boss_movement_system
│   │           │   ├── spell_card.rs # spell_card_bonus
│   │           │   └── bosses/       # 各ボス実装
│   │           │       ├── rumia.rs
│   │           │       ├── cirno.rs
│   │           │       ├── meiling.rs
│   │           │       ├── patchouli.rs
│   │           │       ├── sakuya.rs
│   │           │       ├── remilia.rs
│   │           │       └── flandre.rs
│   │           ├── item.rs       # item_attract, item_collection
│   │           ├── score.rs      # score_update, extend_check
│   │           ├── stage.rs      # stage_script_system, stage_clear
│   │           ├── difficulty.rs # difficulty params, rank system
│   │           └── effects/      # 視覚エフェクト
│   │               ├── mod.rs
│   │               ├── flash.rs      # 被弾フラッシュ、ボムフラッシュ
│   │               ├── particles.rs  # パーティクルエフェクト
│   │               └── shake.rs      # 画面揺れ
│   │
│   ├── ui/                       # scarlet-ui クレート
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # ScarletUiPlugin
│   │       ├── camera.rs         # カメラセットアップ
│   │       ├── styles.rs         # UIスタイル定数
│   │       ├── components.rs     # UIマーカーコンポーネント
│   │       └── screens/
│   │           ├── mod.rs
│   │           ├── title.rs         # タイトル画面
│   │           ├── character_select.rs # キャラ選択画面
│   │           ├── difficulty_select.rs # 難易度選択
│   │           ├── loading.rs       # ロード画面
│   │           ├── pause.rs         # ポーズメニュー
│   │           ├── game_over.rs     # ゲームオーバー画面
│   │           ├── stage_clear.rs   # ステージクリア画面
│   │           ├── ending.rs        # エンディング
│   │           └── hud/
│   │               ├── mod.rs
│   │               ├── score.rs     # スコア表示
│   │               ├── lives.rs     # 残機表示
│   │               ├── bombs.rs     # ボム表示
│   │               ├── power.rs     # パワーゲージ
│   │               ├── boss_bar.rs  # ボスHPバー
│   │               └── popups.rs    # エクステンド等ポップアップ
│   │
│   ├── audio/                    # scarlet-audio クレート
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # ScarletAudioPlugin
│   │       ├── channels.rs       # BgmChannel, SfxChannel
│   │       ├── handles.rs        # AudioHandles リソース
│   │       ├── bgm.rs            # BGM再生・遷移システム
│   │       └── sfx/
│   │           ├── mod.rs
│   │           ├── events.rs     # PlaySfxEvent, SfxKind
│   │           └── triggers.rs   # ゲームイベント→SE変換
│   │
│   ├── assets/                   # scarlet-assets クレート
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # ScarletAssetsPlugin
│   │       └── loader.rs         # ScarletAssets リソース、アセット読み込み
│   │
│   └── touhou-project-scarlet/  # バイナリクレート
│       ├── Cargo.toml
│       └── src/
│           └── main.rs           # App 組み立て・Plugin 登録
│
├── docs/                         # ドキュメント
│   ├── specification.md          # ゲーム仕様書
│   ├── 02_architecture.md        # Bevy ECSアーキテクチャ
│   ├── 03_danmaku_systems.md     # 弾幕システム詳細
│   ├── 04_ui_ux.md              # UI/UXデザイン
│   ├── 05_audio.md              # オーディオ設計
│   ├── 06_implementation_plan.md # 実装計画
│   ├── 07_project_structure.md   # このファイル
│   ├── 08_crate_architecture.md  # クレート設計
│   ├── 09_quick_reference.md     # クイックリファレンス
│   └── roadmap/
│       ├── README.md
│       ├── phase-01.md
│       ├── ...
│       └── phase-19.md
│
└── docs/references/              # 参照実装
    ├── suika-game/               # すいかゲームクローン（Bevy 0.17参照）
    └── vampire-survivors/        # Vampire Survivorsクローン（Bevy 0.17参照）
```

---

## ファイル別責務

### `app/touhou-project-scarlet/src/main.rs`

```rust
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "東方紅魔郷 ~ Embodiment of Scarlet Devil".into(),
                    resolution: (640.0, 480.0).into(),
                    ..default()
                }),
                ..default()
            }),
            ScarletAssetsPlugin,
            ScarletCorePlugin,
            ScarletUiPlugin,
            ScarletAudioPlugin,
        ))
        .run();
}
```

### `app/core/src/lib.rs`

`ScarletCorePlugin` がシステム・リソース・イベントをすべて登録する。

```rust
pub struct ScarletCorePlugin;

impl Plugin for ScarletCorePlugin {
    fn build(&self, app: &mut App) {
        app
            // State
            .init_state::<AppState>()

            // Resources
            .init_resource::<GameData>()
            .init_resource::<BombState>()
            .init_resource::<StageData>()
            .init_resource::<SpatialGrid>()
            .init_resource::<FragmentTracker>()
            .init_resource::<SelectedCharacter>()
            .init_resource::<SelectedDifficulty>()

            // Events
            .add_event::<PlayerHitEvent>()
            .add_event::<GrazeEvent>()
            .add_event::<BombUsedEvent>()
            .add_event::<EnemyDefeatedEvent>()
            .add_event::<BossPhaseChangedEvent>()
            .add_event::<ItemCollectedEvent>()
            .add_event::<ExtendEvent>()
            .add_event::<StageClearedEvent>()
            .add_event::<ShootEvent>()

            // System Sets
            .configure_sets(Update, (
                GameSystemSet::Input,
                GameSystemSet::PlayerLogic,
                GameSystemSet::BulletEmit,
                GameSystemSet::Movement,
                GameSystemSet::Collision,
                GameSystemSet::GameLogic,
                GameSystemSet::StageControl,
                GameSystemSet::Effects,
                GameSystemSet::Cleanup,
            ).chain().run_if(in_state(AppState::Playing)))

            // Systems (Playing state)
            .add_systems(OnEnter(AppState::Playing), spawn_player)
            // ... システム登録
            ;
    }
}
```

### `app/core/src/states.rs`

```rust
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Title,
    CharacterSelect,
    DifficultySelect,
    Loading,
    Playing,
    Paused,
    StageClear,
    GameOver,
    Ending,
    StaffRoll,
}
```

### `app/core/src/events.rs`

```rust
#[derive(Event)] pub struct PlayerHitEvent;
#[derive(Event)] pub struct GrazeEvent { pub bullet_entity: Entity }
#[derive(Event)] pub struct BombUsedEvent { pub is_counter_bomb: bool }
#[derive(Event)] pub struct EnemyDefeatedEvent { pub enemy_entity: Entity, pub score: u32 }
#[derive(Event)] pub struct BossPhaseChangedEvent { pub entity: Entity, pub phase: usize }
#[derive(Event)] pub struct ItemCollectedEvent { pub kind: ItemKind, pub score: u32 }
#[derive(Event)] pub struct ExtendEvent { pub threshold: u64 }
#[derive(Event)] pub struct StageClearedEvent { pub stage: u8 }
#[derive(Event)] pub struct ShootEvent;
```

---

## アセットディレクトリ構造

```
app/touhou-project-scarlet/assets/
├── audio/
│   ├── bgm/         # BGM OGGファイル
│   └── sfx/         # SE OGGファイル
├── sprites/
│   ├── player/
│   │   ├── reimu.png         # 霊夢スプライトシート
│   │   └── marisa.png        # 魔理沙スプライトシート
│   ├── enemies/
│   │   ├── fairy.png
│   │   ├── bat.png
│   │   └── bosses/
│   │       ├── rumia.png
│   │       ├── cirno.png
│   │       ├── meiling.png
│   │       ├── patchouli.png
│   │       ├── sakuya.png
│   │       ├── remilia.png
│   │       └── flandre.png
│   ├── bullets/
│   │   └── bullet_sheet.png  # 全弾種スプライトシート
│   ├── items/
│   │   └── items.png         # アイテムスプライトシート
│   ├── effects/
│   │   ├── explosion.png
│   │   └── graze_spark.png
│   └── ui/
│       ├── title_logo.png
│       ├── life_icon.png
│       ├── bomb_icon.png
│       └── backgrounds/
│           ├── stage1_bg.png
│           └── ...
├── fonts/
│   ├── main.ttf
│   └── japanese.ttf
├── config/
│   ├── player.ron    # プレイヤーパラメータ
│   ├── enemy.ron     # 敵パラメータ
│   └── game.ron      # ゲーム全般パラメータ
└── shaders/
    ├── bullet_glow.wgsl
    ├── bullet_trail.wgsl
    ├── graze_field.wgsl
    ├── hit_flash.wgsl
    ├── spell_card_bg.wgsl
    ├── bomb_reimu.wgsl
    ├── bomb_marisa.wgsl
    ├── pixel_outline.wgsl
    ├── post_bloom.wgsl
    ├── post_crt.wgsl
    └── common/
        ├── math.wgsl
        └── noise.wgsl
```

---

## RON設定ファイル

### `assets/config/player.ron`

```ron
(
    base_speed: 200.0,
    slow_speed: 100.0,
    base_hp: 1,          // STGなので1ヒットで死亡
    pickup_radius: 80.0,
    hitbox_radius: 2.5,
    graze_radius: 16.0,
    initial_power: 0,
    initial_lives: 3,
    initial_bombs: 3,
)
```

### `assets/config/game.ron`

```ron
(
    play_area_width: 384.0,
    play_area_height: 448.0,
    score_line_y: 192.0,
    point_item_max_value: 10000,
    point_item_min_value: 100,
    graze_score: 500,
    extend_thresholds: [10000000, 20000000, 40000000, 60000000],
    max_bombs: 3,
    max_lives: 8,
    bomb_duration: 3.5,
    counter_bomb_window: 0.1,
    invincible_duration: 3.0,
    bomb_invincible_duration: 5.0,
)
```

### `assets/config/enemy.ron`

```ron
(
    fairy_hp: 10.0,
    fairy_score: 100,
    fairy_bullet_speed: 120.0,
    bat_hp: 5.0,
    bat_score: 50,
    // ...
)
```

---

## テスト構成

```
app/core/src/systems/collision.rs  # unit test: check_circle_collision
app/core/src/systems/score.rs      # unit test: calc_point_item_value
app/core/src/systems/graze.rs      # unit test: graze zone logic
app/core/src/resources/spatial.rs  # unit test: SpatialGrid insert/query
```

テストの実行:
```bash
just unit-test scarlet-core
cargo test -p scarlet-core -- --nocapture
```

統合テストは `MinimalPlugins + StatesPlugin` を使用。物理時間は `Time::advance_by` で制御。
