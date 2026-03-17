# 09 クイックリファレンス

## コマンド一覧

```bash
just run          # ゲーム実行 (debug)
just dev          # RUST_LOG=debug で実行 (ゲームログ付き、シェーダーホットリロード有効)
just build        # ワークスペースビルド (debug)
just test         # 全テスト実行
just check        # fmt --check + clippy -D warnings
just fmt          # 全コード自動フォーマット
just clippy       # Clippy -D warnings

# クレート別テスト
just unit-test scarlet-core                      # scarlet-core の全ユニットテスト
just unit-test scarlet-core graze_detection      # 特定テスト
cargo test -p scarlet-core -- --nocapture        # stdout 付きテスト
```

---

## クレートマップ

| クレート | パス | 役割 |
|---|---|---|
| `scarlet-core` | `app/core/` | ゲームロジック全体、ECS コンポーネント/リソース/システム |
| `scarlet-ui` | `app/ui/` | カメラ、全UI画面、HUD |
| `scarlet-audio` | `app/audio/` | BGM/SFX (bevy_kira_audio) |
| `scarlet-assets` | `app/assets/` | スプライト/フォント/音声のロード |
| `touhou-project-scarlet` | `app/touhou-project-scarlet/` | バイナリ: 4つのプラグインを組み合わせる |

アセットは `app/touhou-project-scarlet/assets/` に置く（Bevyはバイナリクレートからパスを解決する）。

---

## AppState 遷移

```
Title → CharacterSelect → DifficultySelect → Loading → Playing
                                                          │  ↕ ESC
                                                          │  Paused
                                                          ├──→ StageClear
                                                          ├──→ GameOver
                                                          └──→ Ending → StaffRoll
```

デフォルト状態: `Title`。Playing状態のシステムを単体テストする場合は `#[default]` を一時的に `Playing` に変更。

---

## よく使う import

```rust
// ゲームロジック全般
use scarlet_core::prelude::*;

// コンポーネント
use scarlet_core::{
    Player, PlayerStats, ShootTimer, InvincibilityTimer,
    Enemy, EnemyKind, Boss, BossPhaseData,
    PlayerBullet, EnemyBullet, EnemyBulletKind, BulletVelocity, BulletEmitter,
    ItemKind, ItemPhysics,
    CircleCollider, DespawnOutOfBounds,
};

// リソース
use scarlet_core::{
    GameData, BombState, StageData, EnemySpawner,
    SpatialGrid, SelectedCharacter, SelectedDifficulty,
    FragmentTracker,
};

// イベント
use scarlet_core::{
    PlayerHitEvent, GrazeEvent, BombUsedEvent,
    EnemyDefeatedEvent, BossPhaseChangedEvent,
    ItemCollectedEvent, ExtendEvent, StageClearedEvent,
    ShootEvent,
};

// 状態
use scarlet_core::{AppState, GameSystemSet};

// オーディオ (scarlet-audio)
use scarlet_audio::{PlayBgmEvent, PlaySfxEvent, BgmTrack, SfxKind};

// アセット (scarlet-assets)
use scarlet_assets::ScarletAssets;
```

---

## 主要定数

```rust
// 当たり判定
const PLAYER_HITBOX_RADIUS: f32 = 2.5;   // プレイヤーヒットボックス半径
const GRAZE_RADIUS: f32 = 16.0;           // グレイズ検知半径

// プレイエリア
const PLAY_AREA_W: f32 = 384.0;
const PLAY_AREA_H: f32 = 448.0;
const PLAY_AREA_HALF_W: f32 = 192.0;
const PLAY_AREA_HALF_H: f32 = 224.0;

// ゲームシステム
const SCORE_LINE_Y: f32 = 192.0;          // アイテム最大値取得ライン
const POI_BASE_VALUE: u32 = 10_000;       // ポイントアイテム最大値
const POI_MIN_VALUE: u32 = 100;           // ポイントアイテム最小値
const GRAZE_SCORE: u64 = 500;             // グレイズ1回のスコア
const MAX_BOMBS: u8 = 3;                  // 最大ボム数
const BOMB_DURATION: f32 = 3.5;          // ボム持続秒数
const COUNTER_BOMB_WINDOW: f32 = 0.1;    // カウンターボム受付時間

// エクステンド閾値
const EXTEND_THRESHOLDS: &[u64] = &[10_000_000, 20_000_000, 40_000_000, 60_000_000];
```

