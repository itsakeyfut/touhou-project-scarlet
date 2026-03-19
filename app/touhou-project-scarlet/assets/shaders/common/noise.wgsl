// Common noise utilities shared across WGSL shaders.
//
// Import individual functions with Bevy's naga_oil syntax:
//   #import "shaders/common/noise.wgsl"::hash21

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
