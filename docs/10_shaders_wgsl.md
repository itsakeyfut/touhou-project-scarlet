# 10 WGSLシェーダー設計

## 概要

本プロジェクトは東方紅魔郷の**現代的2Dピクセルアートリメイク**として、WGSLシェーダーを積極活用する。弾幕・グレイズ・スペルカード・ボムなど各要素に固有のシェーダーエフェクトを適用し、クオリティの高いビジュアルを実現する。

---

## 1. Bevy 0.17 でのシェーダー利用

### 1.1 利用可能な主要アプローチ

| アプローチ | 用途 | 難易度 |
|---|---|---|
| `Material2d` | スプライト/Mesh2dへのカスタムシェーダー | ★★ |
| `PostProcessNode` (RenderGraph) | 全画面ポストプロセス | ★★★ |
| `ComputePipeline` | GPUパーティクル | ★★★★ |
| `ExtractComponent` + カスタムRenderPhase | 高度なバッチ処理 | ★★★★★ |

### 1.2 アセット配置

```
app/touhou-project-scarlet/assets/shaders/
├── bullet_glow.wgsl         # 弾のグロー発光
├── bullet_trail.wgsl        # 弾の残像
├── graze_field.wgsl         # グレイズ電気フィールド
├── hit_flash.wgsl           # 被弾白フラッシュ
├── spell_card_bg.wgsl       # スペルカード背景生成
├── bomb_reimu.wgsl          # 霊夢ボム「封魔陣」
├── bomb_marisa.wgsl         # 魔理沙ボム「マスタースパーク」
├── pixel_outline.wgsl       # ピクセルアウトライン
├── post_bloom.wgsl          # ブルームポストプロセス
├── post_crt.wgsl            # CRTピクセル風ポストプロセス
└── common/
    ├── math.wgsl            # 共通数学関数
    └── noise.wgsl           # ノイズ関数
```

### 1.3 Cargo.toml 追記

```toml
# scarlet-core/Cargo.toml
[features]
default = []
shader-debug = []  # ホットリロード有効化

# ワークスペース
[workspace.dependencies]
bevy = { version = "0.17.3", features = ["file_watcher"] }  # シェーダーホットリロード用
```

---

## 2. Material2d の基本パターン

### 2.1 Material2d の定義（Rust側）

```rust
// app/core/src/shaders/mod.rs

use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::Material2d,
};

/// 弾グロー素材
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct BulletGlowMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
    #[uniform(0)]
    pub glow_intensity: f32,
    #[uniform(0)]
    pub time: f32,
    #[texture(1)]
    #[sampler(2)]
    pub texture: Handle<Image>,
}

impl Material2d for BulletGlowMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/bullet_glow.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend  // 加算合成のため
    }
}
```

### 2.2 プラグイン登録

```rust
// app/core/src/shaders/plugin.rs

pub struct ScarletShadersPlugin;

impl Plugin for ScarletShadersPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(Material2dPlugin::<BulletGlowMaterial>::default())
            .add_plugins(Material2dPlugin::<GrazeMaterial>::default())
            .add_plugins(Material2dPlugin::<HitFlashMaterial>::default())
            .add_plugins(Material2dPlugin::<SpellCardBgMaterial>::default())
            .add_plugins(Material2dPlugin::<BombReimuMaterial>::default())
            .add_plugins(Material2dPlugin::<BombMarisaMaterial>::default())
            .add_plugins(Material2dPlugin::<PixelOutlineMaterial>::default())
            .add_systems(Update, (
                update_bullet_glow_time,
                update_graze_material_time,
                update_spell_card_bg_time,
            ).run_if(in_state(AppState::Playing)));
    }
}
```

---

## 3. 弾グロー (BulletGlow)

弾幕に発光エフェクトを付与する。弾の色に合わせてグローが変化する。

### 3.1 WGSL (`shaders/bullet_glow.wgsl`)

