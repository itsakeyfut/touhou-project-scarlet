# 05 オーディオ設計

## 概要

bevy_kira_audio 0.24.0 を使用したBGM・SEシステムの設計ドキュメント。

---

## 1. オーディオアーキテクチャ

### 1.1 チャンネル構成

```rust
// app/audio/src/channels.rs

#[derive(Resource)]
pub struct BgmChannel;

#[derive(Resource)]
pub struct SfxChannel;

#[derive(Resource)]
pub struct VoiceChannel; // 将来実装用
```

```rust
// app/audio/src/lib.rs

pub struct ScarletAudioPlugin;

impl Plugin for ScarletAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AudioPlugin)
            .add_audio_channel::<BgmChannel>()
            .add_audio_channel::<SfxChannel>()
            .add_systems(Startup, set_channel_volumes)
            .add_systems(Update, (
                handle_bgm_transitions,
                handle_sfx_events,
            ).chain());
    }
}

fn set_channel_volumes(
    bgm: Res<AudioChannel<BgmChannel>>,
    sfx: Res<AudioChannel<SfxChannel>>,
) {
    bgm.set_volume(0.7);
    sfx.set_volume(0.9);
}
```

### 1.2 アセットハンドル

```rust
// app/audio/src/handles.rs

#[derive(Resource, Default)]
pub struct AudioHandles {
    // BGM
    pub bgm_title: Handle<AudioSource>,
    pub bgm_stage1: Handle<AudioSource>,
    pub bgm_stage2: Handle<AudioSource>,
    pub bgm_stage3: Handle<AudioSource>,
    pub bgm_stage4: Handle<AudioSource>,
    pub bgm_stage5: Handle<AudioSource>,
    pub bgm_stage6: Handle<AudioSource>,
    pub bgm_extra: Handle<AudioSource>,
    pub bgm_boss_rumia: Handle<AudioSource>,
    pub bgm_boss_cirno: Handle<AudioSource>,
    pub bgm_boss_meiling: Handle<AudioSource>,
    pub bgm_boss_patchouli: Handle<AudioSource>,
    pub bgm_boss_sakuya: Handle<AudioSource>,
    pub bgm_boss_remilia: Handle<AudioSource>,
    pub bgm_boss_flandre: Handle<AudioSource>,
    pub bgm_game_over: Handle<AudioSource>,
    pub bgm_ending: Handle<AudioSource>,
    pub bgm_staff_roll: Handle<AudioSource>,

    // SFX
    pub sfx_shoot_reimu: Handle<AudioSource>,
    pub sfx_shoot_marisa: Handle<AudioSource>,
    pub sfx_enemy_hit: Handle<AudioSource>,
    pub sfx_enemy_die: Handle<AudioSource>,
    pub sfx_player_hit: Handle<AudioSource>,
    pub sfx_graze: Handle<AudioSource>,
    pub sfx_bomb: Handle<AudioSource>,
    pub sfx_item_collect: Handle<AudioSource>,
    pub sfx_power_up: Handle<AudioSource>,
    pub sfx_extend: Handle<AudioSource>,
    pub sfx_boss_appear: Handle<AudioSource>,
    pub sfx_spell_card: Handle<AudioSource>,
    pub sfx_spell_card_break: Handle<AudioSource>,
    pub sfx_menu_select: Handle<AudioSource>,
    pub sfx_menu_confirm: Handle<AudioSource>,
    pub sfx_menu_cancel: Handle<AudioSource>,
}
```

---

## 2. BGMリスト

### 2.1 ステージBGM

| ステージ | 曲名 | ファイル |
|---|---|---|
| タイトル | 「東方紅魔郷」メインテーマ | `bgm/title.ogg` |
| ステージ1 | 「赤より紅い夢」 | `bgm/stage1.ogg` |
| ステージ2 | 「妖魔夜行」 | `bgm/stage2.ogg` |
| ステージ3 | 「上海紅茶館 〜 Chinese Tea」 | `bgm/stage3.ogg` |
| ステージ4A | 「ラクトガール 〜 若い未来」 | `bgm/stage4.ogg` |
| ステージ4B | 「眠れる勇者と目醒めの呪い」 | `bgm/stage4b.ogg` |
| ステージ5 | 「月時計 〜 ルナ・ダイアル」 | `bgm/stage5.ogg` |
| ステージ6 | 「ツェペシュの幼き末裔」 | `bgm/stage6.ogg` |
| Extraステージ | 「メイドと血の懐中時計」 | `bgm/extra.ogg` |