---

## コリジョン判定

```rust
/// 2円の重なり判定 (当たり判定の基本)
pub fn check_circle_collision(pos_a: Vec2, r_a: f32, pos_b: Vec2, r_b: f32) -> bool {
    (pos_a - pos_b).length_squared() < (r_a + r_b).powi(2)
}

/// グレイズ判定 (ヒットボックス外かつグレイズ圏内)
pub fn is_graze(player: Vec2, bullet: Vec2, bullet_r: f32) -> bool {
    let dist = (player - bullet).length();
    let in_graze = dist < GRAZE_RADIUS + bullet_r;
    let not_hit  = dist >= PLAYER_HITBOX_RADIUS + bullet_r;
    in_graze && not_hit
}
```

弾種別コリジョン半径:

| 弾種 | 半径 |
|---|---|
| SmallCard | 3px |
| SmallRound, Knife | 4px |
| Rice | 5px |
| MediumRound, Oval | 7px |
| Star, Amulet | 8px |
| Bubble, Butterfly | 9px |
| LargeRound | 11px |
| Laser (幅/2) | 4px |

---

## Bevy 0.17 API 変更点

```rust
// Query::get_single_mut() は廃止
// ✓ 正: Result を返す
let Ok(player) = query.single_mut() else { return };

// StateScoped<S> は DespawnOnExit<S> の型エイリアス
// ✓ コンストラクタとしては DespawnOnExit を使う
commands.spawn(DespawnOnExit(AppState::Playing));

// テストでの時間進め方
app.world_mut().resource_mut::<Time>().advance_by(Duration::from_secs_f32(1.0 / 60.0));
app.world_mut().run_system_once(my_system).unwrap();

// 統合テストのプラグイン構成
App::new().add_plugins((MinimalPlugins, StatesPlugin, ScarletCorePlugin))

// query_filtered は World を分割して借用
let mut q = app.world_mut().query_filtered::<Entity, With<Foo>>();
let entity = q.single(app.world()).unwrap();
```

---

## Entity ライフサイクル

```rust
// プレイ中スポーンしたエンティティは DespawnOnExit で自動クリーンアップ
commands.spawn((
    MyComponent,
    DespawnOnExit(AppState::Playing),
    // ...
));

// ボスは撃破時に commands.entity(entity).despawn_recursive() で手動消去
// 子エンティティ (HPバー等) も一緒に消える

// 画面外弾は DespawnOutOfBounds コンポーネントで自動消去
commands.spawn((
    EnemyBullet { damage: 1 },
    DespawnOutOfBounds,  // despawn_out_of_bounds_system が処理
));
```

---

## ボスフェーズ追加の手順

1. `app/core/src/systems/boss/bosses/` に新しいファイルを作成
2. `BossPhaseData` のベクタを返す関数を実装:

```rust
pub fn rumia_phases(difficulty: Difficulty) -> Vec<BossPhaseData> {
    let params = DifficultyParams::for_difficulty(difficulty);
    vec![
        BossPhaseData {
            hp: 800.0 * params.enemy_hp_multiplier,
            is_spell_card: false,
            spell_card_name: None,
            time_limit_secs: 30.0 * params.boss_time_multiplier,
            pattern: BulletPattern::AimedRing { count: 8, speed: 120.0 * params.bullet_speed_multiplier },
            movement: BossMovement::Pendulum { amplitude: 100.0, frequency: 0.5 },
            spell_card_bonus: 0,
        },
        BossPhaseData {
            hp: 1200.0 * params.enemy_hp_multiplier,
            is_spell_card: true,
            spell_card_name: Some("闇符「ディマーケイション」".to_string()),
            time_limit_secs: 40.0 * params.boss_time_multiplier,
            pattern: BulletPattern::SpellCard(SpellCardPattern::Dimarcation),
            movement: BossMovement::Circle { radius: 80.0, speed_deg: 45.0 },
            spell_card_bonus: 1_000_000,
        },
        // ...
    ]
}
```