```wgsl
#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct BulletGlowMaterial {
    color: vec4<f32>,
    glow_intensity: f32,
    time: f32,
    _padding: vec2<f32>,
}

@group(2) @binding(0)
var<uniform> material: BulletGlowMaterial;
@group(2) @binding(1)
var base_texture: texture_2d<f32>;
@group(2) @binding(2)
var base_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(base_texture, base_sampler, in.uv);

    // スプライトの中心からの距離でグロー強度を計算
    let center = vec2<f32>(0.5, 0.5);
    let dist = distance(in.uv, center);

    // 時間で脈動するグロー
    let pulse = 1.0 + 0.2 * sin(material.time * 6.0);
    let glow = material.glow_intensity * pulse * max(0.0, 1.0 - dist * 2.0);

    // 元テクスチャ色 + グロー
    let glow_color = material.color * glow;

    // アルファブレンド
    let final_alpha = tex_color.a + glow_color.a * (1.0 - tex_color.a);
    let final_color = (tex_color.rgb * tex_color.a + glow_color.rgb * glow_color.a * (1.0 - tex_color.a)) / max(final_alpha, 0.001);

    return vec4<f32>(final_color, final_alpha);
}
```

### 3.2 Rust側: 弾スポーン時に適用

```rust
pub fn spawn_enemy_bullet_with_shader(
    commands: &mut Commands,
    origin: Vec2,
    velocity: Vec2,
    kind: EnemyBulletKind,
    glow_materials: &mut Assets<BulletGlowMaterial>,
    meshes: &mut Assets<Mesh>,
    assets: &ScarletAssets,
) {
    let color = kind.glow_color();    // LinearRgba に変換
    let mesh = meshes.add(Circle::new(kind.visual_radius()));

    let material = glow_materials.add(BulletGlowMaterial {
        color,
        glow_intensity: 1.5,
        time: 0.0,
        texture: assets.bullet_sheet.clone(),
    });

    commands.spawn((
        EnemyBullet { damage: 1 },
        kind,
        BulletVelocity(velocity),
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::from_translation(origin.extend(1.5)),
        DespawnOutOfBounds,
        DespawnOnExit(AppState::Playing),
    ));
}

/// 毎フレームtime uniformを更新
pub fn update_bullet_glow_time(
    time: Res<Time>,
    mut materials: ResMut<Assets<BulletGlowMaterial>>,
) {
    let t = time.elapsed_secs();
    for (_, mat) in materials.iter_mut() {
        mat.time = t;
    }
}
```

---

## 4. 弾の残像 (BulletTrail)

高速弾（ナイフ、霊夢ショット等）に速度方向の残像を付与する。

### 4.1 WGSL (`shaders/bullet_trail.wgsl`)

```wgsl
#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct BulletTrailMaterial {
    color: vec4<f32>,
    velocity_dir: vec2<f32>,   // 正規化された速度方向
    trail_length: f32,          // 残像の長さ (UV空間)
    time: f32,
}

@group(2) @binding(0)
var<uniform> material: BulletTrailMaterial;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // UV空間で速度方向に沿って残像を描画
    // 速度方向の逆が「後方」
    let trail_dir = -material.velocity_dir;

    // 弾の中心 (UV 0.5, 0.5) から後方へのフェード
    let center = vec2<f32>(0.5, 0.5);
    let to_frag = in.uv - center;

    // 速度方向への射影
    let projection = dot(to_frag, trail_dir);
    let perp = to_frag - projection * trail_dir;

    // 残像のアルファ計算
    let trail_alpha = select(
        0.0,
        max(0.0, 1.0 - length(perp) * 8.0) * max(0.0, projection / material.trail_length),
        projection > 0.0
    );

    // 本体の円形
    let core_dist = length(to_frag);
    let core_alpha = max(0.0, 1.0 - core_dist * 4.0);

    let alpha = max(core_alpha, trail_alpha * 0.6);
    return vec4<f32>(material.color.rgb, material.color.a * alpha);
}
```

---

## 5. グレイズ電気フィールド (GrazeField)

プレイヤーのグレイズ圏を可視化する動的な電気フィールドエフェクト。低速移動中（Shift）に常時表示。

