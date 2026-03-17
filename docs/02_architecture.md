# 東方紅魔郷クローン - 技術アーキテクチャ設計書

## 1. 技術スタック

### 1.1 コアテクノロジー
- **ゲームエンジン**: Bevy 0.17.3
  - ECS（Entity Component System）アーキテクチャ
  - 高性能な並列システム処理
  - Rustの安全性と速度
  - 2D機能セット使用

### 1.2 主要依存クレート

```toml
[workspace.dependencies]
bevy = { version = "0.17.3", features = ["file_watcher"] }
bevy_kira_audio = "0.24.0"
rand = "0.10.0"
ron = "0.12.0"
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.149"

[profile.dev]
opt-level = 1

[profile.dev.package."*"]
opt-level = 3

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

### 1.3 非採用技術の理由
- **bevy_rapier2d（物理エンジン）**: 弾幕STGは物理エンジン不要。当たり判定は小さな点（ヒットボックス5×5px）に対する距離計算で実装する。rapierのオーバーヘッドを避けて軽量に保つ。

---

## 2. ECSアーキテクチャ設計

### 2.1 Entity（エンティティ）一覧

主要エンティティ：
- **Player**: プレイヤーキャラクター（1体）
- **PlayerBullet**: プレイヤーの弾（多数）
- **Enemy**: 通常敵エンティティ（数十体）
- **EnemyBullet**: 敵弾（数百〜数千体）
- **Boss**: ボスエンティティ（1体）
- **Item**: アイテム（パワー・ポイント・ボム・1UP・スター）
- **StageScroll**: 背景スクロール
- **Camera**: 固定カメラ（プレイフィールド中央）
- **UI Elements**: HUD・メニュー等

### 2.2 Component（コンポーネント）設計

#### 2.2.1 プレイヤー関連

```rust
/// プレイヤーマーカー
#[derive(Component)]
pub struct Player;

/// プレイヤーの状態
#[derive(Component)]
pub struct PlayerState {
    pub character: CharacterType,
    pub shot_type: ShotType,
    pub is_focused: bool,       // Shift押下中
    pub invincibility_timer: f32,  // 無敵時間（秒）
    pub respawn_timer: f32,        // リスポーン演出中（秒）
}

/// プレイヤーの弾関連
#[derive(Component)]
pub struct ShootTimer {
    pub timer: f32,
    pub interval: f32,
}

/// グレイズ用の判定領域コンポーネント
/// （ヒットボックスより大きな円形領域）
#[derive(Component)]
pub struct GrazeBox {
    pub radius: f32,
}
```

#### 2.2.2 プレイヤー弾関連

```rust
/// プレイヤーの弾
#[derive(Component)]
pub struct PlayerBullet {
    pub damage: f32,
    pub velocity: Vec2,
}
```

#### 2.2.3 敵関連

```rust
/// 敵タイプ
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum EnemyKind {
    SmallFairy,
    LargeFairy,
    Bat,
    // ... その他
}

/// 敵の基本パラメータ
#[derive(Component)]
pub struct Enemy {
    pub kind: EnemyKind,
    pub max_hp: f32,
    pub current_hp: f32,
    pub score: u32,
    pub drop_table: DropTable,
}

/// 敵の移動パターン
#[derive(Component)]
pub struct EnemyMovement {
    pub pattern: MovementPattern,
    pub phase_timer: f32,
}

#[derive(Clone)]
pub enum MovementPattern {
    /// 直線移動
    Linear { velocity: Vec2 },
    /// 曲線移動（制御点付き）
    Bezier { points: Vec<Vec2>, t: f32, speed: f32 },
    /// 固定位置
    Static,
}
```

#### 2.2.4 ボス関連

```rust
/// ボスマーカー
#[derive(Component)]
pub struct Boss;

/// ボスフェーズ管理
#[derive(Component)]
pub struct BossPhase {
    pub phases: Vec<BossPhaseData>,
    pub current_phase: usize,
    pub phase_hp: f32,          // 現フェーズのHP
    pub time_limit: f32,        // スペルカードのタイムリミット
    pub time_elapsed: f32,
}

