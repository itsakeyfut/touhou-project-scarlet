use bevy::{prelude::*, sprite_render::Material2dPlugin};

use crate::game_set::GameSystemSet;

use crate::{
    components::player::{GrazeVisual, Player},
    events::{BossHitEvent, GrazeEvent},
    shaders::{
        BulletGlowMaterial, BulletTrailMaterial, GrazeMaterial, HitFlashMaterial,
        SpellCardBackground, SpellCardBgMaterial,
    },
    states::AppState,
};

/// Registers all custom shader materials and their per-frame uniform updaters.
///
/// Add this plugin to the `App` in `main.rs` **after** `DefaultPlugins`.
///
/// # Adding new materials
///
/// ```rust,ignore
/// app.add_plugins(Material2dPlugin::<MyMaterial>::default());
/// // add a time-update system if the material has a `time` field
/// ```
pub struct ScarletShadersPlugin;

impl Plugin for ScarletShadersPlugin {
    fn build(&self, app: &mut App) {
        // Phase 04 materials.
        app.add_plugins(Material2dPlugin::<BulletGlowMaterial>::default())
            .add_plugins(Material2dPlugin::<BulletTrailMaterial>::default());

        // Phase 05 materials.
        app.add_plugins(Material2dPlugin::<GrazeMaterial>::default());

        // Phase 08 materials.
        app.add_plugins(Material2dPlugin::<HitFlashMaterial>::default())
            .add_plugins(Material2dPlugin::<SpellCardBgMaterial>::default());

        // Uniform time updates — only while the game is running.
        app.add_systems(
            Update,
            (update_bullet_glow_time, update_bullet_trail_time).run_if(in_state(AppState::Playing)),
        );

        // Hit-flash systems — trigger on BossHitEvent, then decay each frame.
        app.add_systems(
            Update,
            (trigger_boss_hit_flash, update_hit_flash)
                .chain()
                .in_set(GameSystemSet::Effects),
        );

        // Spell-card background time uniform update.
        app.add_systems(
            Update,
            update_spell_card_bg_time.run_if(in_state(AppState::Playing)),
        );

        // Graze field visual — spawn once when the player entity appears,
        // then update uniforms every frame.
        // update_graze_material must run after Collision so that GrazeEvents
        // emitted by graze_detection_system are visible to MessageReader.
        app.add_systems(
            Update,
            (
                setup_graze_visual,
                update_graze_material.after(GameSystemSet::Collision),
            )
                .run_if(in_state(AppState::Playing)),
        );

        // TODO(phase-09): add Material2dPlugin::<BombReimuMaterial>, BombMarisaMaterial
        // TODO(phase-12): add Material2dPlugin::<PixelOutlineMaterial>
    }
}

// ---------------------------------------------------------------------------
// Time uniform updaters
// ---------------------------------------------------------------------------

/// Advances the `time` field on every [`BulletGlowMaterial`] instance.
///
/// This drives the pulse animation in `bullet_glow.wgsl`.
/// Performance note: iterates over all material instances — O(n) per frame.
/// When bullet counts grow, consider sharing one material per `EnemyBulletKind`.
pub fn update_bullet_glow_time(time: Res<Time>, mut materials: ResMut<Assets<BulletGlowMaterial>>) {
    let t = time.elapsed_secs();
    for (_, mat) in materials.iter_mut() {
        mat.time = t;
    }
}

/// Advances the `time` field on every [`BulletTrailMaterial`] instance.
pub fn update_bullet_trail_time(
    time: Res<Time>,
    mut materials: ResMut<Assets<BulletTrailMaterial>>,
) {
    let t = time.elapsed_secs();
    for (_, mat) in materials.iter_mut() {
        mat.time = t;
    }
}

// ---------------------------------------------------------------------------
// Graze field systems
// ---------------------------------------------------------------------------

/// Spawns a [`GrazeVisual`] child entity on the player the first frame it exists.
///
/// Uses [`Added<Player>`] so it fires exactly once after
/// [`crate::systems::player::spawn_player`] runs. The graze circle
/// (`Mesh2d(Circle::new(16.0))`) is added as a child so its position
/// automatically tracks the player with no extra system needed.
///
/// Registered in [`ScarletShadersPlugin`].
pub fn setup_graze_visual(
    mut commands: Commands,
    player: Query<Entity, Added<Player>>,
    mut graze_materials: ResMut<Assets<GrazeMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let Ok(player_entity) = player.single() else {
        return;
    };

    // Match the 16 px graze detection radius defined in PlayerStats::graze_radius.
    let mesh = meshes.add(Circle::new(16.0));
    let material = graze_materials.add(GrazeMaterial::default());

    let graze_visual = commands
        .spawn((
            GrazeVisual,
            Mesh2d(mesh),
            MeshMaterial2d(material),
            // Render behind the player sprite (z=1.0) but above the background.
            Transform::from_xyz(0.0, 0.0, 0.5),
        ))
        .id();

    commands.entity(player_entity).add_child(graze_visual);
}