### 5.1 素材定義

```rust
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct GrazeMaterial {
    #[uniform(0)]
    pub time: f32,
    #[uniform(0)]
    pub graze_intensity: f32,  // 0.0〜1.0 (グレイズ発生時にスパイク)
    #[uniform(0)]
    pub slow_mode: u32,        // 低速移動中は1
    #[uniform(0)]
    pub _padding: f32,
}

impl Material2d for GrazeMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/graze_field.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
```

### 5.2 WGSL (`shaders/graze_field.wgsl`)

```wgsl
#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import "shaders/common/noise.wgsl"::hash21

struct GrazeMaterial {
    time: f32,
    graze_intensity: f32,
    slow_mode: u32,
    _padding: f32,
}

@group(2) @binding(0)
var<uniform> material: GrazeMaterial;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let center = vec2<f32>(0.5, 0.5);
    let uv = in.uv - center;
    let dist = length(uv);
    let angle = atan2(uv.y, uv.x);

    // 電気フィールドのリング
    let ring_width = 0.04;
    let ring_r = 0.45;  // グレイズ圏の外縁 (UV正規化)
    let ring_dist = abs(dist - ring_r);
    let ring = max(0.0, 1.0 - ring_dist / ring_width);

    // ノイズで電気的なジャグジャグ感
    let noise_angle = angle + material.time * 3.0;
    let n = hash21(vec2<f32>(noise_angle * 5.0, material.time));
    let jagged_ring = ring * (0.5 + 0.5 * n);

    // グレイズ発生時のスパーク
    let spark = material.graze_intensity * max(0.0, 1.0 - dist * 3.0);

    // 低速移動中のみ表示 (Shiftキー)
    let visibility = select(
        jagged_ring * 0.15,  // 通常: うっすら
        jagged_ring * 0.6 + spark,  // 低速: 明瞭
        material.slow_mode == 1u
    );

    // 青白い電気色
    let color = vec3<f32>(0.3, 0.7, 1.0) + spark * vec3<f32>(1.0, 1.0, 0.5);
    return vec4<f32>(color, visibility);
}
```

### 5.3 プレイヤーエンティティへの適用

```rust
pub fn setup_graze_visual(
    mut commands: Commands,
    player: Query<Entity, Added<Player>>,
    mut graze_materials: ResMut<Assets<GrazeMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let Ok(player_entity) = player.single() else { return };

    // グレイズ圏 (r=16px) に合わせたMesh2d
    let mesh = meshes.add(Circle::new(16.0));
    let material = graze_materials.add(GrazeMaterial {
        time: 0.0,
        graze_intensity: 0.0,
        slow_mode: 0,
        _padding: 0.0,
    });

    let graze_visual = commands.spawn((
        GrazeVisual,
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::from_xyz(0.0, 0.0, 0.5),
    )).id();

    // プレイヤーの子エンティティとして追加
    commands.entity(player_entity).add_child(graze_visual);
}

pub fn update_graze_material(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut materials: ResMut<Assets<GrazeMaterial>>,
    graze_visuals: Query<&MeshMaterial2d<GrazeMaterial>, With<GrazeVisual>>,
    mut graze_events: EventReader<GrazeEvent>,
) {
    let t = time.elapsed_secs();
    let slow = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    let grazed = graze_events.read().count() > 0;

    for handle in &graze_visuals {
        let Some(mat) = materials.get_mut(handle) else { continue };
        mat.time = t;
        mat.slow_mode = if slow { 1 } else { 0 };
        if grazed {
            mat.graze_intensity = 1.0;
        } else {
            mat.graze_intensity = (mat.graze_intensity - time.delta_secs() * 5.0).max(0.0);
        }
    }
}
```

---

## 6. 被弾フラッシュ (HitFlash)

プレイヤーやボスが被弾した瞬間に白くフラッシュするシェーダー。

### 6.1 WGSL (`shaders/hit_flash.wgsl`)

