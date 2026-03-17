# Phase 13: オーディオ統合

## 目標

BGMとSFXが機能する。状態遷移でBGMが自動切り替えされる。

## 完了条件

- [ ] BGMチャンネル・SFXチャンネルの初期化
- [ ] 状態遷移によるBGM自動切り替え
- [ ] 全SFXトリガーシステム
- [ ] 射撃SE間引き
- [ ] ポーズ時BGM一時停止/再開
- [ ] 仮音源ファイルの配置

---

## タスク詳細

### 1. ScarletAudioPlugin 完全実装

```rust
// app/audio/src/lib.rs

pub struct ScarletAudioPlugin;

impl Plugin for ScarletAudioPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(AudioPlugin)
            .add_audio_channel::<BgmChannel>()
            .add_audio_channel::<SfxChannel>()
            .init_resource::<AudioHandles>()
            .init_resource::<AudioSettings>()
            .add_event::<PlayBgmEvent>()
            .add_event::<StopBgmEvent>()
            .add_event::<PlaySfxEvent>()
            .add_systems(Startup, set_initial_volumes)
            .add_systems(Update, (
                bgm::handle_bgm_transitions,
                sfx::handle_sfx_events,
                apply_audio_settings,
            ))
            // BGM 自動切り替え
            .add_systems(OnEnter(AppState::Title),      bgm::on_enter_title)
            .add_systems(OnEnter(AppState::Playing),    bgm::on_enter_playing)
            .add_systems(OnEnter(AppState::Paused),     bgm::on_enter_pause)
            .add_systems(OnExit(AppState::Paused),      bgm::on_exit_pause)
            .add_systems(OnEnter(AppState::GameOver),   bgm::on_enter_game_over)
            .add_systems(OnEnter(AppState::Ending),     bgm::on_enter_ending)
            // SFX トリガー
            .add_systems(Update, (
                sfx::triggers::sfx_on_shoot,
                sfx::triggers::sfx_on_player_hit,
                sfx::triggers::sfx_on_graze,
                sfx::triggers::sfx_on_enemy_die,
                sfx::triggers::sfx_on_item_collect,
                sfx::triggers::sfx_on_extend,
                sfx::triggers::sfx_on_spell_card,
                sfx::triggers::sfx_on_spell_card_break,
                sfx::triggers::sfx_on_bomb,
                sfx::triggers::sfx_on_boss_appear,
            ).run_if(in_state(AppState::Playing)));
    }
}

fn set_initial_volumes(
    bgm: Res<AudioChannel<BgmChannel>>,
    sfx: Res<AudioChannel<SfxChannel>>,
    settings: Res<AudioSettings>,
) {
    bgm.set_volume((settings.master_volume * settings.bgm_volume) as f64);
    sfx.set_volume((settings.master_volume * settings.sfx_volume) as f64);
}
```

### 2. BGMシステム

