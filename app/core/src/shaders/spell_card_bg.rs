use bevy::{
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d},
};

/// Marker component placed on the full-play-area background entity that is
/// spawned when a boss spell card becomes active.
///
/// Queried by [`crate::shaders::plugin::update_spell_card_bg_time`] to
/// locate the entity's [`MeshMaterial2d<SpellCardBgMaterial>`] handle.
/// Despawned automatically via [`crate::components::GameSessionEntity`]
/// when the game session ends.
#[derive(Component)]
pub struct SpellCardBackground;

/// Full-play-area procedural background rendered during a boss spell card.
///
/// A `Mesh2d(Rectangle::new(384.0, 448.0))` entity carrying this material
/// is spawned by `on_spell_card_start` (registered in phase-08) when a
/// [`crate::events::BossPhaseChangedEvent`] indicates an `is_spell_card`
/// phase. It is placed at `z = -0.5` so it renders behind gameplay entities.
///
/// Seven procedural patterns — selected by `pattern_id` — are implemented in
/// `assets/shaders/spell_card_bg.wgsl`:
///
/// | `pattern_id` | Pattern       | Boss       |
/// |:---:|---|---|
/// | 0 | Dark vortex (swirl)        | Rumia      |
/// | 1 | Six-fold snowflake crystal | Cirno      |
/// | 2 | Concentric ripple rings    | Meiling    |
/// | 3 | Rotating magic circle      | Patchouli  |
/// | 4 | Clock gear                 | Sakuya     |
/// | 5 | Bat-wing silhouette        | Remilia    |
/// | 6 | Eight-fold kaleidoscope    | Flandre    |
///
/// `primary_color` and `secondary_color` are supplied by
/// [`crate::components::BossType::spell_card_colors`].
///
/// # Layout (must match `spell_card_bg.wgsl`, 48 bytes, std140)
///
/// | Offset | Field            | WGSL type   |
/// |--------|------------------|-------------|
/// |  0     | `time`           | `f32`       |
/// |  4     | `pattern_id`     | `u32`       |
/// |  8     | `intensity`      | `f32`       |
/// | 12     | `_pad`           | `f32`       |
/// | 16     | `primary_color`  | `vec4<f32>` |
/// | 32     | `secondary_color`| `vec4<f32>` |
/// | 48     | (end)            |             |
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct SpellCardBgMaterial {
    /// Elapsed time in seconds — updated every frame by
    /// [`crate::shaders::plugin::update_spell_card_bg_time`].
    #[uniform(0)]
    pub time: f32,
    /// Pattern selector (0–6). See struct-level docs for the mapping.
    #[uniform(0)]
    pub pattern_id: u32,
    /// Fade-in multiplier in `[0.0, 1.0]`.
    ///
    /// Set to `0.0` on spawn and driven toward `1.0` by
    /// `update_spell_card_bg_time` over ≈ 0.5 s.
    #[uniform(0)]
    pub intensity: f32,
    /// Struct padding — not used by the shader.
    #[uniform(0)]
    pub _pad: f32,
    /// Dominant colour for the pattern (foreground).
    #[uniform(0)]
    pub primary_color: LinearRgba,
    /// Recessive colour for the pattern (background fill).
    #[uniform(0)]
    pub secondary_color: LinearRgba,
}

impl Default for SpellCardBgMaterial {
    /// Returns a Rumia-themed dark-vortex background at zero intensity.
    fn default() -> Self {
        Self {
            time: 0.0,
            pattern_id: 0,
            intensity: 0.0,
            _pad: 0.0,
            primary_color: LinearRgba::new(0.15, 0.05, 0.25, 0.7),
            secondary_color: LinearRgba::new(0.02, 0.01, 0.05, 0.9),
        }
    }
}

impl Material2d for SpellCardBgMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/spell_card_bg.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
