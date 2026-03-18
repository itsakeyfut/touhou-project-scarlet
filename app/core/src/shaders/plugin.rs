use bevy::{prelude::*, sprite_render::Material2dPlugin};

use crate::{
    shaders::{BulletGlowMaterial, BulletTrailMaterial},
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

        // Uniform time updates — only while the game is running.
        app.add_systems(
            Update,
            (update_bullet_glow_time, update_bullet_trail_time).run_if(in_state(AppState::Playing)),
        );

        // TODO(phase-05): add Material2dPlugin::<GrazeMaterial>
        // TODO(phase-08): add Material2dPlugin::<SpellCardBgMaterial>, HitFlashMaterial
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
