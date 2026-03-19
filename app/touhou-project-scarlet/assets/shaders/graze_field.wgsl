#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import "shaders/common/noise.wgsl"::hash21

// Must match GrazeMaterial in app/core/src/shaders/graze_field.rs (16 bytes).
struct GrazeMaterial {
    time: f32,
    graze_intensity: f32,
    slow_mode: u32,
    _padding: f32,
}

@group(2) @binding(0)
var<uniform> material: GrazeMaterial;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let center = vec2<f32>(0.5, 0.5);
    let uv     = in.uv - center;
    let dist   = length(uv);
    let angle  = atan2(uv.y, uv.x);

    // ---- Graze ring ----------------------------------------------------------
    // UV-space radius 0.45 ≈ the 16 px graze zone boundary on a Circle::new(16.0).
    let ring_r     = 0.45;
    let ring_width = 0.04;
    let ring       = max(0.0, 1.0 - abs(dist - ring_r) / ring_width);

    // Electrical jaggedness: modulate ring brightness with angular noise.
    let noise_input = vec2<f32>((angle + material.time * 3.0) * 5.0, material.time);
    let n           = hash21(noise_input);
    let jagged_ring = ring * (0.5 + 0.5 * n);

    // ---- Graze spark ---------------------------------------------------------
    // Bright radial flash at the graze event, centred on the player.
    let spark = material.graze_intensity * max(0.0, 1.0 - dist * 3.0);

    // ---- Visibility ----------------------------------------------------------
    // Normal mode: very faint (0.15 scale) so the player can see it without
    // it being distracting.  Slow mode (Shift): clearly visible (0.6 scale).
    let ring_vis   = jagged_ring * select(0.15, 0.6, material.slow_mode == 1u);
    let visibility = ring_vis + spark;

    if visibility < 0.001 {
        discard;
    }

    // Blue-white electrical colour; spark adds a warm yellow tint.
    let base_color = vec3<f32>(0.3, 0.7, 1.0);
    let spark_tint = spark * vec3<f32>(1.0, 1.0, 0.5);
    let color      = base_color + spark_tint;

    return vec4<f32>(color, visibility);
}
