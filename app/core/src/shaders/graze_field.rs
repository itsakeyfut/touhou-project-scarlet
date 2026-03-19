use bevy::{
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d},
};

/// Custom material that renders a procedural electric graze-field ring.
///
/// Attach to a `Mesh2d(Circle::new(16.0))` child entity of the player.
/// The ring represents the 16 px graze detection zone:
///
/// - **Normal mode** — barely visible ring (alpha ≈ 0.15 scale) so it does
///   not distract the player during normal play.
/// - **Slow mode** (Shift held) — ring is clearly visible (alpha ≈ 0.6 scale),
///   helping the player thread bullets precisely.
/// - **Graze event** — `graze_intensity` spikes to `1.0` and decays at
///   `5.0/s`, producing a short electric spark animation.
///
/// # Struct layout (must match `assets/shaders/graze_field.wgsl`, 16 bytes)
///
/// | Offset | Field             | WGSL type |
/// |--------|-------------------|-----------|
/// | 0      | `time`            | `f32`     |
/// | 4      | `graze_intensity` | `f32`     |
/// | 8      | `slow_mode`       | `u32`     |
/// | 12     | `_padding`        | `f32`     |
/// | 16     | (end)             |           |
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct GrazeMaterial {
    /// Elapsed time in seconds — updated every frame by
    /// [`crate::shaders::plugin::update_graze_material`].
    #[uniform(0)]
    pub time: f32,
    /// Graze flash intensity in `[0.0, 1.0]`.
    ///
    /// Set to `1.0` on a graze event; decays at 5.0/s between events.
    #[uniform(0)]
    pub graze_intensity: f32,
    /// `1` while Shift is held (focus/slow mode), `0` otherwise.
    #[uniform(0)]
    pub slow_mode: u32,
    /// Struct padding — not used directly.
    #[uniform(0)]
    pub _padding: f32,
}

impl Default for GrazeMaterial {
    fn default() -> Self {
        Self {
            time: 0.0,
            graze_intensity: 0.0,
            slow_mode: 0,
            _padding: 0.0,
        }
    }
}

impl Material2d for GrazeMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/graze_field.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
