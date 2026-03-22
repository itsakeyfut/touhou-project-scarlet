// Reimu bomb effect — "Fantasy Seal" (封魔陣)
//
// Full-play-area Mesh2d effect rendered while Reimu's bomb is active.
// Renders a hexagonal barrier that expands from the player and fades out.
//
// Uniforms: see `BombReimuMaterial` in `app/core/src/shaders/bomb_reimu.rs`.

#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import "shaders/common/math.wgsl"::{TAU, rotate2d}

struct BombReimuMaterial {
    /// Seconds elapsed since the bomb was activated.
    time: f32,
    /// Fade multiplier in [0.0, 1.0]. 1.0 = fully visible, 0.0 = invisible.
    intensity: f32,
    /// Expand progress in [0.0, 1.0]; drives the barrier radius.
    expand_radius: f32,
    /// Struct padding.
    _padding: f32,
}

@group(2) @binding(0)
var<uniform> material: BombReimuMaterial;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let center = in.uv - vec2<f32>(0.5, 0.5);
    let dist   = length(center);
    let angle  = atan2(center.y, center.x);
    let t      = material.time;

    // --- Hexagonal boundary ring ------------------------------------------
    // Map angle into one of six equal sectors, compute the "hex distance".
    let sector_angle = TAU / 6.0;
    let hex_angle    = fract(angle / TAU + 1.0) * TAU; // normalise to [0, TAU)
    let local_angle  = hex_angle - floor(hex_angle / sector_angle) * sector_angle;
    let hex_dist     = dist / cos(local_angle - sector_angle * 0.5);

    let ring_r   = material.expand_radius * 0.48;
    let ring_raw = abs(hex_dist - ring_r) - 0.008;
    let ring     = smoothstep(0.012, 0.0, ring_raw);

    // --- Inner yin-yang ripple pattern ------------------------------------
    // Animated sin rings modulated by the rotation angle, visible inside the ring.
    let spin    = angle * 6.0 + t * 2.5;
    let ripple  = 0.25 + 0.15 * sin(spin) * sin(dist * 14.0 - t * 1.5);
    let inner   = select(0.0, ripple, dist < ring_r);

    // --- Rotating 8-fold star rays ----------------------------------------
    let star_a  = fract(angle / TAU * 8.0 + t * 0.08);
    let ray     = pow(max(0.0, sin(star_a * TAU)), 8.0) * 0.18;
    let rays    = select(0.0, ray, dist < ring_r - 0.03);

    // --- Yin-yang colour split --------------------------------------------
    let yin_yang    = step(0.5, fract(angle / TAU + t * 0.06));
    let crimson     = vec3<f32>(2.0, 0.25, 0.35);  // HDR crimson (Reimu red)
    let ivory       = vec3<f32>(2.0, 1.9,  1.8);   // HDR near-white
    let base_color  = mix(crimson, ivory, yin_yang);

    // Bright golden ring edge
    let ring_color  = mix(base_color, vec3<f32>(3.0, 2.5, 0.5), ring);

    let alpha = clamp((ring + inner + rays) * material.intensity * (1.0 - dist * 1.5), 0.0, 1.0);
    return vec4<f32>(ring_color, alpha);
}
