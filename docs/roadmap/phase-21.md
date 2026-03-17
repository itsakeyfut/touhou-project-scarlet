# Phase 21: スペルプラクティス + リプレイシステム

## 目標

東方シリーズの定番機能である「スペルプラクティスモード」と「リプレイ保存・再生」を実装する。

## 完了条件

### スペルプラクティス
- [ ] スペルカード個別を選択して練習できるモードの実装
- [ ] 解放条件：初回キャプチャ（ノーミス・ノーボム撃破）でアンロック
- [ ] 自機・難易度の自由選択
- [ ] 挑戦回数・成功回数の記録と表示
- [ ] `SpellPractice` AppState と対応するUI画面
- [ ] スペルプラクティス選択画面（ステージ別・ボス別フィルタリング）

### リプレイシステム
- [ ] クリア時・ゲームオーバー時に入力データを自動保存
- [ ] クリア画面・ゲームオーバー画面からの手動保存（最大10件）
- [ ] リプレイファイル形式（`.scarlet-replay`）の定義と実装
- [ ] メインメニューからリプレイを選択して再生
- [ ] リプレイ再生中の早送り（×2、×4）と一時停止
- [ ] リプレイ一覧画面（日時・キャラ・難易度・スコアの表示）

## 完了条件（テスト）

- [ ] スペルプラクティスで特定スペルを選択→開始→終了のフローが動く
- [ ] リプレイを保存して再生すると同じプレイが再現される（決定論的再生）
- [ ] `just test` が通る

---

## 実装詳細

### AppState 追加

```rust
pub enum AppState {
    // ... 既存
    SpellPractice,      // 追加
    SpellPracticePlay,  // スペルプラクティス中（Playing とは別State）
    ReplaySelect,       // 追加
    ReplayPlaying,      // リプレイ再生中
}
```

### リプレイデータ構造

```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct ReplayData {
    pub version: u32,              // 互換性管理
    pub character: SelectedCharacter,
    pub shot_type: ShotType,
    pub difficulty: Difficulty,
    pub rng_seed: u64,             // 乱数シード（再現のため）
    pub frames: Vec<FrameInput>,   // フレームごとの入力データ
    pub final_score: u64,
    pub timestamp: String,         // RFC3339
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct FrameInput {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub shot: bool,
    pub bomb: bool,
    pub focus: bool,
}
```

### 決定論的再生の保証

- `rand` の乱数シードを固定し、同じシードで同じ乱数列を再生
- 浮動小数点演算の再現性確保（`f32` 演算順序を固定）
- フレーム単位の入力を記録するため、フレームレート変動による差異が出ない

### スペルプラクティス解放管理

```rust
#[derive(Resource, Serialize, Deserialize, Default)]
pub struct SpellCardCollection {
    // キー: (character, shot_type, difficulty, boss_id, spell_index)
    pub records: HashMap<SpellCardKey, SpellCardRecord>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct SpellCardRecord {
    pub captured: bool,    // ノーミス・ノーボム撃破 → プラクティス解放
    pub cleared: bool,     // 撃破（ボムあり可）
    pub attempts: u32,
    pub practice_attempts: u32,
    pub practice_successes: u32,
}
```

---

## 参照

- `docs/01_specification.md` § 13.1 スペルプラクティス
- `docs/01_specification.md` § 13.2 リプレイシステム
- `docs/09_quick_reference.md` § AppState 遷移
