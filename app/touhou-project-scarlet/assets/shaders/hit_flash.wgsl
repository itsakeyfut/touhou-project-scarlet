// Hit-flash shader — applied to boss/enemy mesh when struck by a player bullet.
//
// Blends the entity's base colour toward pure white proportionally to
// `flash_intensity` (1.0 = fully white, 0.0 = original colour). The system
// `update_hit_flash` fades `flash_intensity` back to 0.0 at ~8 units/s so the
// effect lasts ≈ 0.125 s per hit.
//
// Must match `HitFlashMaterial` in `app/core/src/shaders/hit_flash.rs`.
//
// Uniform layout (32 bytes, std140):
//   offset  0 : color_tint      vec4<f32>   — base RGBA of the entity
//   offset 16 : flash_intensity f32         — blend factor [0.0, 1.0]
//   offset 20 : _pad0           f32         — unused padding
//   offset 24 : _pad1           f32         — unused padding
//   offset 28 : _pad2           f32         — unused padding

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct HitFlashMaterial {
    color_tint:      vec4<f32>,
    flash_intensity: f32,
    _pad0:           f32,
    _pad1:           f32,
    _pad2:           f32,
}

@group(2) @binding(0)
var<uniform> material: HitFlashMaterial;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let white = vec4<f32>(3.0, 3.0, 3.0, material.color_tint.a);
    return mix(material.color_tint, white, material.flash_intensity);
}
