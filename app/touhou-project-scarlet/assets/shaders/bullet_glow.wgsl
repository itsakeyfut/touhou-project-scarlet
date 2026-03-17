#import bevy_sprite::mesh2d_vertex_output::VertexOutput

// Matches the Rust BulletGlowMaterial struct layout (32 bytes, 16-byte aligned).
struct BulletGlowMaterial {
    color: vec4<f32>,        // HDR colour; values > 1.0 produce bloom
    glow_intensity: f32,     // multiplier applied on top of the distance falloff
    time: f32,               // elapsed seconds — drives the pulse animation
    _pad: vec2<f32>,         // padding to reach 32-byte struct size
}

@group(2) @binding(0)
var<uniform> material: BulletGlowMaterial;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Distance from UV centre (0.5, 0.5).  Ranges 0..~0.7 across the quad.
    let dist = distance(in.uv, vec2<f32>(0.5, 0.5));

    // Soft radial falloff: full brightness at centre, zero at dist >= 0.5.
    let falloff = max(0.0, 1.0 - dist * 2.0);

    // Slow pulse — ±20 % amplitude at 6 rad/s (~1 Hz).
    let pulse = 1.0 + 0.2 * sin(material.time * 6.0);

    let glow = material.glow_intensity * pulse * falloff;

    // Discard fully transparent fragments to avoid overdraw.
    let alpha = falloff * glow;
    if alpha < 0.001 {
        discard;
    }

    return vec4<f32>(material.color.rgb * glow, alpha);
}
