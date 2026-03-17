# Phase 23: Phantasm ステージ（独自シナリオ）

## 目標

全キャラクター・全ショットタイプでExtraステージをクリアすることで解放される最高難易度ステージ「Phantasm」を実装する。東方世界観に沿ったオリジナルボスと独自シナリオを持つ。

## 完了条件

- [ ] Phantasm ステージの解放条件実装（全キャラ×全タイプでExtraクリア）
- [ ] Phantasm ステージの背景・ステージスクロール
- [ ] Phantasm 専用ダイアログ（日本語・英語）
- [ ] オリジナルボスキャラクターの実装（仮称：??????）
  - [ ] ノンスペル（通常攻撃）× 3フェーズ
  - [ ] スペルカード × 5枚（Phantasm難易度のみ）
- [ ] スペルカードコレクションにPhantasmカードを追加
- [ ] Phantasm専用エンディング・スタッフロール
- [ ] Phantasm専用BGM（オリジナル楽曲）
- [ ] 実績：「Phantasm撃破」

## 完了条件（テスト）

- [ ] Phantasm解放条件が正しく判定される
- [ ] `just test` が通る

---

## 設計メモ

### ボスデザイン方針

Phantasmボスは以下の方針で設計する：

- **世界観**：東方幻想郷の結界・時空に関わる存在（ZUN氏の世界観を尊重）
- **難易度**：Lunaticを大幅に超える密度・速度
- **特徴**：既存ボスの攻撃を「吸収・模倣」する演出（コレクター的キャラクター）
- **ビジュアル**：複数の前ボスの要素を取り込んだシェーダー演出

### 弾幕パラメータ

```rust
// Phantasm 難易度補正（phase-09.mdのDifficultyParamsに追加）
Difficulty::Phantasm => DifficultyParams {
    bullet_speed_multiplier: 2.2,
    bullet_count_multiplier: 2.0,
    enemy_hp_multiplier: 1.8,
    boss_time_multiplier: 0.7,
    point_item_max_value: 500_000,
},
```

### Phantasm ステージ解放チェック

```rust
#[derive(Resource, Serialize, Deserialize, Default)]
pub struct ProgressData {
    // キー: (character, shot_type)
    pub extra_cleared: HashSet<(CharacterType, ShotType)>,
    // Phantasm解放条件: 全キャラ×全タイプ(6通り)がすべてtrue
}

pub fn is_phantasm_unlocked(progress: &ProgressData) -> bool {
    use CharacterType::*;
    use ShotType::*;
    [(Reimu, A), (Reimu, B), (Marisa, A), (Marisa, B), (Sakuya, A), (Sakuya, B)]
        .iter()
        .all(|key| progress.extra_cleared.contains(key))
}
```

---

## 参照

- `docs/01_specification.md` § 7.2 Phantasm ステージ
- `docs/01_specification.md` § 12.3 Phantasm ステージ仕様