### 2.2 ボスBGM

| ボス | 曲名 | ファイル |
|---|---|---|
| ルーミア (Stage 1) | 「ルーミアのテーマ」 | `bgm/boss_rumia.ogg` |
| チルノ (Stage 2) | 「おてんば恋娘」 | `bgm/boss_cirno.ogg` |
| 紅美鈴 (Stage 3) | 「上海紅茶館」 | `bgm/boss_meiling.ogg` |
| パチュリー (Stage 4) | 「ラクトガール」 | `bgm/boss_patchouli.ogg` |
| 十六夜咲夜 (Stage 5) | 「月時計 〜 ルナ・ダイアル」 | `bgm/boss_sakuya.ogg` |
| レミリア・スカーレット (Stage 6) | 「亡き王女の為のセプテット」 | `bgm/boss_remilia.ogg` |
| フランドール・スカーレット (Extra) | 「U.N.オーエンは彼女なのか？」 | `bgm/boss_flandre.ogg` |

### 2.3 その他BGM

| 場面 | 曲名 | ファイル |
|---|---|---|
| ゲームオーバー | （短いジングル） | `bgm/game_over.ogg` |
| エンディング | 「夢のホワイトトラベラー」 | `bgm/ending.ogg` |
| スタッフロール | 「夢想天生」 | `bgm/staff_roll.ogg` |

---

## 3. SFXリスト

### 3.1 プレイヤー

| イベント | ファイル | 説明 |
|---|---|---|
| 霊夢ショット | `sfx/shoot_reimu.ogg` | 短い高音 |
| 魔理沙ショット | `sfx/shoot_marisa.ogg` | やや重い音 |
| プレイヤー被弾 | `sfx/player_hit.ogg` | 短い爆発音 |
| グレイズ | `sfx/graze.ogg` | スパーク音 (低音量) |
| ボム | `sfx/bomb.ogg` | 大きな爆発 |
| ボム（霊夢） | `sfx/bomb_reimu.ogg` | 結界展開音 |
| ボム（魔理沙） | `sfx/bomb_marisa.ogg` | レーザー発射音 |

### 3.2 敵・ステージ

| イベント | ファイル | 説明 |
|---|---|---|
| 敵ヒット | `sfx/enemy_hit.ogg` | 短い打撃音 |
| 敵撃破 | `sfx/enemy_die.ogg` | 爆発音 |
| ボス登場 | `sfx/boss_appear.ogg` | 重厚な登場音 |
| スペルカード | `sfx/spell_card.ogg` | スペルカード発動音 |
| スペルカード撃破 | `sfx/spell_card_break.ogg` | クリア音 |

### 3.3 アイテム・UI

| イベント | ファイル | 説明 |
|---|---|---|
| アイテム取得 | `sfx/item_collect.ogg` | コレクト音 |
| パワーアップ | `sfx/power_up.ogg` | パワー段階上昇音 |
| エクステンド | `sfx/extend.ogg` | 残機増加ジングル |
| メニュー選択 | `sfx/menu_select.ogg` | カーソル移動 |
| メニュー決定 | `sfx/menu_confirm.ogg` | 決定音 |
| メニューキャンセル | `sfx/menu_cancel.ogg` | キャンセル音 |

---

## 4. BGM遷移システム

### 4.1 BGMトリガーイベント

```rust
#[derive(Event)]
pub struct PlayBgmEvent {
    pub track: BgmTrack,
    pub fade_in_secs: f32,
}

#[derive(Event)]
pub struct StopBgmEvent {
    pub fade_out_secs: f32,
}

pub enum BgmTrack {
    Title,
    Stage(u8),
    Boss(BossType),
    GameOver,
    Ending,
    StaffRoll,
}
```

### 4.2 BGM再生システム