```rust
// app/audio/src/bgm.rs

pub fn handle_bgm_transitions(
    bgm: Res<AudioChannel<BgmChannel>>,
    mut play_events: EventReader<PlayBgmEvent>,
    mut stop_events: EventReader<StopBgmEvent>,
    handles: Res<AudioHandles>,
) {
    for event in stop_events.read() {
        bgm.stop().fade_out(AudioTween::linear(
            Duration::from_secs_f32(event.fade_out_secs)
        ));
    }

    for event in play_events.read() {
        let Some(source) = get_bgm_handle(&handles, &event.track) else { continue };
        bgm.play(source).looped().fade_in(AudioTween::linear(
            Duration::from_secs_f32(event.fade_in_secs)
        ));
    }
}

fn get_bgm_handle(handles: &AudioHandles, track: &BgmTrack) -> Option<Handle<AudioSource>> {
    Some(match track {
        BgmTrack::Title        => handles.bgm_title.clone(),
        BgmTrack::Stage(1)     => handles.bgm_stage1.clone(),
        BgmTrack::Stage(2)     => handles.bgm_stage2.clone(),
        BgmTrack::Stage(3)     => handles.bgm_stage3.clone(),
        BgmTrack::Stage(4)     => handles.bgm_stage4.clone(),
        BgmTrack::Stage(5)     => handles.bgm_stage5.clone(),
        BgmTrack::Stage(6)     => handles.bgm_stage6.clone(),
        BgmTrack::Stage(_)     => return None,
        BgmTrack::Boss(BossType::Rumia)     => handles.bgm_boss_rumia.clone(),
        BgmTrack::Boss(BossType::Cirno)     => handles.bgm_boss_cirno.clone(),
        BgmTrack::Boss(BossType::Meiling)   => handles.bgm_boss_meiling.clone(),
        BgmTrack::Boss(BossType::Patchouli) => handles.bgm_boss_patchouli.clone(),
        BgmTrack::Boss(BossType::Sakuya)    => handles.bgm_boss_sakuya.clone(),
        BgmTrack::Boss(BossType::Remilia)   => handles.bgm_boss_remilia.clone(),
        BgmTrack::Boss(BossType::Flandre)   => handles.bgm_boss_flandre.clone(),
        BgmTrack::GameOver     => handles.bgm_game_over.clone(),
        BgmTrack::Ending       => handles.bgm_ending.clone(),
        BgmTrack::StaffRoll    => handles.bgm_staff_roll.clone(),
    })
}

pub fn on_enter_title(mut play: EventWriter<PlayBgmEvent>, mut stop: EventWriter<StopBgmEvent>) {
    stop.write(StopBgmEvent { fade_out_secs: 1.0 });
    play.write(PlayBgmEvent { track: BgmTrack::Title, fade_in_secs: 1.0 });
}

pub fn on_enter_playing(
    mut play: EventWriter<PlayBgmEvent>,
    mut stop: EventWriter<StopBgmEvent>,
    stage_data: Res<StageData>,
) {
    stop.write(StopBgmEvent { fade_out_secs: 0.5 });
    play.write(PlayBgmEvent { track: BgmTrack::Stage(stage_data.current_stage), fade_in_secs: 0.5 });
}

pub fn on_enter_pause(bgm: Res<AudioChannel<BgmChannel>>) { bgm.pause(); }
pub fn on_exit_pause(bgm: Res<AudioChannel<BgmChannel>>) { bgm.resume(); }

pub fn on_enter_game_over(
    mut play: EventWriter<PlayBgmEvent>,
    mut stop: EventWriter<StopBgmEvent>,
) {
    stop.write(StopBgmEvent { fade_out_secs: 0.5 });
    play.write(PlayBgmEvent { track: BgmTrack::GameOver, fade_in_secs: 0.0 });
}

pub fn on_enter_ending(
    mut play: EventWriter<PlayBgmEvent>,
    mut stop: EventWriter<StopBgmEvent>,
) {
    stop.write(StopBgmEvent { fade_out_secs: 1.0 });
    play.write(PlayBgmEvent { track: BgmTrack::Ending, fade_in_secs: 1.0 });
}

/// ボス出現時: ステージBGM → ボスBGM
pub fn on_boss_appear(
    mut phase_events: EventReader<BossPhaseChangedEvent>,
    bosses: Query<&Boss>,
    mut play: EventWriter<PlayBgmEvent>,
    mut stop: EventWriter<StopBgmEvent>,
) {
    for event in phase_events.read() {
        if event.phase != 0 { continue; } // 最初のフェーズ変化のみ
        let Ok(boss) = bosses.get(event.entity) else { continue };
        stop.write(StopBgmEvent { fade_out_secs: 0.5 });
        play.write(PlayBgmEvent {
            track: BgmTrack::Boss(boss.boss_type),
            fade_in_secs: 0.5,
        });
    }
}
```

### 3. SFXシステム

```rust
// app/audio/src/sfx/mod.rs

pub fn handle_sfx_events(
    sfx: Res<AudioChannel<SfxChannel>>,
    mut events: EventReader<PlaySfxEvent>,
    handles: Res<AudioHandles>,
) {
    for event in events.read() {
        let Some(source) = get_sfx_handle(&handles, &event.sfx) else { continue };
        sfx.play(source).with_volume(event.volume as f64);
    }
}

fn get_sfx_handle(handles: &AudioHandles, kind: &SfxKind) -> Option<Handle<AudioSource>> {
    Some(match kind {
        SfxKind::ShootReimu       => handles.sfx_shoot_reimu.clone(),
        SfxKind::ShootMarisa      => handles.sfx_shoot_marisa.clone(),
        SfxKind::EnemyHit         => handles.sfx_enemy_hit.clone(),
        SfxKind::EnemyDie         => handles.sfx_enemy_die.clone(),
        SfxKind::PlayerHit        => handles.sfx_player_hit.clone(),
        SfxKind::Graze            => handles.sfx_graze.clone(),
        SfxKind::Bomb             => handles.sfx_bomb.clone(),
        SfxKind::ItemCollect      => handles.sfx_item_collect.clone(),
        SfxKind::PowerUp          => handles.sfx_power_up.clone(),
        SfxKind::Extend           => handles.sfx_extend.clone(),
        SfxKind::BossAppear       => handles.sfx_boss_appear.clone(),
        SfxKind::SpellCard        => handles.sfx_spell_card.clone(),
        SfxKind::SpellCardBreak   => handles.sfx_spell_card_break.clone(),
        SfxKind::MenuSelect       => handles.sfx_menu_select.clone(),
        SfxKind::MenuConfirm      => handles.sfx_menu_confirm.clone(),
        SfxKind::MenuCancel       => handles.sfx_menu_cancel.clone(),
    })
}
```

