# Phase 24: Steam統合 + 多言語対応（JP/EN）+ ゲームパッド

## 目標

Steamプラットフォームへの正式対応、日本語・英語の多言語サポート、ゲームパッドの完全サポートを実装する。

## 完了条件

### Steam統合
- [ ] Steamworks SDK（`steamworks` crate）の組み込み
- [ ] Steam実績の登録・解放（§13.5 の実績を Steam Achievement に対応）
- [ ] Steamスコアボード（難易度・キャラ別リーダーボード）
- [ ] Steam Cloud セーブ（セーブデータ・実績の自動同期）
- [ ] Steam Rich Presence（現在のステージ・スコアを表示）
- [ ] Steam非接続環境でのフォールバック（ゲーム内記録のみ）

### 多言語対応（i18n）
- [ ] 言語ファイル形式の設計（RON または JSON）
- [ ] 日本語テキスト（ダイアログ・UI・実績・スペルカード名）
- [ ] 英語テキスト（同上）
- [ ] フォント切り替え（日本語フォント / 英語フォント）
- [ ] タイトル画面に言語選択（JP / EN）を追加
- [ ] 設定画面から言語を変更可能

### ゲームパッド対応
- [ ] XInput（Xbox互換コントローラー）対応
- [ ] DInput（DirectInput / HID）対応
- [ ] ゲームパッドでメニューを操作できる
- [ ] ゲームパッドで全ゲームプレイが可能
- [ ] ボタン設定（ゲームパッド用）の実装・永続化
- [ ] コントローラー接続/切断の動的検出
- [ ] Steam Input API（任意）

## 完了条件（テスト）

- [ ] Steam未接続時でもゲームが正常起動・動作する
- [ ] JP/EN 言語切り替えで全テキストが正しく表示される
- [ ] ゲームパッドのみでタイトルからクリアまで操作できる
- [ ] `just test` が通る

---

## 実装詳細

### 多言語ファイル構成

```
assets/
└── locales/
    ├── ja.ron   # 日本語
    └── en.ron   # 英語
```

```ron
// assets/locales/ja.ron（抜粋）
(
    ui: {
        "title.start": "ゲームスタート",
        "title.practice": "スペルプラクティス",
        "title.replay": "リプレイ",
        "title.settings": "設定",
    },
    spell_cards: {
        "rumia.night_bird": "夜符「ナイトバード」",
        "rumia.demarcation": "闇符「ディマーケーション」",
        // ...
    },
    achievements: {
        "clear_stage1.name": "Stage 1 クリア",
        "clear_stage1.desc": "Stage 1 を初めてクリアした",
        // ...
    },
)
```

### ゲームパッド入力統合

```rust
// Bevy 0.17 の Gamepad リソースを使用
pub fn gamepad_input_system(
    gamepads: Query<&Gamepad>,
    mut player_input: ResMut<PlayerInput>,
    config: Res<GamepadConfig>,
) {
    for gamepad in &gamepads {
        player_input.left  = gamepad.pressed(config.left_button)
            || gamepad.left_stick().x < -0.3;
        player_input.right = gamepad.pressed(config.right_button)
            || gamepad.left_stick().x > 0.3;
        player_input.up    = gamepad.pressed(config.up_button)
            || gamepad.left_stick().y > 0.3;
        player_input.down  = gamepad.pressed(config.down_button)
            || gamepad.left_stick().y < -0.3;
        player_input.shot  = gamepad.pressed(config.shot_button);
        player_input.bomb  = gamepad.pressed(config.bomb_button);
        player_input.focus = gamepad.pressed(config.focus_button);
    }
}
```

### Steam ビルドの条件コンパイル

```rust
// Cargo.toml
[features]
steam = ["steamworks"]

// main.rs
#[cfg(feature = "steam")]
fn init_steam() -> Option<steamworks::Client> {
    steamworks::Client::init_app(APP_ID).ok().map(|(c, _)| c)
}
```

---

## 参照

- `docs/01_specification.md` § 17 入力・コントローラー
- `docs/01_specification.md` § 19 プラットフォーム・配信
- `docs/01_specification.md` § 15.1 ダイアログシステム（言語設定）
