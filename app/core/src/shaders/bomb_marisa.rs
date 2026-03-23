use bevy::{
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d},
};

/// Marker component attached to the full-play-area visual entity spawned when
/// Marisa's bomb activates.
///
/// Queried by [`crate::shaders::plugin::update_bomb_marisa_material`] to locate
/// the entity's [`MeshMaterial2d<BombMarisaMaterial>`] handle and sync uniforms
/// each frame.
///
/// Despawned by [`crate::shaders::plugin::despawn_finished_bomb_visuals`] once
/// [`crate::resources::BombState`] becomes inactive.
#[derive(Component)]
pub struct BombMarisaVisual;

/// Full-play-area procedural material for Marisa's bomb effect "Master Spark".
///
/// Attach this to a tall `Mesh2d(Rectangle::new(384.0, 448.0))` entity at
/// `z = 9.0` (above gameplay). The shader renders a wide rainbow laser beam
/// that fills the upper portion of the play area.
///
/// The shader (`assets/shaders/bomb_marisa.wgsl`) renders:
/// - A wide, edge-turbulent beam centred on the play area's horizontal middle.
/// - Rainbow colour cycling along the beam's length.
/// - High-frequency sparkle flicker that gives an electrical/magical feel.
///
/// # Layout (must match `bomb_marisa.wgsl`, 16 bytes, std140)
///
/// | Offset | Field       | WGSL type |
/// |--------|-------------|-----------|
/// |  0     | `time`      | `f32`     |
/// |  4     | `intensity` | `f32`     |
/// |  8     | `width`     | `f32`     |
/// | 12     | `_padding`  | `f32`     |
/// | 16     | (end)       |           |
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct BombMarisaMaterial {
    /// Seconds elapsed since the bomb was activated.
    ///
    /// Drives the rainbow hue drift and edge-noise animation.
    /// Updated every frame by [`crate::shaders::plugin::update_bomb_marisa_material`].
    #[uniform(0)]
    pub time: f32,
    /// Fade multiplier in `[0.0, 1.0]`.
    ///
    /// `1.0` — fully opaque; `0.0` — transparent.
    /// Driven by the inverse of the bomb timer's fraction (fades out as the
    /// bomb effect expires).
    #[uniform(0)]
    pub intensity: f32,
    /// Beam width factor in `[0.0, 1.0]`.
    ///
    /// `1.0` — beam fills the full play area width; `0.0` — beam is invisible.
    /// Ramps from `0.0` to `1.0` over the first ~0.3 s of the bomb to give
    /// an expanding beam feel.
    #[uniform(0)]
    pub width: f32,
    /// Struct padding — not used by the shader.
    #[uniform(0)]
    pub _padding: f32,
}

impl Default for BombMarisaMaterial {
    fn default() -> Self {
        Self {
            time: 0.0,
            intensity: 1.0,
            width: 0.0,
            _padding: 0.0,
        }
    }
}

impl Material2d for BombMarisaMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/bomb_marisa.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