/// 個別フェーズデータ
#[derive(Clone)]
pub struct BossPhaseData {
    pub phase_type: PhaseType,
    pub max_hp: f32,
    pub time_limit: Option<f32>,  // Noneは通常攻撃
    pub spell_name: Option<String>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum PhaseType {
    NonSpell,    // 通常攻撃
    SpellCard,   // スペルカード
}
```

#### 2.2.5 弾幕関連

```rust
/// 敵弾（単体）
#[derive(Component)]
pub struct EnemyBullet {
    pub bullet_type: BulletType,
    pub velocity: Vec2,
    pub damage: f32,
    pub grazeable: bool,
    pub grazed: bool,           // 既にグレイズ済みか
}

/// レーザー弾（継続グレイズ可能）
#[derive(Component)]
pub struct LaserBullet {
    pub width: f32,
    pub length: f32,
    pub angle: f32,
    pub damage_per_frame: f32,
    pub graze_cooldown: f32,    // 同一フレームのグレイズ防止
}

/// 弾の生存時間
#[derive(Component)]
pub struct Lifetime {
    pub remaining: f32,
}

/// 弾発射元（ターゲティング用）
#[derive(Component)]
pub struct BulletEmitter {
    pub pattern: BulletPattern,
    pub fire_timer: f32,
    pub fire_interval: f32,
    pub active: bool,
}
```

#### 2.2.6 アイテム関連

```rust
/// アイテム種別
#[derive(Component, Clone, Copy)]
pub enum ItemKind {
    PowerSmall,   // パワーアイテム（小）
    PowerLarge,   // パワーアイテム（大）
    Point,        // ポイントアイテム
    Bomb,         // ボムアイテム
    OneUp,        // 1UP
    Star,         // スター（弾消し後）
}

/// アイテムの物理挙動
#[derive(Component)]
pub struct ItemPhysics {
    pub velocity: Vec2,
    pub gravity: f32,
    pub auto_collect: bool,   // 自動収集中フラグ
}
```

#### 2.2.7 汎用コンポーネント

```rust
/// 円形コライダー（衝突・グレイズ判定用）
#[derive(Component)]
pub struct CircleCollider {
    pub radius: f32,
}

/// プレイフィールドの外に出たエンティティを自動削除
#[derive(Component)]
pub struct DespawnOutOfBounds;

/// 状態スコープ（状態離脱時に自動削除）
/// Bevy 0.17では DespawnOnExit<S> を使用
```

### 2.3 Resource（リソース）設計

```rust
/// ゲームの進行状態
#[derive(Resource)]
pub struct GameData {
    pub score: u64,
    pub hi_score: u64,
    pub lives: u8,              // 残機
    pub bombs: u8,              // ボムストック
    pub power: u8,              // パワー値 (0〜128)
    pub graze: u32,             // グレイズカウンター
    pub point_value: u32,       // 現在のポイントアイテム最大値
    pub difficulty: Difficulty,
    pub character: CharacterType,
    pub shot_type: ShotType,
    pub stage: u8,
    pub continues_used: u32,
}

/// エクステンドスコア閾値管理
#[derive(Resource)]
pub struct ExtendData {
    pub next_extend_threshold: u64,  // 次のエクステンドスコア
}

/// ボム使用状態（カウンターボム判定）
#[derive(Resource)]
pub struct BombState {
    pub counter_bomb_window: f32,  // 被弾後のカウンターボム受付時間
    pub is_active: bool,
    pub bomb_timer: f32,
}

/// ステージ管理
#[derive(Resource)]
pub struct StageData {
    pub wave_index: usize,
    pub wave_timer: f32,
    pub boss_active: bool,
    pub scroll_speed: f32,
}

/// ハイスコアデータ（永続化対象）
#[derive(Resource, serde::Serialize, serde::Deserialize)]
pub struct HiScoreData {
    pub scores: HashMap<(Difficulty, CharacterType, ShotType), u64>,
}
```

### 2.4 System（システム）設計

#### 2.4.1 システムカテゴリと実行タイミング

**Startupシステム:**
- `setup_camera`: 固定カメラのセットアップ（`scarlet-ui` が担当）
- `load_assets`: アセット読み込み

**Updateシステム（Playing状態のみ）:**

1. **入力処理**
   - `handle_player_input`: WASD/矢印+Shift+Z+X入力処理

2. **プレイヤーシステム**
   - `update_player_movement`: プレイヤーの移動・境界クランプ
   - `update_player_shoot`: 弾の発射タイミング
   - `spawn_player_bullets`: プレイヤー弾の生成
   - `update_player_invincibility`: 無敵時間の更新

3. **弾幕システム**
   - `update_bullet_emitters`: 弾発射パターンのスケジューリング
   - `move_enemy_bullets`: 敵弾の移動
   - `move_player_bullets`: プレイヤー弾の移動
   - `despawn_expired_bullets`: 寿命切れ弾の削除
   - `despawn_out_of_bounds`: フィールド外弾の削除

4. **敵システム**
   - `spawn_enemies`: ウェーブスクリプトに基づく敵スポーン
   - `update_enemy_movement`: 敵の移動パターン処理
   - `update_boss_phase`: ボスフェーズ管理

5. **衝突判定システム**
   - `player_bullet_vs_enemy`: プレイヤー弾 vs 敵
   - `enemy_bullet_vs_player`: 敵弾 vs プレイヤーヒットボックス
   - `graze_detection`: グレイズ判定（敵弾がグレイズ判定領域内通過）
   - `player_vs_items`: プレイヤー vs アイテム収集

6. **ゲームロジック**
   - `update_power`: パワー値管理
   - `update_score`: スコア計算
   - `check_extend`: エクステンド判定
   - `update_items`: アイテム物理挙動
   - `auto_collect_items`: MAX時の全自動収集
   - `handle_player_death`: ミス処理
   - `handle_bomb`: ボム使用処理

7. **ステージ**
   - `update_stage_scroll`: 背景スクロール
   - `advance_wave`: ウェーブ進行

8. **UI / HUD**（`scarlet-ui` が担当）
   - `update_hud`: HUD表示更新

9. **エフェクト**
   - `update_damage_effects`: 被ダメージ点滅
   - `update_bomb_effect`: ボムエフェクト
   - `update_death_effects`: 敵撃破エフェクト

#### 2.4.2 システム実行順序

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
    UI,
    Effects,
    Cleanup,
}

app.configure_sets(
    Update,
    (
        GameSystemSet::Input,
        GameSystemSet::PlayerLogic,
        GameSystemSet::BulletEmit,
        GameSystemSet::Movement,
        GameSystemSet::Collision,
        GameSystemSet::GameLogic,
        GameSystemSet::StageControl,
        GameSystemSet::UI,
        GameSystemSet::Effects,
        GameSystemSet::Cleanup,
    ).chain().run_if(in_state(AppState::Playing))
);
```

### 2.5 State（ゲーム状態）管理

```rust
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Title,
    CharacterSelect,   // キャラクター・ショットタイプ選択
    DifficultySelect,  // 難易度選択
    Loading,           // ステージ読み込み
    Playing,           // ゲームプレイ中
    Paused,            // ポーズ中
    StageClear,        // ステージクリア演出
    GameOver,          // ゲームオーバー
    Ending,            // エンディング
}
```

**状態遷移:**
```
Title ──→ CharacterSelect ──→ DifficultySelect ──→ Loading ──→ Playing
                                                               │  ↕ Esc
                                                             Paused
                                                               │
                                                        ┌──────┼──────┐
                                                    StageClear  │  GameOver
                                                        │    (続行)     │
                                                      (次ステージ)   Title
                                                        │
                                                      Ending
```

---

## 3. 衝突判定設計

### 3.1 手動円形衝突判定

物理エンジン不使用。シンプルな距離計算を使用する：

```rust
/// 2点間の衝突チェック（平方根を避けた最適化版）
pub fn check_circle_collision(
    pos1: Vec2,
    radius1: f32,
    pos2: Vec2,
    radius2: f32,
) -> bool {
    let dist_sq = pos1.distance_squared(pos2);
    let radius_sum = radius1 + radius2;
    dist_sq < radius_sum * radius_sum
}
```

### 3.2 判定サイズ設計

| エンティティ | ヒットボックス半径 | グレイズ半径 |
|------------|-----------------|------------|
| プレイヤー | 2.5 px（5×5の半分） | 16 px |
| 敵弾（小丸弾） | 4 px | — |
| 敵弾（中丸弾） | 6 px | — |
| 敵弾（大丸弾） | 10 px | — |
| 敵弾（ナイフ） | 3 px | — |
| レーザー | 幅 × 長さの矩形 | — |
| アイテム収集 | 8 px | — |

### 3.3 グレイズ判定

- プレイヤーのグレイズ半径（16px）と敵弾が重なった時にグレイズ発生
- ヒットボックス（2.5px）との衝突は別途チェック
- グレイズ判定は `grazed: false` の弾に対してのみ発動
- レーザーは毎フレームグレイズ可能（`graze_cooldown` で頻度制御）

---

## 4. カメラシステム

カメラ関連は `scarlet-ui` クレートが担当。

```rust
// app/ui/src/camera.rs
pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Transform::from_xyz(0.0, 0.0, 999.9),
    ));
}
```

- 固定カメラ（スクロールはせず、背景スプライトが移動する）
- プレイフィールド：画面左半分（384×448 px相当）
- サイドバー：画面右半分（HUD表示）

---

## 5. イベント駆動設計

### 5.1 カスタムイベント一覧

```rust
/// プレイヤーが被弾した
#[derive(Event)]
pub struct PlayerHitEvent {
    pub position: Vec2,
    pub counter_bomb_window: bool,  // カウンターボム可能か
}

/// グレイズが発生した
#[derive(Event)]
pub struct GrazeEvent {
    pub bullet_entity: Entity,
    pub position: Vec2,
    pub score_gain: u32,
}

/// 敵が撃破された
#[derive(Event)]
pub struct EnemyDefeatedEvent {
    pub entity: Entity,
    pub position: Vec2,
    pub score: u32,
    pub drop_table: DropTable,
}

/// ボスフェーズが変化した
#[derive(Event)]
pub struct BossPhaseChangedEvent {
    pub phase_index: usize,
    pub phase_type: PhaseType,
    pub spell_name: Option<String>,
}

/// アイテムを収集した
#[derive(Event)]
pub struct ItemCollectedEvent {
    pub kind: ItemKind,
    pub position: Vec2,
}

/// ボムが使用された
#[derive(Event)]
pub struct BombUsedEvent {
    pub is_counter_bomb: bool,
}

/// ステージクリア
#[derive(Event)]
pub struct StageClearedEvent {
    pub stage: u8,
    pub clear_score: u64,
}
```

---

## 6. プラグイン構成

```rust
// app/touhou-project-scarlet/src/main.rs
use bevy::prelude::*;
use scarlet_assets::GameAssetsPlugin;
use scarlet_audio::GameAudioPlugin;
use scarlet_core::GameCorePlugin;
use scarlet_ui::GameUIPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "東方紅魔郷 Clone".into(),
                resolution: (640.0, 480.0).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(GameAssetsPlugin)
        .add_plugins(GameCorePlugin)
        .add_plugins(GameUIPlugin)
        .add_plugins(GameAudioPlugin)
        .run();
}
```

---

## 7. パフォーマンス最適化戦略

### 7.1 弾幕最適化
- 敵弾は最大1000発程度を想定
- `Lifetime` コンポーネントで毎フレーム寿命を減らし、0以下でDespawn
- `DespawnOutOfBounds` で画面外に出た弾を即削除
- 空間グリッドパーティショニング（敵弾 vs プレイヤーの最適化）

### 7.2 ECSベストプラクティス
- クエリフィルタの活用（`With<EnemyBullet>`, `Without<Player>`）
- `Changed<T>` でHUD更新の頻度削減
- 弾・エフェクトは即座にDespawn（メモリリーク防止）

### 7.3 レンダリング最適化
- スプライトバッチング（同テクスチャをまとめて描画）
- Zオーダー管理（背景 < アイテム < 敵 < プレイヤー弾 < 敵弾 < UI）

---

## 8. テスト戦略

### 8.1 単体テスト
- 弾幕パターンの弾数・角度計算
- スコア計算ロジック
- グレイズ判定ロジック
- パワー段階管理

### 8.2 統合テスト
- プレイヤー弾 vs 敵の衝突
- ボムによる弾消去とアイテム変換

### 8.3 手動テスト
- Stage 1〜6の通しプレイ
- 各難易度での動作確認
- 60fps維持確認

---

**バージョン**: 1.0
**最終更新**: 2026-03-16
**ステータス**: 承認済み
