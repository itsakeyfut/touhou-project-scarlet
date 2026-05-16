// Reimu bomb effect — "Fantasy Seal" (封魔陣)
//
// Full-play-area Mesh2d effect rendered while Reimu's bomb is active.
// Renders a hexagonal barrier that expands from the player and fades out.
//
// Uniforms: see `BombReimuMaterial` in `app/core/src/shaders/bomb_reimu.rs`.

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct BombReimuMaterial {
    time: f32,
    intensity: f32,
    expand_radius: f32,
    _padding: f32,
}

@group(2) @binding(0)
var<uniform> material: BombReimuMaterial;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let t      = material.time;
    let intens = material.intensity;

    // Centre at (0,0). UV space is [-0.5, 0.5] x [-0.5, 0.5].
    let c     = in.uv - vec2<f32>(0.5, 0.5);
    let dist  = length(c);
    let angle = atan2(c.y, c.x);

    // Slowly rotate the hexagonal grid (manual rotation, no helper needed).
    let rs = sin(t * 0.25);
    let rc = cos(t * 0.25);
    let hx = rc * c.x - rs * c.y;
    let hy = rs * c.x + rc * c.y;

    // Hexagonal SDF — flat-top hex, inlined.
    // hex_d == 1.0 on the boundary of a unit hexagon; scales linearly.
    let hq    = abs(vec2<f32>(hx, hy));
    let hex_d = max(hq.x * 0.866025 + hq.y * 0.5, hq.y);

    // Expanding hexagonal ring. ring_r ranges 0.135 → 0.45 in UV hex-space.
    let ring_r = material.expand_radius * 0.45;
    let ring   = max(0.0, 1.0 - abs(hex_d - ring_r) / 0.04);

    // Soft inner glow (bright near center, fades toward ring boundary).
    let inside     = max(0.0, ring_r - hex_d) / max(ring_r, 0.001);
    let inner_glow = inside * inside * 0.28;

    // Six-pointed star rays: two 3-fold cosine patterns offset by 60° (1.047 rad).
    // Strongest at center, falls off toward the ring edge.
    let star_a = abs(cos(angle * 3.0 + t * 0.6));
    let star_b = abs(cos(angle * 3.0 - t * 0.4 + 1.047));
    let rays   = max(star_a, star_b) * max(0.0, 1.0 - dist * 3.5);

    // Crimson / ivory colour split driven by polar angle.
    // The angular wave rotates slowly to give a yin-yang shimmer.
    let hue = sin(angle - t * 0.35) * 0.5 + 0.5;
    let col = mix(
        vec3<f32>(0.86, 0.07, 0.24),   // crimson  (#DC1340)
        vec3<f32>(1.00, 0.97, 0.92),   // ivory    (#FFF7EB)
        hue
    );

    let alpha = clamp(
        (ring * 0.95 + rays * 0.35 + inner_glow) * intens,
        0.0, 1.0
    );
    return vec4<f32>(col, alpha);
}