```wgsl
#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct HitFlashMaterial {
    flash_intensity: f32,  // 0.0 (通常) 〜 1.0 (完全白)
    color_tint: vec4<f32>, // キャラクターのベース色
    _padding: vec2<f32>,
}

@group(2) @binding(0)
var<uniform> material: HitFlashMaterial;
@group(2) @binding(1)
var base_texture: texture_2d<f32>;
@group(2) @binding(2)
var base_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var tex = textureSample(base_texture, base_sampler, in.uv);
    if tex.a < 0.01 { discard; }

    // フラッシュ: テクスチャ色を白に向かってブレンド
    let white = vec4<f32>(1.0, 1.0, 1.0, tex.a);
    return mix(tex, white, material.flash_intensity);
}
```

### 6.2 Rust側

```rust
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct HitFlashMaterial {
    #[uniform(0)]
    pub flash_intensity: f32,
    #[uniform(0)]
    pub color_tint: LinearRgba,
    #[texture(1)]
    #[sampler(2)]
    pub texture: Handle<Image>,
}

impl Material2d for HitFlashMaterial {
    fn fragment_shader() -> ShaderRef { "shaders/hit_flash.wgsl".into() }
    fn alpha_mode(&self) -> AlphaMode2d { AlphaMode2d::Blend }
}

/// 被弾時にフラッシュ開始
pub fn trigger_hit_flash(
    mut hit_events: EventReader<PlayerHitEvent>,
    player: Query<&MeshMaterial2d<HitFlashMaterial>, With<Player>>,
    mut flash_materials: ResMut<Assets<HitFlashMaterial>>,
) {
    for _ in hit_events.read() {
        for handle in &player {
            if let Some(mat) = flash_materials.get_mut(handle) {
                mat.flash_intensity = 1.0;
            }
        }
    }
}

/// 毎フレームフラッシュをフェードアウト
pub fn update_hit_flash(
    mut flash_materials: ResMut<Assets<HitFlashMaterial>>,
    time: Res<Time>,
) {
    for (_, mat) in flash_materials.iter_mut() {
        mat.flash_intensity = (mat.flash_intensity - time.delta_secs() * 8.0).max(0.0);
    }
}
```

---

## 7. スペルカード背景 (SpellCardBg)

各スペルカードに対応した動的なプロシージャル背景を生成する。

### 7.1 ボス別背景パターン

| ボス | パターン | 色 |
|---|---|---|
| ルーミア | 暗闇の渦 (Swirl) | 暗紫・黒 |
| チルノ | 雪の結晶 (Snowflake) | 水色・白 |
| 美鈴 | 五行色の波紋 (Ripple) | 五色 |
| パチュリー | 魔法陣 (Sigil) | 深紫・金 |
| 咲夜 | 時計歯車 (Clock) | 銀・白 |
| レミリア | 紅い蝙蝠 (Bat) | 深紅・黒 |
| フランドール | 万華鏡 (Kaleidoscope) | 虹色 |

### 7.2 WGSL (`shaders/spell_card_bg.wgsl`)

