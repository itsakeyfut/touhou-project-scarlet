use bevy::{
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d},
};

/// Marker component attached to the full-play-area visual entity spawned when
/// Reimu's bomb activates.
///
/// Queried by [`crate::shaders::plugin::update_bomb_reimu_material`] to locate
/// the entity's [`MeshMaterial2d<BombReimuMaterial>`] handle and sync uniforms
/// each frame.
///
/// Despawned by [`crate::shaders::plugin::despawn_finished_bomb_visuals`] once
/// [`crate::resources::BombState`] becomes inactive.
#[derive(Component)]
pub struct BombReimuVisual;

/// Full-play-area procedural material for Reimu's bomb effect "Fantasy Seal".
///
/// Attach this to a `Mesh2d(Rectangle::new(384.0, 448.0))` entity placed at
/// `z = 9.0` (above gameplay) so it covers the entire play area.
///
/// The shader (`assets/shaders/bomb_reimu.wgsl`) renders:
/// - A hexagonal barrier ring that expands from the player outward.
/// - Rotating yin-yang star rays inside the ring.
/// - Crimson/ivory colour split driven by the polar angle.
///
/// # Layout (must match `bomb_reimu.wgsl`, 16 bytes, std140)
///
/// | Offset | Field           | WGSL type |
/// |--------|-----------------|-----------|
/// |  0     | `time`          | `f32`     |
/// |  4     | `intensity`     | `f32`     |
/// |  8     | `expand_radius` | `f32`     |
/// | 12     | `_padding`      | `f32`     |
/// | 16     | (end)           |           |
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct BombReimuMaterial {
    /// Seconds elapsed since the bomb was activated.
    ///
    /// Drives all animations (ring pulse, star-ray rotation, yin-yang drift).
    /// Updated every frame by [`crate::shaders::plugin::update_bomb_reimu_material`].
    #[uniform(0)]
    pub time: f32,
    /// Fade multiplier in `[0.0, 1.0]`.
    ///
    /// `1.0` — fully opaque; `0.0` — transparent.
    /// Driven by the inverse of the bomb timer's fraction (fades out as the
    /// bomb effect expires).
    #[uniform(0)]
    pub intensity: f32,
    /// Hexagonal barrier expand progress in `[0.0, 1.0]`.
    ///
    /// `0.0` — barrier starts collapsed at the player; `1.0` — barrier fills
    /// the play area. Ramps from `0.0` to `1.0` over
    /// [`crate::resources::BOMB_DURATION_SECS`] to give an expanding feel.
    #[uniform(0)]
    pub expand_radius: f32,
    /// Struct padding — not used by the shader.
    #[uniform(0)]
    pub _padding: f32,
}

impl Default for BombReimuMaterial {
    fn default() -> Self {
        Self {
            time: 0.0,
            intensity: 1.0,
            expand_radius: 0.0,
            _padding: 0.0,
        }
    }
}

impl Material2d for BombReimuMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/bomb_reimu.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