3. `spawn_boss` でこの関数を呼び出す
4. `BossPhaseChangedEvent` ハンドラで演出（BGM切り替え等）を実装

---

## FallbackConstantsの書き方

```rust
// ✓ 各ファイルの先頭に private DEFAULT_* 定数として定義
const DEFAULT_PLAYER_BASE_SPEED: f32 = 200.0;
const DEFAULT_GRAZE_SCORE: u64 = 500;

// ✗ constants.rs を作らない (削除済み)
// ✗ pub const にしない (sibling テストが必要な場合のみ pub(crate))
```

---

## パフォーマンスチェックポイント

弾幕密度が高い場面では以下を確認:

- `despawn_out_of_bounds_system`: 毎フレーム全弾をスキャンするため O(n)
  → `SpatialGrid` は衝突判定のみ使用、範囲外チェックは AABB で高速化
- `graze_detection_system`: `Local<HashSet<Entity>>` でグレイズ済み弾を管理
  → 画面外で消えた弾は自動的にセットから除外
- ボス戦で弾が 500+ になる場合: `SpatialGrid` のセルサイズを調整 (default 64px)

```bash
# フレームレート確認
RUST_LOG=bevy_diagnostic=debug just dev
```

---

## WGSLシェーダー クイックリファレンス

### シェーダー一覧と導入フェーズ

| シェーダー | Rust型 | 導入 |
|---|---|---|
| `bullet_glow.wgsl` | `BulletGlowMaterial` | Phase 4 |
| `bullet_trail.wgsl` | `BulletTrailMaterial` | Phase 4 |
| `graze_field.wgsl` | `GrazeMaterial` | Phase 5 |
| `hit_flash.wgsl` | `HitFlashMaterial` | Phase 8 |
| `spell_card_bg.wgsl` | `SpellCardBgMaterial` | Phase 8 |
| `bomb_reimu.wgsl` | `BombReimuMaterial` | Phase 9 |
| `bomb_marisa.wgsl` | `BombMarisaMaterial` | Phase 9 |
| `pixel_outline.wgsl` | `PixelOutlineMaterial` | Phase 12 |
| `post_bloom.wgsl` | Camera `Bloom` component | Phase 18 |
| `post_crt.wgsl` | PostProcess Node | Phase 18 |

### Material2d の最小実装パターン

```rust
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct MyMaterial {
    #[uniform(0)]
    pub time: f32,
    #[texture(1)]
    #[sampler(2)]
    pub texture: Handle<Image>,
}

impl Material2d for MyMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/my_shader.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode2d { AlphaMode2d::Blend }
}

// Plugin登録
app.add_plugins(Material2dPlugin::<MyMaterial>::default());

// スポーン (Sprite の代わりに Mesh2d + MeshMaterial2d)
commands.spawn((
    Mesh2d(meshes.add(Circle::new(radius))),
    MeshMaterial2d(my_materials.add(MyMaterial { .. })),
    Transform::from_translation(pos.extend(z)),
));
```

### WGSLでの Bevy imports

```wgsl
// 頂点出力（フラグメントシェーダーへの入力）
#import bevy_sprite::mesh2d_vertex_output::VertexOutput

// 全画面ポストプロセス用
#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

// 共通数学・ノイズ（このプロジェクト独自）
#import "shaders/common/math.wgsl"::{TAU, rotate2d, hsv_to_rgb}
#import "shaders/common/noise.wgsl"::{hash21, fbm}
```

### ブルームのためのHDRカラー設定

```rust
// カメラを HDR モードに
Camera { hdr: true, ..default() }

// 弾グロー色: 1.0以上の値でブルームが発光
LinearRgba::new(2.5, 0.3, 0.3, 1.0)  // 赤く発光する弾
LinearRgba::new(0.3, 0.3, 3.0, 1.0)  // 青く発光する弾

// 通常スプライト色 (1.0以下): ブルームなし
Color::srgb(0.8, 0.8, 0.8)
```

### シェーダーホットリロード

WGSLファイルを保存すると即座に反映される（`file_watcher` feature が有効な場合）。エラーはコンソールに表示。

```bash
just dev  # ホットリロード有効で起動
```

詳細: `docs/10_shaders_wgsl.md`
