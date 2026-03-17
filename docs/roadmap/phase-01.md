# Phase 01: プロジェクトセットアップ

## 目標

ビルドが通り、640×480の黒いウィンドウが表示される状態を作る。

## 完了条件

- [ ] `just run` で 640×480 のウィンドウが開く
- [ ] `just check` (fmt + clippy) がエラーなし
- [ ] `just test` が通る

---

## タスク詳細

### 1. ワークスペース確認

`Cargo.toml` に以下の5クレートが登録されていること:
- `app/core` (scarlet-core)
- `app/ui` (scarlet-ui)
- `app/audio` (scarlet-audio)
- `app/assets` (scarlet-assets)
- `app/touhou-project-scarlet` (バイナリ)

### 2. 各クレートの `Cargo.toml` 作成

**`app/core/Cargo.toml`**:
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
```

**`app/ui/Cargo.toml`**:
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

**`app/audio/Cargo.toml`**:
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

**`app/assets/Cargo.toml`**:
```toml
[package]
name = "scarlet-assets"
version = "0.1.0"
edition = "2021"

[dependencies]
bevy = { workspace = true }
```

**`app/touhou-project-scarlet/Cargo.toml`**:
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

### 3. `AppState` 定義

`app/core/src/states.rs`:
```rust
use bevy::prelude::*;

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

### 4. スタブプラグイン作成

各クレートに最小限の `lib.rs` を作成（空のプラグイン）。

**`app/core/src/lib.rs`**:
```rust
use bevy::prelude::*;
pub mod states;
pub use states::AppState;

pub struct ScarletCorePlugin;
impl Plugin for ScarletCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>();
    }
}
```

### 5. `main.rs` 作成

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
                .set(ImagePlugin::default_nearest()),
            ScarletAssetsPlugin,
            ScarletCorePlugin,
            ScarletUiPlugin,
            ScarletAudioPlugin,
        ))
        .run();
}
```

### 6. justfile 確認

```justfile
run:
    cargo run -p touhou-project-scarlet

dev:
    RUST_LOG=debug cargo run -p touhou-project-scarlet

build:
    cargo build

test:
    cargo test

check:
    cargo fmt --check
    cargo clippy -- -D warnings

fmt:
    cargo fmt

clippy:
    cargo clippy -- -D warnings

unit-test crate *args:
    cargo test -p {{crate}} -- {{args}}
```

### 7. アセットディレクトリ作成

```
app/touhou-project-scarlet/assets/
├── audio/bgm/   (空)
├── audio/sfx/   (空)
├── sprites/     (空)
├── fonts/       (空)
└── config/      (空)
```

---

## 参照

- `docs/references/vampire-survivors/CLAUDE.md` — 同じ5クレート構成の参照
- `docs/08_crate_architecture.md` — クレート依存グラフ
