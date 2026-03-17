use bevy::{
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d},
};

/// Custom material that renders a procedural radial glow around a bullet.
///
/// Used with `Mesh2d(Circle::new(r))` — no sprite texture required.
/// In Phase 19 a texture binding will be added and the WGSL updated to
/// sample a sprite sheet instead.
///
/// # HDR & Bloom
///
/// Requires the camera to have `Camera { hdr: true, ..default() }` and
/// a `Bloom` component. Set `color` values > 1.0 to trigger bloom.
///
/// # Layout (must match `assets/shaders/bullet_glow.wgsl`)
///
/// | Offset | Field           | WGSL type   |
/// |--------|-----------------|-------------|
/// | 0      | `color`         | `vec4<f32>` |
/// | 16     | `glow_intensity`| `f32`       |
/// | 20     | `time`          | `f32`       |
/// | 24     | `_pad`          | `vec2<f32>` |
/// | 32     | (end)           |             |
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct BulletGlowMaterial {
    /// Emissive colour in linear HDR space.
    /// Values > 1.0 produce bloom when the camera has `hdr: true` + `Bloom`.
    #[uniform(0)]
    pub color: LinearRgba,
    /// Multiplier applied on top of the radial distance falloff.
    /// Default: `1.5`. Increase for brighter bullets.
    #[uniform(0)]
    pub glow_intensity: f32,
    /// Elapsed time in seconds — updated every frame by
    /// [`crate::shaders::plugin::update_bullet_glow_time`].
    #[uniform(0)]
    pub time: f32,
    /// Struct padding — do not use directly.
    #[uniform(0)]
    pub _pad: Vec2,
}

impl Default for BulletGlowMaterial {
    fn default() -> Self {
        Self {
            color: LinearRgba::new(1.0, 0.2, 0.2, 1.0),
            glow_intensity: 1.5,
            time: 0.0,
            _pad: Vec2::ZERO,
        }
    }
}

impl Material2d for BulletGlowMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/bullet_glow.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
