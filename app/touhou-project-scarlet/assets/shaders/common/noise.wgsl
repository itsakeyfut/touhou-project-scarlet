// Common noise utilities shared across WGSL shaders.
//
// Import individual functions with Bevy's naga_oil syntax:
//   #import "shaders/common/noise.wgsl"::{hash21, value_noise, fbm}

#define_import_path "shaders/common/noise.wgsl"

/// Pseudo-random hash: maps a 2-D point to a scalar in [0.0, 1.0).
///
/// Based on the classic "fract-sin" trick, cheap and sufficient for
/// procedural visual effects (electrical fields, sparkle patterns, etc.).
/// Not suitable for cryptographic or statistical randomness.
fn hash21(p: vec2<f32>) -> f32 {
    var q = fract(p * vec2<f32>(127.1, 311.7));
    q += dot(q, q + 19.19);
    return fract(q.x * q.y);
}

/// Smooth value noise in [0.0, 1.0) over a 2-D domain.
///
/// Uses bilinear interpolation between hashed corner values.
/// More visually coherent than `hash21` alone for background effects.
fn value_noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    // Smoothstep for C¹ continuity.
    let u = f * f * (3.0 - 2.0 * f);
    let a = hash21(i);
    let b = hash21(i + vec2<f32>(1.0, 0.0));
    let c = hash21(i + vec2<f32>(0.0, 1.0));
    let d = hash21(i + vec2<f32>(1.0, 1.0));
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

/// Fractional Brownian Motion — sums `octaves` octaves of `value_noise`.
///
/// Each octave doubles the frequency and halves the amplitude (lacunarity=2,
/// gain=0.5). The result is in approximately [0.0, 1.0].
/// Suitable for cloud-like or turbulent background textures.
fn fbm(p: vec2<f32>, octaves: i32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;
    for (var i = 0; i < octaves; i++) {
        value += amplitude * value_noise(p * frequency);
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    return value;
}