/// Updates [`GrazeMaterial`] uniforms every frame.
///
/// - `time` — drives the noise-based ring animation.
/// - `slow_mode` — set to `1` while Shift is held, making the ring clearly visible.
/// - `graze_intensity` — spikes to `1.0` when a [`GrazeEvent`] is received,
///   then decays at `5.0/s` (≈ 0.2 s flash) each subsequent frame.
///
/// Registered in [`ScarletShadersPlugin`].
pub fn update_graze_material(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut materials: ResMut<Assets<GrazeMaterial>>,
    graze_visuals: Query<&MeshMaterial2d<GrazeMaterial>, With<GrazeVisual>>,
    mut graze_events: MessageReader<GrazeEvent>,
) {
    let t = time.elapsed_secs();
    let slow = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    let grazed_this_frame = graze_events.read().count() > 0;

    for handle in &graze_visuals {
        let Some(mat) = materials.get_mut(handle) else {
            continue;
        };
        mat.time = t;
        mat.slow_mode = u32::from(slow);
        if grazed_this_frame {
            mat.graze_intensity = 1.0;
        } else {
            mat.graze_intensity = (mat.graze_intensity - time.delta_secs() * 5.0).max(0.0);
        }
    }
}

// ---------------------------------------------------------------------------
// Hit-flash systems
// ---------------------------------------------------------------------------

/// Sets `flash_intensity = 1.0` on every [`HitFlashMaterial`] belonging to a
/// boss entity that received a [`BossHitEvent`] this frame.
///
/// Runs in [`GameSystemSet::Effects`] (after Collision so events are visible).
/// [`update_hit_flash`] is chained after this system to begin the decay
/// immediately on the same frame.
pub fn trigger_boss_hit_flash(
    mut hit_events: MessageReader<BossHitEvent>,
    bosses: Query<&MeshMaterial2d<HitFlashMaterial>>,
    mut flash_materials: ResMut<Assets<HitFlashMaterial>>,
) {
    for event in hit_events.read() {
        let Ok(handle) = bosses.get(event.entity) else {
            continue;
        };
        if let Some(mat) = flash_materials.get_mut(handle) {
            mat.flash_intensity = 1.0;
        }
    }
}

/// Fades `flash_intensity` toward `0.0` at `8.0 units/s` on every
/// [`HitFlashMaterial`] instance, producing a ≈ 0.125 s white-flash effect.
///
/// Chained after [`trigger_boss_hit_flash`] in [`GameSystemSet::Effects`].
pub fn update_hit_flash(time: Res<Time>, mut flash_materials: ResMut<Assets<HitFlashMaterial>>) {
    let decay = time.delta_secs() * 8.0;
    for (_, mat) in flash_materials.iter_mut() {
        mat.flash_intensity = (mat.flash_intensity - decay).max(0.0);
    }
}

// ---------------------------------------------------------------------------
// Spell-card background time updater
// ---------------------------------------------------------------------------

/// Advances `time` and fades `intensity` toward `1.0` on every
/// [`SpellCardBgMaterial`] instance that backs a [`SpellCardBackground`] entity.
///
/// `intensity` increases at `2.0 units/s` (≈ 0.5 s fade-in from zero).
/// Runs unconditionally while [`AppState::Playing`] so the animation is
/// smooth even if the boss phase timer is paused.
pub fn update_spell_card_bg_time(
    time: Res<Time>,
    bg_entities: Query<&MeshMaterial2d<SpellCardBgMaterial>, With<SpellCardBackground>>,
    mut spell_materials: ResMut<Assets<SpellCardBgMaterial>>,
) {
    let t = time.elapsed_secs();
    let dt = time.delta_secs();
    for handle in &bg_entities {
        let Some(mat) = spell_materials.get_mut(handle) else {
            continue;
        };
        mat.time = t;
        mat.intensity = (mat.intensity + dt * 2.0).min(1.0);
    }
}