```rust
pub fn handle_bgm_transitions(
    bgm_channel: Res<AudioChannel<BgmChannel>>,
    mut play_events: EventReader<PlayBgmEvent>,
    mut stop_events: EventReader<StopBgmEvent>,
    handles: Res<AudioHandles>,
) {
    for event in stop_events.read() {
        bgm_channel
            .stop()
            .fade_out(AudioTween::linear(Duration::from_secs_f32(event.fade_out_secs)));
    }

    for event in play_events.read() {
        let source = match event.track {
            BgmTrack::Title => handles.bgm_title.clone(),
            BgmTrack::Stage(1) => handles.bgm_stage1.clone(),
            BgmTrack::Stage(2) => handles.bgm_stage2.clone(),
            BgmTrack::Stage(3) => handles.bgm_stage3.clone(),
            BgmTrack::Stage(4) => handles.bgm_stage4.clone(),
            BgmTrack::Stage(5) => handles.bgm_stage5.clone(),
            BgmTrack::Stage(6) => handles.bgm_stage6.clone(),
            BgmTrack::Boss(BossType::Remilia) => handles.bgm_boss_remilia.clone(),
            BgmTrack::Boss(BossType::Flandre) => handles.bgm_boss_flandre.clone(),
            // ... その他のボス
            BgmTrack::GameOver => handles.bgm_game_over.clone(),
            BgmTrack::Ending => handles.bgm_ending.clone(),
            _ => return,
        };

        bgm_channel
            .play(source)
            .looped()
            .fade_in(AudioTween::linear(Duration::from_secs_f32(event.fade_in_secs)));
    }
}
```

### 4.3 状態遷移によるBGM自動切り替え

```rust
pub fn on_enter_title(
    mut play_bgm: EventWriter<PlayBgmEvent>,
    mut stop_bgm: EventWriter<StopBgmEvent>,
) {
    stop_bgm.write(StopBgmEvent { fade_out_secs: 1.0 });
    play_bgm.write(PlayBgmEvent {
        track: BgmTrack::Title,
        fade_in_secs: 1.0,
    });
}

pub fn on_enter_playing(
    stage_data: Res<StageData>,
    mut play_bgm: EventWriter<PlayBgmEvent>,
) {
    play_bgm.write(PlayBgmEvent {
        track: BgmTrack::Stage(stage_data.current_stage),
        fade_in_secs: 0.5,
    });
}

pub fn on_boss_appear(
    boss_events: EventReader<BossPhaseChangedEvent>,
    bosses: Query<&Boss>,
    mut play_bgm: EventWriter<PlayBgmEvent>,
    mut stop_bgm: EventWriter<StopBgmEvent>,
) {
    // ボス出現でステージBGMをフェードアウト、ボスBGM開始
}
```

---

## 5. SEシステム

### 5.1 SFXイベント駆動

```rust
#[derive(Event)]
pub struct PlaySfxEvent {
    pub sfx: SfxKind,
    pub volume: f32, // 0.0 - 1.0
}

pub enum SfxKind {
    ShootReimu,
    ShootMarisa,
    EnemyHit,
    EnemyDie,
    PlayerHit,
    Graze,
    Bomb,
    ItemCollect,
    PowerUp,
    Extend,
    BossAppear,
    SpellCard,
    SpellCardBreak,
    MenuSelect,
    MenuConfirm,
    MenuCancel,
}
```

```rust
pub fn handle_sfx_events(
    sfx_channel: Res<AudioChannel<SfxChannel>>,
    mut sfx_events: EventReader<PlaySfxEvent>,
    handles: Res<AudioHandles>,
) {
    for event in sfx_events.read() {
        let source = match event.sfx {
            SfxKind::ShootReimu => handles.sfx_shoot_reimu.clone(),
            SfxKind::Graze => handles.sfx_graze.clone(),
            SfxKind::Extend => handles.sfx_extend.clone(),
            // ...
        };

        sfx_channel.play(source).with_volume(event.volume as f64);
    }
}
```

### 5.2 SEトリガーシステム

各ゲームイベントをSFXに変換するシステム群：

```rust
pub fn sfx_on_graze(
    mut graze_events: EventReader<GrazeEvent>,
    mut sfx: EventWriter<PlaySfxEvent>,
) {
    for _ in graze_events.read() {
        sfx.write(PlaySfxEvent {
            sfx: SfxKind::Graze,
            volume: 0.4, // グレイズ音は小さめ
        });
    }
}

pub fn sfx_on_player_hit(
    mut hit_events: EventReader<PlayerHitEvent>,
    mut sfx: EventWriter<PlaySfxEvent>,
) {
    for _ in hit_events.read() {
        sfx.write(PlaySfxEvent {
            sfx: SfxKind::PlayerHit,
            volume: 1.0,
        });
    }
}

pub fn sfx_on_extend(
    mut extend_events: EventReader<ExtendEvent>,
    mut sfx: EventWriter<PlaySfxEvent>,
) {
    for _ in extend_events.read() {
        sfx.write(PlaySfxEvent {
            sfx: SfxKind::Extend,
            volume: 1.0,
        });
    }
}
```

