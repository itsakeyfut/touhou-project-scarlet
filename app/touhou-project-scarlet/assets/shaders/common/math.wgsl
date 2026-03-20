// Common math utilities shared across WGSL shaders.
//
// Import individual constants or functions with Bevy's naga_oil syntax:
//   #import "shaders/common/math.wgsl"::{TAU, rotate2d}

#define_import_path "shaders/common/math.wgsl"

/// Full turn in radians (2π ≈ 6.283185).
const TAU: f32 = 6.283185307179586;

/// Rotate a 2-D vector `v` counter-clockwise by `angle` radians.
fn rotate2d(v: vec2<f32>, angle: f32) -> vec2<f32> {
    let s = sin(angle);
    let c = cos(angle);
    return vec2<f32>(c * v.x - s * v.y, s * v.x + c * v.y);
}
