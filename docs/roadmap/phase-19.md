# Phase 19: ピクセルアート統合 + 最終調整

## 目標

仮グラフィック（色付き矩形）をオリジナルピクセルアートに差し替え、全シェーダーとピクセルアートを組み合わせた完成版に仕上げる。ピクセルアートとグロー発光を共存させることで、現代的な弾幕STGのビジュアルを実現する。

## 完了条件

- [ ] プレイヤースプライト（霊夢・魔理沙）
- [ ] 全ボスキャラクタースプライト
- [ ] 全ザコ敵スプライト
- [ ] 弾種スプライトシート
- [ ] アイテムスプライト
- [ ] UIアイコン（残機・ボム）
- [ ] タイトルロゴ・背景
- [ ] スプライトアニメーション統合

---

## ピクセルアート仕様

### 1. キャンバスサイズ

| 対象 | スプライトサイズ | アニメーションフレーム数 |
|---|---|---|
| プレイヤー (霊夢・魔理沙) | 32×48 px | 3〜8フレーム |
| ボス (ルーミア〜フランドール) | 64×64 px | 4〜6フレーム |
| ザコ妖精 | 24×24 px | 2〜4フレーム |
| コウモリ | 16×16 px | 2フレーム |
| 弾種シート (全種) | 各16×16 px | 1〜2フレーム |
| アイテム | 各12×12 px | 2フレーム |
| UIアイコン（残機・ボム） | 16×16 px | 1フレーム |

### 2. スプライトシート構成

**`bullets/bullet_sheet.png`** (128×64 px, 16×16 per cell):
```
Row 0: SmallRound (赤, 青, 緑, 黄, 紫, 白, ...)
Row 1: MediumRound
Row 2: LargeRound
Row 3: Rice, Knife, Star, Bubble, Amulet, Oval
```

**`items/items.png`** (48×48 px, 16×16 per cell):
```
(0,0): PowerSmall - 赤い小星
(1,0): PowerLarge - 赤い大星
(2,0): PointItem  - 青いP
(0,1): LifeFragment
(1,1): BombFragment
(2,1): FullPower - 赤いP
```

---

## タスク詳細

### 1. `ScarletAssets` 拡張

```rust
// app/assets/src/lib.rs

#[derive(Resource, Default)]
pub struct ScarletAssets {
    // スプライト
    pub player_reimu: Handle<Image>,
    pub player_reimu_layout: Handle<TextureAtlasLayout>,
    pub player_marisa: Handle<Image>,
    pub player_marisa_layout: Handle<TextureAtlasLayout>,

    // ボス
    pub boss_rumia: Handle<Image>,
    pub boss_rumia_layout: Handle<TextureAtlasLayout>,
    pub boss_cirno: Handle<Image>,
    pub boss_meiling: Handle<Image>,
    pub boss_patchouli: Handle<Image>,
    pub boss_sakuya: Handle<Image>,
    pub boss_remilia: Handle<Image>,
    pub boss_flandre: Handle<Image>,

    // 敵
    pub enemy_fairy: Handle<Image>,
    pub enemy_bat: Handle<Image>,

    // 弾
    pub bullet_sheet: Handle<Image>,
    pub bullet_layout: Handle<TextureAtlasLayout>,

    // アイテム
    pub item_sheet: Handle<Image>,
    pub item_layout: Handle<TextureAtlasLayout>,

    // エフェクト
    pub effect_explosion: Handle<Image>,
    pub effect_explosion_layout: Handle<TextureAtlasLayout>,
    pub effect_graze_spark: Handle<Image>,

    // UI
    pub title_logo: Handle<Image>,
    pub ui_life_icon: Handle<Image>,
    pub ui_bomb_icon: Handle<Image>,
    pub bg_stage: [Handle<Image>; 6],  // ステージ背景

    // フォント
    pub font_main: Handle<Font>,
    pub font_japanese: Handle<Font>,
    pub font_score: Handle<Font>,
}
```

### 2. スプライトアトラス登録

```rust
// app/assets/src/loader.rs

pub fn load_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut loading: ResMut<AssetsLoading>,
) {
    let mut assets = ScarletAssets::default();

    // プレイヤー（16フレームx2キャラクター）
    assets.player_reimu = asset_server.load("sprites/player/reimu.png");
    assets.player_reimu_layout = texture_atlases.add(
        TextureAtlasLayout::from_grid(UVec2::new(32, 48), 8, 2, None, None)
    );

    // 弾スプライトシート（8列×4行）
    assets.bullet_sheet = asset_server.load("sprites/bullets/bullet_sheet.png");
    assets.bullet_layout = texture_atlases.add(
        TextureAtlasLayout::from_grid(UVec2::new(16, 16), 8, 4, None, None)
    );

    loading.handles.push(assets.player_reimu.clone().untyped());
    loading.handles.push(assets.bullet_sheet.clone().untyped());
    // ... 全アセット

    commands.insert_resource(assets);
}
```