```wgsl
#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import "shaders/common/math.wgsl"::{rotate2d, TAU}
#import "shaders/common/noise.wgsl"::{fbm}

struct SpellCardBgMaterial {
    time: f32,
    pattern_id: u32,     // 0=swirl, 1=snowflake, 2=ripple, 3=sigil, 4=clock, 5=bat, 6=kaleidoscope
    primary_color: vec4<f32>,
    secondary_color: vec4<f32>,
    intensity: f32,
    _padding: vec3<f32>,
}

@group(2) @binding(0)
var<uniform> material: SpellCardBgMaterial;

fn pattern_swirl(uv: vec2<f32>, t: f32) -> f32 {
    let center = uv - 0.5;
    let angle = atan2(center.y, center.x) + t * 0.5;
    let dist = length(center);
    let spiral = fract(angle / TAU + dist * 3.0 - t * 0.3);
    return smoothstep(0.4, 0.6, spiral) * max(0.0, 1.0 - dist * 1.8);
}

fn pattern_snowflake(uv: vec2<f32>, t: f32) -> f32 {
    let center = uv - 0.5;
    let dist = length(center);
    let angle = atan2(center.y, center.x);
    // 6回対称
    let sym_angle = fract(angle / TAU * 6.0) * TAU / 6.0;
    let petal = abs(cos(sym_angle * 3.0)) * max(0.0, 1.0 - dist * 2.5);
    return petal * (0.5 + 0.5 * sin(dist * 20.0 - t * 4.0));
}

fn pattern_kaleidoscope(uv: vec2<f32>, t: f32) -> f32 {
    let center = uv - 0.5;
    let dist = length(center);
    let angle = atan2(center.y, center.x) + t * 0.2;
    // 8回対称
    let sym = fract(angle / TAU * 8.0) * TAU / 8.0;
    let v = abs(sin(sym * 5.0 + t)) * abs(cos(dist * 15.0 - t * 2.0));
    return v * max(0.0, 1.0 - dist * 1.5);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let t = material.time;

    var pattern_value: f32;
    switch material.pattern_id {
        case 0u: { pattern_value = pattern_swirl(uv, t); }
        case 1u: { pattern_value = pattern_snowflake(uv, t); }
        case 6u: { pattern_value = pattern_kaleidoscope(uv, t); }
        default: { pattern_value = pattern_swirl(uv, t); }
    }

    // FBMノイズとパターンの合成
    let noise = fbm(uv * 4.0 + t * 0.1, 3);
    let combined = clamp(pattern_value + noise * 0.2, 0.0, 1.0) * material.intensity;

    let final_color = mix(material.secondary_color, material.primary_color, combined);
    return vec4<f32>(final_color.rgb, final_color.a * combined);
}
```

---

## 8. ボム演出シェーダー

### 8.1 霊夢ボム「封魔陣」(`shaders/bomb_reimu.wgsl`)

```wgsl
#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import "shaders/common/math.wgsl"::{TAU}

struct BombReimuMaterial {
    time: f32,           // ボム発動からの経過時間
    expand_radius: f32,  // 展開中の半径 (0.0〜1.0)
    _padding: vec2<f32>,
}

@group(2) @binding(0)
var<uniform> material: BombReimuMaterial;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let center = in.uv - 0.5;
    let dist = length(center);
    let angle = atan2(center.y, center.x);
    let t = material.time;

    // 六角形の結界
    let hexagon_angle = fract(angle / TAU * 6.0) * TAU / 6.0;
    let hex_dist = dist / cos(hexagon_angle - floor(hexagon_angle / (TAU / 6.0)) * (TAU / 6.0));

    // 境界リング
    let ring_r = material.expand_radius * 0.5;
    let ring = smoothstep(0.01, 0.0, abs(dist - ring_r) - 0.01);

    // 内部の霊夢紋様（陰陽玉的な模様）
    let inner = select(
        0.0,
        0.3 + 0.2 * sin(angle * 8.0 + t * 3.0) * sin(dist * 10.0 - t),
        dist < ring_r
    );

    let alpha = ring + inner;
    let yin_yang = step(0.5, fract(angle / TAU + t * 0.1));
    let base_color = mix(
        vec3<f32>(1.0, 0.2, 0.3),  // 紅色
        vec3<f32>(1.0, 1.0, 0.9),  // 白色
        yin_yang
    );

    return vec4<f32>(base_color, alpha * (1.0 - dist * 1.2));
}
```

### 8.2 魔理沙ボム「マスタースパーク」(`shaders/bomb_marisa.wgsl`)

