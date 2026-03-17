# Phase 22: スペルカードコレクション + ミュージックルーム + 実績

## 目標

コレクション要素・音楽鑑賞モード・実績システムを実装し、ゲームの寿命とやり込み要素を大幅に強化する。

## 完了条件

### スペルカードコレクション
- [ ] 全スペルカードの取得状況（◎/○/×）をグリッド表示するUI
- [ ] キャラ・難易度・ステージでのフィルタリング
- [ ] スペルカード名・ボス名・取得日時の表示
- [ ] 全スペルカード◎達成で実績解放

### ミュージックルーム
- [ ] 解放済みBGMをリスト表示・再生
- [ ] BGMモード切り替え（原作アレンジ / チップチューン / オリジナル）
- [ ] 楽曲タイトル・作曲クレジット表示
- [ ] 対応するステージ・シーンの情報表示

### 実績システム
- [ ] ゲーム内実績の定義と実装（カテゴリ別、全30〜50件）
- [ ] 実績一覧画面（アイコン・説明・達成日時）
- [ ] 実績解放時のゲーム内通知（画面右下ポップアップ）
- [ ] セーブデータへの実績進捗の永続化

## 完了条件（テスト）

- [ ] スペルカードを初回キャプチャした時に `SpellCardCollection` が更新される
- [ ] 実績イベントが発火した時に対応する実績が解放される
- [ ] `just test` が通る

---

## 実装詳細

### 実績定義

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Achievement {
    // ストーリー
    ClearStage1,
    ClearStage6,
    ClearExtraStage,
    ClearPhantasm,
    ClearNoContinue,
    ClearNoMiss,
    ClearNoBomb,

    // スコア
    Score10Million,
    Score50Million,
    Score100Million,

    // スペルカード
    FirstCapture,               // 初めてノーミスノーボム撃破
    CaptureAll,                 // 全スペルカードキャプチャ
    CaptureFlandreAll,          // フランドール全スペルキャプチャ
    SpellPractice100Times,      // スペルプラクティス100回

    // コレクション
    UnlockAllBgm,
    CompleteSpellCollection,

    // 技術
    GrazeCount1000,
    CounterBomb,                // カウンターボム成功
    ClearWithReimu,
    ClearWithMarisa,
    ClearWithSakuya,
}

#[derive(Resource, Serialize, Deserialize, Default)]
pub struct AchievementData {
    pub unlocked: HashSet<Achievement>,
    pub progress: HashMap<Achievement, u64>,  // 進捗カウンター
}
```

### 実績イベント

```rust
#[derive(Event)]
pub struct AchievementUnlockedEvent {
    pub achievement: Achievement,
}

// システム：既存のゲームイベントを監視して実績チェック
pub fn check_achievements(
    mut stage_cleared: EventReader<StageClearedEvent>,
    mut score_events: EventReader<ScoreChangedEvent>,
    mut graze_events: EventReader<GrazeEvent>,
    mut achievement_data: ResMut<AchievementData>,
    mut unlock_events: EventWriter<AchievementUnlockedEvent>,
    game_data: Res<GameData>,
) {
    // 各イベントをチェックして実績解放
}
```

### ミュージックルームのBGMモード管理

```rust
#[derive(Resource, Serialize, Deserialize, Default, Clone, Copy, PartialEq)]
pub enum BgmMode {
    #[default]
    OriginalArrange,  // 原作アレンジ
    Chiptune,         // チップチューン
    Original,         // オリジナル楽曲
}

// BgmTrack を BgmMode に応じて切り替え
pub fn resolve_bgm_track(track: BgmTrack, mode: BgmMode) -> &'static str {
    match (track, mode) {
        (BgmTrack::Stage1, BgmMode::OriginalArrange) => "audio/bgm/arrange/stage1.ogg",
        (BgmTrack::Stage1, BgmMode::Chiptune)        => "audio/bgm/chiptune/stage1.ogg",
        (BgmTrack::Stage1, BgmMode::Original)        => "audio/bgm/original/stage1.ogg",
        // ...
    }
}
```

---

## 参照

- `docs/specification.md` § 13.3 スペルカードコレクション
- `docs/specification.md` § 13.4 ミュージックルーム
- `docs/specification.md` § 13.5 実績システム