### 3. TextureAtlas への切り替え

仮グラフィック（Sprite + color）からTextureAtlasへの切り替え:

```rust
// 仮 → 本番 の変換例

// 仮:
commands.spawn((
    Sprite {
        color: Color::srgb(1.0, 0.5, 0.5),
        custom_size: Some(Vec2::splat(16.0)),
        ..default()
    },
    // ...
));

// 本番:
commands.spawn((
    Sprite {
        image: assets.player_reimu.clone(),
        texture_atlas: Some(TextureAtlas {
            layout: assets.player_reimu_layout.clone(),
            index: 0,
        }),
        ..default()
    },
    // ...
));
```

### 4. アニメーションシステム

```rust
// app/core/src/systems/effects/particles.rs に追加

#[derive(Component)]
pub struct SpriteAnimation {
    pub frames: Vec<usize>,      // アトラスのインデックス列
    pub current: usize,
    pub timer: Timer,
    pub looping: bool,
}

pub fn sprite_animation_system(
    mut commands: Commands,
    mut animated: Query<(Entity, &mut Sprite, &mut SpriteAnimation)>,
    time: Res<Time>,
) {
    for (entity, mut sprite, mut anim) in &mut animated {
        if !anim.timer.tick(time.delta()).just_finished() { continue; }

        anim.current += 1;
        if anim.current >= anim.frames.len() {
            if anim.looping {
                anim.current = 0;
            } else {
                commands.entity(entity).despawn();
                continue;
            }
        }

        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = anim.frames[anim.current];
        }
    }
}
```

### 5. プレイヤーアニメーション

```rust
// 左移動: フレーム 0〜3
// 正面: フレーム 4〜5
// 右移動: フレーム 6〜9
// 低速移動: フレーム 10〜13 (ヒットボックス表示)

pub fn update_player_animation(
    keys: Res<ButtonInput<KeyCode>>,
    mut players: Query<(&mut Sprite, &mut SpriteAnimation), With<Player>>,
) {
    let Ok((mut sprite, mut anim)) = players.single_mut() else { return };

    let moving_left  = keys.pressed(KeyCode::ArrowLeft)  || keys.pressed(KeyCode::KeyA);
    let moving_right = keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD);
    let slow = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

    let new_frames = match (moving_left, moving_right, slow) {
        (true, _, false)  => vec![0, 1, 2, 3],   // 左移動
        (_, true, false)  => vec![6, 7, 8, 9],   // 右移動
        (_, _, true)      => vec![10, 11, 12, 13], // 低速
        _                 => vec![4, 5],          // 正面
    };

    if anim.frames != new_frames {
        anim.frames = new_frames;
        anim.current = 0;
    }
}
```

### 6. `ImagePlugin::default_nearest()` の確認

ピクセルアートの描画には nearest-neighbor フィルタリングが必須。`main.rs` で設定済み:

```rust
DefaultPlugins.set(ImagePlugin::default_nearest())
```

### シェーダーとピクセルアートの共存

ピクセルアートにシェーダーを組み合わせる際の重要な設定:

```rust
// 最重要: nearest-neighbor フィルタリング
DefaultPlugins.set(ImagePlugin::default_nearest())
```

**弾**: `BulletGlowMaterial` の `texture` フィールドにスプライトシートを設定すると、ピクセルアート弾にグロー発光が乗る。

**プレイヤー/ボス**: `HitFlashMaterial` と `PixelOutlineMaterial` を使用。スプライトを `texture` として渡す。

**背景**: スペルカード中は `SpellCardBgMaterial` がプレイエリアを動的背景で覆い、ピクセルアート背景の上に半透明で重なる。

**ブルーム**: HDRモードでカメラをセットアップし、弾の `glow_color` を1.0以上のHDR値にすることで、ピクセルアートスプライトの周囲が自然に発光する。

---

## アセット制作ノート

> **著作権について**: 東方projectのキャラクターデザインは上海アリス幻樂団に帰属する。
> 本クローンのグラフィックはオリジナル制作とすること。
> 東方projectの二次創作ガイドラインに従う場合は、非営利かつクレジット明記を行うこと。

制作ツール例:
- [Aseprite](https://www.aseprite.org/) — ピクセルアート専用エディタ
- [LibreSprite](https://libresprite.github.io/) — Aseprite のオープンソースフォーク
- [GIMP](https://www.gimp.org/) — 汎用画像編集

スプライトシートの書き出し: Aseprite の "Export Sprite Sheet" で行を揃えて書き出し。

---

## 参照

- `docs/07_project_structure.md` § アセットディレクトリ構造
- Bevy ドキュメント: TextureAtlas, SpriteAnimation
