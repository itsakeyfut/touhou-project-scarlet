// Marisa bomb effect — "Master Spark" (マスタースパーク)
//
// Full-play-area Mesh2d effect rendered while Marisa's bomb is active.
// Renders a wide upward rainbow laser beam shooting from the player.
//
// Uniforms: see `BombMarisaMaterial` in `app/core/src/shaders/bomb_marisa.rs`.

#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import "shaders/common/noise.wgsl"::{hash21}

struct BombMarisaMaterial {
    /// Seconds elapsed since the bomb was activated.
    time: f32,
    /// Fade multiplier in [0.0, 1.0]. 1.0 = fully visible, 0.0 = invisible.
    intensity: f32,
    /// Spark width factor in [0.0, 1.0]; drives how wide the beam appears.
    width: f32,
    /// Struct padding.
    _padding: f32,
}

@group(2) @binding(0)
var<uniform> material: BombMarisaMaterial;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv   = in.uv;
    let t    = material.time;

    // uv.x = horizontal (0 = left, 1 = right), uv.y = vertical (0 = bottom, 1 = top).
    // The spark travels upward — uv.y = 0 is the base, 1 is the tip.
    let cx = 0.5;
    let dx = abs(uv.x - cx);

    // --- Beam edge with turbulent noise -----------------------------------
    // Wiggle the beam edges using noise for an electrical look.
    let edge_noise  = hash21(vec2<f32>(uv.y * 18.0, t * 6.0)) * 0.04
                    + hash21(vec2<f32>(uv.y * 37.0 + 0.5, t * 9.3 + 1.1)) * 0.02;
    let half_width  = material.width * 0.38 + edge_noise;

    let in_beam = dx < half_width;

    // --- Beam core intensity (brightest at centre, falls off to edges) -----
    let norm_dx = dx / max(half_width, 0.001);
    let core    = pow(max(0.0, 1.0 - norm_dx), 1.5);

    // Fade towards the tip of the beam (uv.y → 1) with a soft gradient.
    let tip_fade = 1.0 - smoothstep(0.7, 1.0, uv.y);
    // Grow the beam from the base over the first 0.2 s.
    let base_fade = smoothstep(0.0, 0.15, uv.y);

    // --- Rainbow colour ---------------------------------------------------
    // Hue cycles along the Y axis and drifts with time.
    let hue = uv.y * 0.6 + t * 0.25;
    let r   = 0.5 + 0.5 * sin(hue * TAU + 0.0);
    let g   = 0.5 + 0.5 * sin(hue * TAU + 2.094);
    let b   = 0.5 + 0.5 * sin(hue * TAU + 4.189);

    // Core centre is HDR white/yellow; outer edges take the rainbow hue.
    let rainbow    = vec3<f32>(r * 2.0, g * 2.0, b * 2.0);
    let beam_color = mix(rainbow, vec3<f32>(3.0, 3.0, 2.5), core * core);

    // High-frequency sparkle flicker inside the beam.
    let flicker = 0.85 + 0.15 * hash21(vec2<f32>(uv.x * 60.0 + t * 20.0, uv.y * 12.0));

    let alpha = select(
        0.0,
        core * tip_fade * base_fade * flicker * material.intensity,
        in_beam
    );

    return vec4<f32>(beam_color, clamp(alpha, 0.0, 1.0));
}

// TAU is not imported from math.wgsl to keep this shader self-contained.
const TAU: f32 = 6.283185307179586;
