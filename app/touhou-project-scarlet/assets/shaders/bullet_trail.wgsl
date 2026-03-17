#import bevy_sprite::mesh2d_vertex_output::VertexOutput

// Matches the Rust BulletTrailMaterial struct layout (32 bytes, 16-byte aligned).
// The mesh is a tall rectangle oriented along the bullet's travel direction.
// UV convention:
//   uv.y = 0.0  →  bullet tip  (fully opaque)
//   uv.y = 1.0  →  trail tail  (fully transparent)
//   uv.x = 0.0 / 1.0  →  left / right edge of the trail ribbon
struct BulletTrailMaterial {
    color: vec4<f32>,        // trail colour; can be HDR for bloom
    length: f32,             // UV-space scale of the trail (1.0 = full quad height)
    alpha_falloff: f32,      // exponent controlling how sharply the tail fades
                             //   2.0 = gradual, 5.0 = sharp
    time: f32,               // elapsed seconds (reserved for future shimmer effects)
    _pad: f32,               // padding to reach 32-byte struct size
}

@group(2) @binding(0)
var<uniform> material: BulletTrailMaterial;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Longitudinal fade: uv.y scaled by length, then raised to alpha_falloff.
    let t = clamp(in.uv.y * material.length, 0.0, 1.0);
    let trail_alpha = pow(1.0 - t, material.alpha_falloff);

    // Lateral soft-edge: smoothly narrows the ribbon toward 0 at the sides.
    let horiz = abs(in.uv.x - 0.5) * 2.0;   // 0 at centre, 1 at edge
    let width_alpha = max(0.0, 1.0 - horiz * 3.0);

    let alpha = trail_alpha * width_alpha * material.color.a;
    if alpha < 0.001 {
        discard;
    }

    return vec4<f32>(material.color.rgb, alpha);
}
