# Phase 10: Stage 1 完全実装（ルーミア）

## 目標

Stage 1 をエンドtoエンドで通しプレイできる。ルーミアの全スペルカードが実装される。60fps維持確認。

## 完了条件

- [ ] ルーミア全スペルカード4種実装
- [ ] Stage 1 全ザコ波スクリプト完成
- [ ] Stage 1 BGM統合（仮音源）
- [ ] ステージクリア後のスコア集計（暫定）
- [ ] 60fps でのパフォーマンス確認

---

## タスク詳細

### 1. ルーミア全スペルカード

```rust
// app/core/src/systems/boss/bosses/rumia.rs

pub fn rumia_phases_normal() -> Vec<BossPhaseData> {
    vec![
        // 通常攻撃 1: 自機狙い3-way
        BossPhaseData {
            hp: 600.0,
            hp_max: 600.0,
            is_spell_card: false,
            spell_card_name: None,
            time_limit_secs: 30.0,
            pattern: BulletPattern::Aimed { count: 3, spread_deg: 20.0, speed: 100.0 },
            movement: BossMovement::Pendulum { amplitude: 80.0, frequency: 0.4, base_x: 0.0 },
            spell_card_bonus: 0,
        },
        // スペルカード 1: 「闇符「ディマーケイション」」
        // 円形弾幕を発射し、360度に広がる
        BossPhaseData {
            hp: 800.0,
            hp_max: 800.0,
            is_spell_card: true,
            spell_card_name: Some("闇符「ディマーケイション」".to_string()),
            time_limit_secs: 40.0,
            pattern: BulletPattern::Ring { count: 16, speed: 90.0 },
            movement: BossMovement::Circle { radius: 60.0, speed_deg: 30.0, center: Vec2::new(0.0, 80.0) },
            spell_card_bonus: 1_000_000,
        },
        // 通常攻撃 2: 螺旋
        BossPhaseData {
            hp: 700.0,
            hp_max: 700.0,
            is_spell_card: false,
            spell_card_name: None,
            time_limit_secs: 30.0,
            pattern: BulletPattern::Spiral { arms: 2, speed: 110.0, rotation_speed_deg: 90.0 },
            movement: BossMovement::Pendulum { amplitude: 60.0, frequency: 0.5, base_x: 0.0 },
            spell_card_bonus: 0,
        },
        // スペルカード 2: 「夜符「ナイトバード」」
        // 大量の小弾を自機狙いで複数方向に発射
        BossPhaseData {
            hp: 1200.0,
            hp_max: 1200.0,
            is_spell_card: true,
            spell_card_name: Some("夜符「ナイトバード」".to_string()),
            time_limit_secs: 50.0,
            pattern: BulletPattern::Aimed { count: 5, spread_deg: 40.0, speed: 130.0 },
            movement: BossMovement::Pendulum { amplitude: 100.0, frequency: 0.6, base_x: 0.0 },
            spell_card_bonus: 1_200_000,
        },
    ]
}

pub fn rumia_phases_hard() -> Vec<BossPhaseData> {
    let base = rumia_phases_normal();
    let params = DifficultyParams::for_difficulty(Difficulty::Hard);
    scale_phases(base, &params)
}

pub fn rumia_phases_lunatic() -> Vec<BossPhaseData> {
    let mut phases = rumia_phases_normal();
    // Lunatic 専用: フェーズ追加
    phases.push(BossPhaseData {
        hp: 1000.0,
        hp_max: 1000.0,
        is_spell_card: true,
        spell_card_name: Some("闇符「ロールオーバー」".to_string()),
        time_limit_secs: 45.0,
        pattern: BulletPattern::Ring { count: 24, speed: 150.0 },
        movement: BossMovement::Circle { radius: 40.0, speed_deg: 60.0, center: Vec2::new(0.0, 60.0) },
        spell_card_bonus: 1_500_000,
    });
    let params = DifficultyParams::for_difficulty(Difficulty::Lunatic);
    scale_phases(phases, &params)
}

fn scale_phases(phases: Vec<BossPhaseData>, params: &DifficultyParams) -> Vec<BossPhaseData> {
    phases.into_iter().map(|mut p| {
        p.hp *= params.enemy_hp_multiplier;
        p.hp_max *= params.enemy_hp_multiplier;
        p.time_limit_secs *= params.boss_time_multiplier;
        // pattern の speed/count は BulletPattern 内で別途スケール
        p
    }).collect()
}
```

### 2. Stage 1 完全スクリプト（2分間 + ルーミア）