### 5.3 射撃SE（連射制限）

プレイヤーのショットSEは連射時に重ならないよう間引き制御する。

```rust
pub struct ShootSfxCooldown {
    pub timer: Timer,
}

impl Default for ShootSfxCooldown {
    fn default() -> Self {
        Self { timer: Timer::from_seconds(0.05, TimerMode::Repeating) }
    }
}

pub fn sfx_on_shoot(
    mut cooldown: Local<ShootSfxCooldown>,
    mut shoot_events: EventReader<ShootEvent>,
    mut sfx: EventWriter<PlaySfxEvent>,
    selected: Res<SelectedCharacter>,
    time: Res<Time>,
) {
    cooldown.timer.tick(time.delta());
    let fired = shoot_events.read().count() > 0;

    if fired && cooldown.timer.just_finished() {
        let sfx_kind = match selected.character {
            CharacterType::Reimu => SfxKind::ShootReimu,
            CharacterType::Marisa => SfxKind::ShootMarisa,
        };
        sfx.write(PlaySfxEvent { sfx: sfx_kind, volume: 0.7 });
    }
}
```

---

## 6. 音量・設定

### 6.1 設定値

```rust
#[derive(Resource, serde::Serialize, serde::Deserialize)]
pub struct AudioSettings {
    pub master_volume: f32,  // 0.0 - 1.0
    pub bgm_volume: f32,     // 0.0 - 1.0
    pub sfx_volume: f32,     // 0.0 - 1.0
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            bgm_volume: 0.7,
            sfx_volume: 0.9,
        }
    }
}
```

### 6.2 音量適用

```rust
pub fn apply_audio_settings(
    settings: Res<AudioSettings>,
    bgm_channel: Res<AudioChannel<BgmChannel>>,
    sfx_channel: Res<AudioChannel<SfxChannel>>,
) {
    if settings.is_changed() {
        bgm_channel.set_volume(
            (settings.master_volume * settings.bgm_volume) as f64
        );
        sfx_channel.set_volume(
            (settings.master_volume * settings.sfx_volume) as f64
        );
    }
}
```

---

## 7. アセットファイル構成

```
app/touhou-project-scarlet/assets/
└── audio/
    ├── bgm/
    │   ├── title.ogg
    │   ├── stage1.ogg
    │   ├── stage2.ogg
    │   ├── stage3.ogg
    │   ├── stage4.ogg
    │   ├── stage5.ogg
    │   ├── stage6.ogg
    │   ├── extra.ogg
    │   ├── boss_rumia.ogg
    │   ├── boss_cirno.ogg
    │   ├── boss_meiling.ogg
    │   ├── boss_patchouli.ogg
    │   ├── boss_sakuya.ogg
    │   ├── boss_remilia.ogg
    │   ├── boss_flandre.ogg
    │   ├── game_over.ogg
    │   ├── ending.ogg
    │   └── staff_roll.ogg
    └── sfx/
        ├── shoot_reimu.ogg
        ├── shoot_marisa.ogg
        ├── enemy_hit.ogg
        ├── enemy_die.ogg
        ├── player_hit.ogg
        ├── graze.ogg
        ├── bomb.ogg
        ├── item_collect.ogg
        ├── power_up.ogg
        ├── extend.ogg
        ├── boss_appear.ogg
        ├── spell_card.ogg
        ├── spell_card_break.ogg
        ├── menu_select.ogg
        ├── menu_confirm.ogg
        └── menu_cancel.ogg
```

> **注意**: 東方projectの楽曲は著作権が上海アリス幻樂団に帰属する。開発・テスト用にはダミーまたはロイヤリティフリーの楽曲を使用すること。公開・配布時は版権楽曲を使用しないこと。

---

## 8. ポーズ時オーディオ制御

```rust
pub fn on_pause(
    bgm_channel: Res<AudioChannel<BgmChannel>>,
) {
    bgm_channel.pause();
}

pub fn on_resume(
    bgm_channel: Res<AudioChannel<BgmChannel>>,
) {
    bgm_channel.resume();
}
```

これらはそれぞれ `OnEnter(AppState::Paused)` および `OnExit(AppState::Paused)` にスケジュールする。