### 4. SEトリガー群

```rust
// app/audio/src/sfx/triggers.rs

pub fn sfx_on_shoot(
    mut cooldown: Local<f32>,
    shoot_events: EventReader<ShootEvent>,
    mut sfx: EventWriter<PlaySfxEvent>,
    selected: Res<SelectedCharacter>,
    time: Res<Time>,
) {
    *cooldown -= time.delta_secs();
    if *cooldown > 0.0 { shoot_events.clear(); return; }

    if !shoot_events.is_empty() {
        shoot_events.clear();
        *cooldown = 0.05; // 50ms間引き
        sfx.write(PlaySfxEvent {
            sfx: match selected.character {
                CharacterType::Reimu => SfxKind::ShootReimu,
                CharacterType::Marisa => SfxKind::ShootMarisa,
            },
            volume: 0.6,
        });
    }
}

pub fn sfx_on_player_hit(
    mut events: EventReader<PlayerHitEvent>,
    mut sfx: EventWriter<PlaySfxEvent>,
) {
    for _ in events.read() {
        sfx.write(PlaySfxEvent { sfx: SfxKind::PlayerHit, volume: 1.0 });
    }
}

pub fn sfx_on_graze(
    mut events: EventReader<GrazeEvent>,
    mut sfx: EventWriter<PlaySfxEvent>,
    mut cooldown: Local<f32>,
    time: Res<Time>,
) {
    *cooldown -= time.delta_secs();
    let count = events.read().count();
    if count > 0 && *cooldown <= 0.0 {
        *cooldown = 0.08; // グレイズSEも間引き
        sfx.write(PlaySfxEvent { sfx: SfxKind::Graze, volume: 0.3 });
    }
}

pub fn sfx_on_enemy_die(mut events: EventReader<EnemyDefeatedEvent>, mut sfx: EventWriter<PlaySfxEvent>) {
    if events.read().count() > 0 {
        sfx.write(PlaySfxEvent { sfx: SfxKind::EnemyDie, volume: 0.8 });
    }
}

pub fn sfx_on_item_collect(mut events: EventReader<ItemCollectedEvent>, mut sfx: EventWriter<PlaySfxEvent>) {
    for _ in events.read() {
        sfx.write(PlaySfxEvent { sfx: SfxKind::ItemCollect, volume: 0.7 });
    }
}

pub fn sfx_on_extend(mut events: EventReader<ExtendEvent>, mut sfx: EventWriter<PlaySfxEvent>) {
    for _ in events.read() {
        sfx.write(PlaySfxEvent { sfx: SfxKind::Extend, volume: 1.0 });
    }
}

pub fn sfx_on_bomb(mut events: EventReader<BombUsedEvent>, mut sfx: EventWriter<PlaySfxEvent>) {
    for _ in events.read() {
        sfx.write(PlaySfxEvent { sfx: SfxKind::Bomb, volume: 1.0 });
    }
}

pub fn sfx_on_spell_card(
    mut phase_events: EventReader<BossPhaseChangedEvent>,
    bosses: Query<&Boss>,
    mut sfx: EventWriter<PlaySfxEvent>,
) {
    for event in phase_events.read() {
        let Ok(boss) = bosses.get(event.entity) else { continue };
        let phase = &boss.phases[event.phase];
        if phase.is_spell_card {
            sfx.write(PlaySfxEvent { sfx: SfxKind::SpellCard, volume: 1.0 });
        }
    }
}

pub fn sfx_on_spell_card_break(
    // スペルカードをHP0で撃破したときの音
    // BossPhaseChangedEvent でボーナスが発生していれば撃破と判断
    mut sfx: EventWriter<PlaySfxEvent>,
    // ...
) {}

pub fn sfx_on_boss_appear(
    mut phase_events: EventReader<BossPhaseChangedEvent>,
    mut sfx: EventWriter<PlaySfxEvent>,
) {
    for event in phase_events.read() {
        if event.phase == 0 {
            sfx.write(PlaySfxEvent { sfx: SfxKind::BossAppear, volume: 1.0 });
        }
    }
}
```

### 5. 仮音源ファイル

開発フェーズでは無音のOGGファイルや、フリー素材（ロイヤリティフリー）を配置:

```bash
# 無音OGGファイルを生成 (ffmpegがある場合)
ffmpeg -f lavfi -i anullsrc -t 5 -q:a 9 -acodec libvorbis assets/audio/bgm/title.ogg
```

または、1kHz単音のOGGファイルをダミーとして配置。

---

## 参照

- `docs/05_audio.md` — オーディオ設計全体
- `docs/references/vampire-survivors/app/audio/`
