use bevy::{
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d},
};

/// Custom material that renders a velocity-aligned fade trail behind a bullet.
///
/// Attach to a vertically-oriented `Rectangle` mesh that is rotated to match
/// the bullet's travel direction. The WGSL shader treats `uv.y = 0` as the
/// tip (opaque) and `uv.y = 1` as the tail (transparent).
///
/// # Layout (must match `assets/shaders/bullet_trail.wgsl`)
///
/// | Offset | Field          | WGSL type   |
/// |--------|----------------|-------------|
/// | 0      | `color`        | `vec4<f32>` |
/// | 16     | `length`       | `f32`       |
/// | 20     | `alpha_falloff`| `f32`       |
/// | 24     | `time`         | `f32`       |
/// | 28     | `_pad`         | `f32`       |
/// | 32     | (end)          |             |
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct BulletTrailMaterial {
    /// Colour of the trail (can be HDR for bloom).
    #[uniform(0)]
    pub color: LinearRgba,
    /// UV-space scale of the trail along `uv.y`.
    /// `1.0` fills the full quad height; `0.5` makes the trail half as long.
    #[uniform(0)]
    pub length: f32,
    /// Exponent controlling how sharply the tail fades.
    /// `2.0` = gradual; `5.0` = sharp cutoff. Default: `3.0`.
    #[uniform(0)]
    pub alpha_falloff: f32,
    /// Elapsed time in seconds (reserved for future shimmer effects).
    #[uniform(0)]
    pub time: f32,
    /// Struct padding — do not use directly.
    #[uniform(0)]
    pub _pad: f32,
}

impl Default for BulletTrailMaterial {
    fn default() -> Self {
        Self {
            color: LinearRgba::new(1.0, 0.5, 0.1, 0.7),
            length: 1.0,
            alpha_falloff: 3.0,
            time: 0.0,
            _pad: 0.0,
        }
    }
}

impl Material2d for BulletTrailMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/bullet_trail.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