```wgsl
#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import "shaders/common/noise.wgsl"::{hash21}

struct BombMarisaMaterial {
    time: f32,
    width: f32,   // スパークの幅 (0〜1 UV)
    _padding: vec2<f32>,
}

@group(2) @binding(0)
var<uniform> material: BombMarisaMaterial;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let t = material.time;

    // X軸: スパークの幅、Y軸: スパークの長さ方向
    let center_x = 0.5;
    let dist_from_center = abs(uv.x - center_x);

    // エッジの揺らぎ
    let edge_noise = hash21(vec2<f32>(uv.y * 20.0, t * 5.0)) * 0.05;
    let half_width = material.width * 0.5 + edge_noise;

    let in_spark = dist_from_center < half_width;

    // スパーク内部の明度（中心が最も明るい）
    let intensity = max(0.0, 1.0 - dist_from_center / half_width);
    let core = pow(intensity, 0.5);

    // 高速ノイズで電気的なテクスチャ
    let spark_noise = hash21(vec2<f32>(uv.x * 30.0 + t * 10.0, uv.y * 5.0));

    // 虹色グラデーション（魔理沙のカラフルなレーザー）
    let hue = uv.y + t * 0.3;
    let r = 0.5 + 0.5 * sin(hue * 6.28 + 0.0);
    let g = 0.5 + 0.5 * sin(hue * 6.28 + 2.09);
    let b = 0.5 + 0.5 * sin(hue * 6.28 + 4.19);
    let rainbow = vec3<f32>(r, g, b);

    // コアは白、外縁は虹色
    let spark_color = mix(rainbow, vec3<f32>(1.0, 1.0, 1.0), core);

    let alpha = select(0.0, core * (0.8 + 0.2 * spark_noise), in_spark);
    return vec4<f32>(spark_color, alpha);
}
```

---

## 9. ピクセルアウトライン (PixelOutline)

ピクセルアートスプライトに1px〜2pxのアウトラインを付与する。スペルカード中のボスや、フォーカス中のプレイヤーに適用。

### 9.1 WGSL (`shaders/pixel_outline.wgsl`)

```wgsl
#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct PixelOutlineMaterial {
    outline_color: vec4<f32>,
    outline_width: f32,   // ピクセル数 (1.0〜3.0)
    texture_size: vec2<f32>,  // テクスチャの実際のサイズ (px)
    _padding: f32,
}

@group(2) @binding(0)
var<uniform> material: PixelOutlineMaterial;
@group(2) @binding(1)
var sprite_texture: texture_2d<f32>;
@group(2) @binding(2)
var sprite_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel_size = 1.0 / material.texture_size;
    let base = textureSample(sprite_texture, sprite_sampler, in.uv);

    // 元ピクセルが不透明なら変更しない
    if base.a > 0.5 {
        return base;
    }

    // 8方向の隣接ピクセルをサンプル (1px アウトライン)
    let w = material.outline_width;
    var outline_alpha = 0.0;
    for (var dx: f32 = -w; dx <= w; dx += 1.0) {
        for (var dy: f32 = -w; dy <= w; dy += 1.0) {
            if dx == 0.0 && dy == 0.0 { continue; }
            let neighbor_uv = in.uv + vec2<f32>(dx, dy) * texel_size;
            let neighbor = textureSample(sprite_texture, sprite_sampler, neighbor_uv);
            outline_alpha = max(outline_alpha, neighbor.a);
        }
    }

    return vec4<f32>(material.outline_color.rgb, material.outline_color.a * outline_alpha);
}
```

---

## 10. ポストプロセス

### 10.1 ブルームポストプロセス (`shaders/post_bloom.wgsl`)

弾幕の発光部分がにじむブルームエフェクト。Bevy の `RenderGraph` を利用。