```rust
// app/core/src/systems/stage/stage1.rs

pub fn stage1_script() -> Vec<SpawnEntry> {
    let mut script = Vec::new();

    // === 0〜30秒: 開幕ウェーブ ===
    // Wave 1: 上から妖精が3体降下
    for i in 0..3 {
        script.push(SpawnEntry {
            time: 2.0 + i as f32 * 0.8,
            kind: EnemyKind::Fairy,
            position: Vec2::new(-80.0 + 80.0 * i as f32, 260.0),
            movement: EnemyMovement::FallDown { speed: 60.0 },
            emitter: Some(aimed_emitter(3, 120.0, 2.5)),
        });
    }

    // Wave 2: 左右から妖精
    for (i, x) in [(-200.0f32, 1.0f32), (200.0, -1.0)].iter() {
        script.push(SpawnEntry {
            time: 10.0,
            kind: EnemyKind::Fairy,
            position: Vec2::new(*x, 80.0),
            movement: EnemyMovement::Linear { velocity: Vec2::new(x * -0.5, -30.0) * 60.0 },
            emitter: Some(ring_emitter(6, 100.0, 2.0)),
        });
    }

    // ... 残りのウェーブ (45秒まで)
    // Wave 3〜10: 省略（実装時に詳細追加）

    // === 90秒: ルーミア出現 ===
    // スポーンはstage_control_systemが担当（全ザコ撃破後）

    script.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    script
}

fn aimed_emitter(count: u8, speed: f32, interval: f32) -> BulletEmitter {
    BulletEmitter {
        pattern: BulletPattern::Aimed { count, spread_deg: 15.0, speed },
        bullet_kind: EnemyBulletKind::SmallRound,
        timer: Timer::from_seconds(interval, TimerMode::Repeating),
        active: true,
    }
}

fn ring_emitter(count: u8, speed: f32, interval: f32) -> BulletEmitter {
    BulletEmitter {
        pattern: BulletPattern::Ring { count, speed },
        bullet_kind: EnemyBulletKind::SmallRound,
        timer: Timer::from_seconds(interval, TimerMode::Repeating),
        active: true,
    }
}
```

### 3. ステージ開始時にスクリプト読み込み

```rust
// app/core/src/systems/stage/mod.rs

pub fn setup_stage(
    mut spawner: ResMut<EnemySpawner>,
    stage_data: Res<StageData>,
    selected: Res<SelectedDifficulty>,
) {
    spawner.script = match stage_data.current_stage {
        1 => stage1::stage1_script(),
        // 2〜6は後のPhaseで追加
        _ => vec![],
    };
    spawner.index = 0;
    spawner.elapsed = 0.0;
}
```

### 4. ステージクリア処理（暫定）

```rust
pub fn on_stage_clear(
    mut commands: Commands,
    game_data: Res<GameData>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    // 仮: 単純にステージクリア状態へ
    next_state.set(AppState::StageClear);
    // Phase 11 でスコア集計画面を追加
}
```

### 5. BGM統合（仮音源）

Phase 10 では仮音源（ループするダミーOGGファイル）を配置する。

```rust
pub fn on_enter_playing_bgm(
    mut play_bgm: EventWriter<PlayBgmEvent>,
    stage_data: Res<StageData>,
) {
    play_bgm.write(PlayBgmEvent {
        track: BgmTrack::Stage(stage_data.current_stage),
        fade_in_secs: 0.5,
    });
}

pub fn on_boss_appear_bgm(
    mut phase_events: EventReader<BossPhaseChangedEvent>,
    bosses: Query<&Boss>,
    mut play_bgm: EventWriter<PlayBgmEvent>,
    mut stop_bgm: EventWriter<StopBgmEvent>,
) {
    for event in phase_events.read() {
        if event.phase == 0 {
            // ボス初登場
            stop_bgm.write(StopBgmEvent { fade_out_secs: 0.5 });
            let Ok(boss) = bosses.get(event.entity) else { continue };
            play_bgm.write(PlayBgmEvent {
                track: BgmTrack::Boss(boss.boss_type),
                fade_in_secs: 0.5,
            });
        }
    }
}
```

### 6. パフォーマンス確認

ルーミアのスペルカード中に弾幕が最大密度になる場面をプロファイル:

```bash
# FPSログ付きで実行
RUST_LOG=bevy_diagnostic=debug just dev

# リリースビルドで確認
cargo run -p touhou-project-scarlet --release
```

期待値: 60fps 安定。弾幕 300+ でも50fps以上を目標。

---

## 参照

- `docs/01_specification.md` § Stage 1 / ルーミア
- `docs/03_danmaku_systems.md` § 5 (ボスシステム)
