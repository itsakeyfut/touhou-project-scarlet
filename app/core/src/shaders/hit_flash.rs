use bevy::{
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d},
};

/// Material that briefly flashes an entity white when it is struck.
///
/// Attach to a `Mesh2d` entity (boss or enemy body). When a
/// [`crate::events::BossHitEvent`] is received, the system
/// [`crate::shaders::plugin::trigger_boss_hit_flash`] sets
/// `flash_intensity = 1.0`. Each subsequent frame,
/// [`crate::shaders::plugin::update_hit_flash`] decays the value at
/// `8.0 units/s`, producing a ≈ 0.125 s white-flash effect.
///
/// # WGSL
///
/// Driven by `assets/shaders/hit_flash.wgsl`. The shader linearly
/// interpolates `color_tint` toward HDR white (`3.0, 3.0, 3.0`) using
/// `flash_intensity` as the blend factor.
///
/// # Layout (must match `hit_flash.wgsl`, 32 bytes, std140)
///
/// | Offset | Field             | WGSL type   |
/// |--------|-------------------|-------------|
/// |  0     | `color_tint`      | `vec4<f32>` |
/// | 16     | `flash_intensity` | `f32`       |
/// | 20     | `_pad0`           | `f32`       |
/// | 24     | `_pad1`           | `f32`       |
/// | 28     | `_pad2`           | `f32`       |
/// | 32     | (end)             |             |
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct HitFlashMaterial {
    /// Base RGBA colour of the entity in linear space.
    ///
    /// Set this to the character's representative colour when spawning.
    /// Normal sprites use values ≤ 1.0; HDR bloom requires > 1.0.
    #[uniform(0)]
    pub color_tint: LinearRgba,
    /// Flash blend factor in `[0.0, 1.0]`.
    ///
    /// `0.0` — original colour; `1.0` — fully white.
    /// Set to `1.0` on each hit and decayed by
    /// [`crate::shaders::plugin::update_hit_flash`].
    #[uniform(0)]
    pub flash_intensity: f32,
    /// Padding — not used by the shader.
    #[uniform(0)]
    _pad0: f32,
    /// Padding — not used by the shader.
    #[uniform(0)]
    _pad1: f32,
    /// Padding — not used by the shader.
    #[uniform(0)]
    _pad2: f32,
}

impl Default for HitFlashMaterial {
    fn default() -> Self {
        Self {
            color_tint: LinearRgba::new(0.8, 0.8, 0.8, 1.0),
            flash_intensity: 0.0,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
        }
    }
}

impl Material2d for HitFlashMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/hit_flash.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