```wgsl
// ブルーム: 高輝度部分を抽出してガウスブラー後に合成

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0)
var screen_texture: texture_2d<f32>;
@group(0) @binding(1)
var screen_sampler: sampler;

struct BloomSettings {
    threshold: f32,     // ブルーム発生輝度閾値 (0.8〜1.5)
    intensity: f32,     // ブルーム強度
    radius: f32,        // ブルーム半径 (px)
    _padding: f32,
}
@group(0) @binding(2)
var<uniform> settings: BloomSettings;

// 輝度抽出 (Threshold Pass)
@fragment
fn extract_bright(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(screen_texture, screen_sampler, in.uv);
    let luminance = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114));
    let bright = max(0.0, luminance - settings.threshold);
    return vec4<f32>(color.rgb * bright, 1.0);
}

// ガウスブラー + 合成 (Composite Pass)
@fragment
fn composite(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let original = textureSample(screen_texture, screen_sampler, in.uv);

    // 簡略化した5x5ガウスブラー
    let tex_size = vec2<f32>(textureDimensions(screen_texture));
    let step = settings.radius / tex_size;
    var bloom = vec3<f32>(0.0);
    let weights = array<f32, 5>(0.0625, 0.25, 0.375, 0.25, 0.0625);

    for (var i: i32 = -2; i <= 2; i++) {
        for (var j: i32 = -2; j <= 2; j++) {
            let offset = vec2<f32>(f32(i), f32(j)) * step;
            let s = textureSample(screen_texture, screen_sampler, in.uv + offset);
            bloom += s.rgb * weights[i + 2] * weights[j + 2];
        }
    }

    return vec4<f32>(original.rgb + bloom * settings.intensity, original.a);
}
```

### 10.2 ポストプロセスノード実装（Rust）

```rust
// app/core/src/shaders/post_process.rs

use bevy::render::{
    render_graph::{Node, RenderGraphContext},
    renderer::RenderContext,
    view::ExtractedWindows,
};

pub struct BloomNode;

impl Node for BloomNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), bevy::render::render_graph::NodeRunError> {
        // Bevy 0.17 のポストプロセスパイプライン実装
        // 詳細は bevy_core_pipeline の PostProcessingPass を参照
        Ok(())
    }
}
```

> **Note**: Bevy 0.17 では `bevy::core_pipeline::bloom::BloomPlugin` が標準提供されている。まずこれを利用し、カスタム要件がある場合のみカスタムノードを実装する。

```rust
// main.rs での標準ブルーム有効化
Camera2d.bloom_settings = Some(BloomSettings {
    intensity: 0.3,
    ..default()
});
```

### 10.3 CRTピクセル風エフェクト (`shaders/post_crt.wgsl`)

ピクセルアートに合わせたサブピクセルシミュレーション（オプション）。

```wgsl
#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

struct CrtSettings {
    pixel_size: f32,      // 仮想ピクセルの倍率 (1〜4)
    scanline_intensity: f32,
    _padding: vec2<f32>,
}

@group(0) @binding(0) var screen: texture_2d<f32>;
@group(0) @binding(1) var screen_sampler: sampler;
@group(0) @binding(2) var<uniform> settings: CrtSettings;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    // ピクセルをスナップ（ドット感の強調）
    let tex_size = vec2<f32>(textureDimensions(screen));
    let pixel_grid = floor(in.uv * tex_size / settings.pixel_size) * settings.pixel_size / tex_size;
    let color = textureSample(screen, screen_sampler, pixel_grid);

    // スキャンライン
    let scanline = 1.0 - settings.scanline_intensity * 0.5 * (1.0 + sin(in.uv.y * tex_size.y * 3.14159));

    return vec4<f32>(color.rgb * scanline, color.a);
}
```

---

## 11. 共通シェーダーライブラリ

### 11.1 `shaders/common/math.wgsl`

```wgsl
const PI: f32 = 3.14159265358979;
const TAU: f32 = 6.28318530717959;

fn rotate2d(v: vec2<f32>, angle: f32) -> vec2<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return vec2<f32>(v.x * c - v.y * s, v.x * s + v.y * c);
}

fn smoothstep_edge(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}

fn luma(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.299, 0.587, 0.114));
}

// HSV → RGB
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> vec3<f32> {
    let c = v * s;
    let x = c * (1.0 - abs(fract(h * 6.0) * 2.0 - 1.0));
    let m = v - c;
    var rgb: vec3<f32>;
    let hi = u32(h * 6.0) % 6u;
    switch hi {
        case 0u: { rgb = vec3<f32>(c, x, 0.0); }
        case 1u: { rgb = vec3<f32>(x, c, 0.0); }
        case 2u: { rgb = vec3<f32>(0.0, c, x); }
        case 3u: { rgb = vec3<f32>(0.0, x, c); }
        case 4u: { rgb = vec3<f32>(x, 0.0, c); }
        default: { rgb = vec3<f32>(c, 0.0, x); }
    }
    return rgb + m;
}
```

