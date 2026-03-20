// Spell-card background shader — rendered on a full-play-area Mesh2d
// (384 × 448) while a boss spell card is active.
//
// `pattern_id` selects the procedural pattern:
//   0 = swirl        (Rumia    — dark vortex)
//   1 = snowflake    (Cirno    — six-fold crystal)
//   2 = ripple       (Meiling  — concentric rings)
//   3 = sigil        (Patchouli — magic circle)
//   4 = clock        (Sakuya   — rotating gears)
//   5 = bat          (Remilia  — crimson wings)
//   6 = kaleidoscope (Flandre  — prismatic symmetry)
//
// `primary_color` / `secondary_color` are set per-boss in `BossType::spell_card_colors`.
// `intensity` fades from 0.0 to 1.0 on spell-card start.
// `time` is updated every frame by `update_spell_card_bg_time`.
//
// Must match `SpellCardBgMaterial` in `app/core/src/shaders/spell_card_bg.rs`.
//
// Uniform layout (48 bytes, std140):
//   offset  0 : time            f32         — elapsed seconds
//   offset  4 : pattern_id      u32         — selects pattern branch
//   offset  8 : intensity       f32         — fade-in multiplier [0.0, 1.0]
//   offset 12 : _pad            f32         — unused padding
//   offset 16 : primary_color   vec4<f32>
//   offset 32 : secondary_color vec4<f32>

#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import "shaders/common/math.wgsl"::{TAU, rotate2d}
#import "shaders/common/noise.wgsl"::{fbm}

struct SpellCardBgMaterial {
    time:            f32,
    pattern_id:      u32,
    intensity:       f32,
    _pad:            f32,
    primary_color:   vec4<f32>,
    secondary_color: vec4<f32>,
}

@group(2) @binding(0)
var<uniform> material: SpellCardBgMaterial;

// ---------------------------------------------------------------------------
// Pattern functions — each returns a scalar in [0.0, 1.0].
// ---------------------------------------------------------------------------

/// Spiral dark vortex (Rumia).
fn pattern_swirl(uv: vec2<f32>, t: f32) -> f32 {
    let c = uv - 0.5;
    let angle = atan2(c.y, c.x) + t * 0.5;
    let dist = length(c);
    let spiral = fract(angle / TAU + dist * 3.0 - t * 0.3);
    return smoothstep(0.4, 0.6, spiral) * max(0.0, 1.0 - dist * 1.8);
}

/// Six-fold snowflake crystal (Cirno).
fn pattern_snowflake(uv: vec2<f32>, t: f32) -> f32 {
    let c = uv - 0.5;
    let dist = length(c);
    let angle = atan2(c.y, c.x);
    // 6-fold symmetry.
    let sym_angle = fract(angle / TAU * 6.0) * TAU / 6.0;
    let petal = abs(cos(sym_angle * 3.0)) * max(0.0, 1.0 - dist * 2.5);
    return petal * (0.5 + 0.5 * sin(dist * 20.0 - t * 4.0));
}

/// Concentric ripple rings (Meiling).
fn pattern_ripple(uv: vec2<f32>, t: f32) -> f32 {
    let c = uv - 0.5;
    let dist = length(c);
    let rings = sin(dist * 30.0 - t * 5.0) * 0.5 + 0.5;
    return rings * max(0.0, 1.0 - dist * 1.6);
}

/// Rotating magic circle / sigil (Patchouli).
fn pattern_sigil(uv: vec2<f32>, t: f32) -> f32 {
    let c = uv - 0.5;
    // Dual rotation: outer ring spins faster.
    let inner = rotate2d(c, t * 0.4);
    let outer = rotate2d(c, -t * 0.7);
    let dist  = length(c);
    // Concentric glyphs via angular saw wave.
    let inner_glyph = abs(sin(atan2(inner.y, inner.x) * 7.0)) * max(0.0, 0.6 - dist * 2.0);
    let outer_glyph = abs(sin(atan2(outer.y, outer.x) * 12.0)) * max(0.0, 1.0 - dist * 1.8) * 0.5;
    return clamp(inner_glyph + outer_glyph, 0.0, 1.0);
}

/// Clock gear with rotating teeth (Sakuya).
fn pattern_clock(uv: vec2<f32>, t: f32) -> f32 {
    let c = uv - 0.5;
    let dist  = length(c);
    let angle = atan2(c.y, c.x);
    let teeth = abs(sin(angle * 16.0 + t * 2.0));
    let rim   = smoothstep(0.02, 0.0, abs(dist - 0.38) - 0.02) * teeth;
    let hub   = max(0.0, 1.0 - dist * 6.0);
    // Inner spokes.
    let spoke = abs(sin(angle * 8.0 - t)) * max(0.0, 0.4 - dist * 2.0);
    return clamp(rim + hub + spoke, 0.0, 1.0);
}

/// Bat-wing silhouette (Remilia).
fn pattern_bat(uv: vec2<f32>, t: f32) -> f32 {
    let c = uv - 0.5;
    let dist  = length(c);
    let angle = atan2(c.y, c.x) + t * 0.15;
    // 2-fold symmetry (two wings).
    let sym = abs(fract(angle / TAU * 2.0) - 0.5) * 2.0;
    let wing = (1.0 - sym) * max(0.0, 1.0 - dist * 2.2);
    // Jagged wing edge.
    let edge = abs(sin(dist * 18.0 - t * 3.0)) * 0.3;
    return clamp(wing * 0.6 + edge * wing, 0.0, 1.0);
}

/// Eight-fold kaleidoscope (Flandre).
fn pattern_kaleidoscope(uv: vec2<f32>, t: f32) -> f32 {
    let c = uv - 0.5;
    let dist  = length(c);
    let angle = atan2(c.y, c.x) + t * 0.2;
    // 8-fold symmetry.
    let sym = fract(angle / TAU * 8.0) * TAU / 8.0;
    let v = abs(sin(sym * 5.0 + t)) * abs(cos(dist * 15.0 - t * 2.0));
    return v * max(0.0, 1.0 - dist * 1.5);
}

// ---------------------------------------------------------------------------
// Fragment entry point
// ---------------------------------------------------------------------------

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let t  = material.time;

    var pat: f32;
    switch material.pattern_id {
        case 0u: { pat = pattern_swirl(uv, t); }
        case 1u: { pat = pattern_snowflake(uv, t); }
        case 2u: { pat = pattern_ripple(uv, t); }
        case 3u: { pat = pattern_sigil(uv, t); }
        case 4u: { pat = pattern_clock(uv, t); }
        case 5u: { pat = pattern_bat(uv, t); }
        case 6u: { pat = pattern_kaleidoscope(uv, t); }
        default: { pat = pattern_swirl(uv, t); }
    }

    // Layer FBM noise for organic texture variation.
    let noise = fbm(uv * 4.0 + t * 0.1, 3);
    let combined = clamp(pat + noise * 0.2, 0.0, 1.0) * material.intensity;

    let col = mix(material.secondary_color, material.primary_color, combined);
    return vec4<f32>(col.rgb, col.a * combined);
}