### 11.2 `shaders/common/noise.wgsl`

```wgsl
// ハッシュ関数
fn hash11(n: f32) -> f32 {
    return fract(sin(n) * 43758.5453123);
}

fn hash21(p: vec2<f32>) -> f32 {
    let p3 = fract(vec3<f32>(p.xyx) * 0.1031);
    let p4 = p3 + dot(p3, p3.yzx + 33.33);
    return fract((p4.x + p4.y) * p4.z);
}

fn hash22(p: vec2<f32>) -> vec2<f32> {
    let p3 = fract(vec3<f32>(p.xyx) * vec3<f32>(0.1031, 0.1030, 0.0973));
    let p4 = p3 + dot(p3, p3.yzx + 33.33);
    return fract((p4.xx + p4.yz) * p4.zy);
}

// Value Noise (2D)
fn value_noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);

    return mix(
        mix(hash21(i + vec2<f32>(0.0, 0.0)), hash21(i + vec2<f32>(1.0, 0.0)), u.x),
        mix(hash21(i + vec2<f32>(0.0, 1.0)), hash21(i + vec2<f32>(1.0, 1.0)), u.x),
        u.y
    );
}

// FBM (Fractional Brownian Motion) - 複数オクターブのノイズ
fn fbm(p: vec2<f32>, octaves: i32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;
    var pp = p;
    for (var i = 0; i < octaves; i++) {
        value += amplitude * value_noise(pp * frequency);
        frequency *= 2.0;
        amplitude *= 0.5;
    }
    return value;
}
```

---

## 12. シェーダーホットリロード

Bevy の `file_watcher` feature でWGSLファイルを保存するたびに自動リロードされる。

```toml
# ワークスペースのCargo.toml
[workspace.dependencies]
bevy = { version = "0.17.3", features = ["file_watcher"] }
```

```bash
# ホットリロードを活かした開発フロー
just dev  # RUST_LOG=debug で起動、WGSLを編集すると即座に反映
```

シェーダーエラーはコンソールに出力される:
```
ERROR bevy_render::render_resource::shader: Failed to process shader:
  shaders/bullet_glow.wgsl: line 15, column 5: expected ';'
```

---

## 13. シェーダー実装ロードマップ

| フェーズ | 実装シェーダー | 優先度 |
|---|---|---|
| Phase 4 | `bullet_glow.wgsl`, `bullet_trail.wgsl` | 高 |
| Phase 5 | `graze_field.wgsl` | 高 |
| Phase 8 | `spell_card_bg.wgsl`, `hit_flash.wgsl` | 高 |
| Phase 9 | `bomb_reimu.wgsl`, `bomb_marisa.wgsl` | 高 |
| Phase 12 | `pixel_outline.wgsl` | 中 |
| Phase 18 | `post_bloom.wgsl`, `post_crt.wgsl` | 中 |
| Phase 19 | 全シェーダーのチューニング | 低 |

---

## 14. パフォーマンス注意点

- **Material2d インスタンス数**: 同じシェーダーを使う弾でも`BulletGlowMaterial`のインスタンスが異なるとドローコールが増加する。弾種ごとに1つのマテリアルインスタンスを共有し、`time`のみ更新する。

- **テクスチャサンプリング**: `pixel_outline.wgsl` の8方向サンプルは高コスト。弾幕への適用は避け、ボス・プレイヤーのみに限定する。

- **ポストプロセス**: `post_bloom.wgsl` の全画面ブラーは GPU 負荷が高い。Bevy 標準の `BloomSettings` を優先し、カスタム実装は必要な場合のみ行う。

- **Uniformバッファ更新**: `update_bullet_glow_time` のような全マテリアル更新は O(n)。弾数が多い場合は `InstancedMaterial` への移行を検討する。
