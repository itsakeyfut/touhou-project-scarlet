// Reimu bomb effect — "Fantasy Seal" (封魔陣)
//
// Full-play-area Mesh2d effect rendered while Reimu's bomb is active.
// Interior: white light explosion + counter-rotating 六芒星 seal overlay.
// Barrier:  three-layer HDR-like hexagonal ring with golden seal glow.
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

    // UV centred at (0,0), range [-0.5, 0.5].
    let c     = in.uv - vec2<f32>(0.5, 0.5);
    let dist  = length(c);
    let angle = atan2(c.y, c.x);

    let ring_r = material.expand_radius * 0.45;
    let safe_r = max(ring_r, 0.001);

    // ─── Rotating hexagonal SDF — barrier ring ────────────────────────────────
    let hrs = sin(t * 0.18);
    let hrc = cos(t * 0.18);
    let hx  = hrc * c.x - hrs * c.y;
    let hy  = hrs * c.x + hrc * c.y;
    let hq  = abs(vec2<f32>(hx, hy));
    let hex_d = max(hq.x * 0.866025 + hq.y * 0.5, hq.y);

    // Three-layer bloom ring — simulates HDR glow without a post-process pass.
    let ring_core  = max(0.0, 1.0 - abs(hex_d - ring_r) / 0.007);
    let ring_mid   = max(0.0, 1.0 - abs(hex_d - ring_r) / 0.030);
    let ring_bloom = max(0.0, 1.0 - abs(hex_d - ring_r) / 0.090);

    let pulse       = 0.92 + 0.08 * sin(t * 3.8);
    let scintillate = 0.85 + 0.15 * (sin(angle * 23.0 + t * 6.1)
                                    * sin(angle * 13.0 - t * 4.3));
    let ring_alpha  = ring_core * scintillate * pulse
                    + ring_mid  * 0.55
                    + ring_bloom * 0.18;

    // Inside/outside mask + interior edge fade.
    // inner_fade → 0 at the ring surface so the ring glow dominates there.
    let inside_f   = max(0.0, sign(safe_r - hex_d));
    let inner_fade = 1.0 - smoothstep(safe_r * 0.78, safe_r, hex_d);

    // Six "ofuda" (御札) golden flares at the hex ring vertices.
    let hex_angle_r = atan2(hy, hx);
    let ofuda_fold  = fract(hex_angle_r / 1.0472);
    let ofuda       = pow(max(0.0, cos(ofuda_fold * 6.2832)), 10.0) * ring_core;

    // ─── Interior 六芒星 (Star of David) seal — counter-rotating ─────────────
    // The seal uses a different rotation rate so it spins opposite the outer ring.
    let k3  = 1.7320508; // sqrt(3)
    let iss = -sin(t * 0.12);
    let isc = cos(t * 0.12);
    let sx  = isc * c.x - iss * c.y;
    let sy  = iss * c.x + isc * c.y;

    // Star tips at 75 % of barrier radius.
    // IQ triangle SDF: circumradius = 2*ssr/k3, so ssr = star_r * k3/2.
    let star_r = safe_r * 0.75;
    let ssr    = star_r * 0.866025;

    // Triangle 1 — pointing up.
    var a1x = abs(sx) - ssr;
    var a1y = sy + ssr / k3;
    if a1x + k3 * a1y > 0.0 {
        let nx = (a1x - k3 * a1y) / 2.0;
        let ny = (-k3 * a1x - a1y) / 2.0;
        a1x = nx;
        a1y = ny;
    }
    a1x = a1x - clamp(a1x, -2.0 * ssr, 0.0);
    let tri1_d = -length(vec2<f32>(a1x, a1y)) * sign(a1y);

    // Triangle 2 — pointing down (negate sy).
    var a2x = abs(sx) - ssr;
    var a2y = -sy + ssr / k3;
    if a2x + k3 * a2y > 0.0 {
        let nx = (a2x - k3 * a2y) / 2.0;
        let ny = (-k3 * a2x - a2y) / 2.0;
        a2x = nx;
        a2y = ny;
    }
    a2x = a2x - clamp(a2x, -2.0 * ssr, 0.0);
    let tri2_d = -length(vec2<f32>(a2x, a2y)) * sign(a2y);

    // Glowing outline of both triangle edges = hexagram lines.
    let seal_w    = 0.010; // line core width in UV space
    let seal_line = max(
        max(0.0, 1.0 - abs(tri1_d) / seal_w),
        max(0.0, 1.0 - abs(tri2_d) / seal_w)
    );
    let seal_glow = max(
        max(0.0, 1.0 - abs(tri1_d) / (seal_w * 4.0)),
        max(0.0, 1.0 - abs(tri2_d) / (seal_w * 4.0))
    );

    // ─── Interior: white light explosion + spiritual ripples ──────────────────

    // Normalised radial distance inside the barrier (0 = player, 1 = ring edge).
    let r = dist / safe_r;

    // 1. Centre light burst.
    let center_glow = pow(max(0.0, 1.0 - r), 1.8);

    // 2. Radial light shafts — twelve blades from two counter-rotating 6-fold waves.
    let shaft_a = pow(abs(cos(angle * 3.0 + t * 0.35)), 5.0);
    let shaft_b = pow(abs(cos(angle * 3.0 - t * 0.25 + 1.047)), 5.0);
    let shaft_c = pow(abs(cos(angle * 6.0 + t * 0.55)), 3.0) * 0.45;
    let shafts  = (max(shaft_a, shaft_b) + shaft_c) * max(0.0, 1.0 - r * 1.1);

    // 3. Spiritual ripple rings.
    let ripple_a = sin(r * 32.0 - t * 6.0) * 0.5 + 0.5;
    let ripple_b = sin(r * 20.0 - t * 4.0 + 1.2) * 0.5 + 0.5;
    let ripples  = (ripple_a * 0.65 + ripple_b * 0.35) * max(0.0, 1.0 - r);

    // 4. Angular shimmer in the mid zone.
    let shimmer = (sin(angle * 24.0 + t * 5.0) * 0.5 + 0.5)
                * (sin(angle * 11.0 - t * 3.5) * 0.5 + 0.5);
    let shimmer_band    = smoothstep(0.2, 0.55, r) * max(0.0, 1.0 - r * 1.4);
    let shimmer_contrib = shimmer * 0.14 * shimmer_band;

    // ─── Interior colour ──────────────────────────────────────────────────────
    let pure_white = vec3<f32>(1.00, 1.00, 1.00);
    let warm_ivory = vec3<f32>(0.99, 0.95, 0.88);
    let blush_pink = vec3<f32>(0.98, 0.87, 0.89);
    let pale_gold  = vec3<f32>(1.00, 0.95, 0.72);
    let seal_col   = vec3<f32>(0.90, 0.12, 0.22); // crimson seal lines

    let col_r0    = mix(pure_white, warm_ivory, smoothstep(0.0, 0.5, r));
    let col_r1    = mix(col_r0,    blush_pink,  smoothstep(0.5, 0.9, r));
    let inner_col = col_r1 + shafts * pale_gold * 0.10;
    // Tint toward crimson where the hexagram lines are bright.
    let inner_col_seal = mix(inner_col, seal_col, clamp(seal_line * 1.8, 0.0, 1.0));

    // ─── Interior alpha ───────────────────────────────────────────────────────
    let inside_alpha = clamp(
        (center_glow * 0.72 + shafts * 0.45 + ripples * 0.12 + shimmer_contrib)
        * inside_f * inner_fade,
        0.0, 1.0
    );
    // Seal glow adds its own alpha on top of the white explosion.
    let seal_alpha = clamp(seal_glow * 0.75 * inside_f * inner_fade, 0.0, 1.0);

    // ─── Ring colour ──────────────────────────────────────────────────────────
    let gold    = vec3<f32>(1.00, 0.85, 0.28);
    let crimson = vec3<f32>(0.86, 0.06, 0.20);
    let ring_col = mix(crimson, gold, min(1.0, ring_core * scintillate + ofuda * 0.7));

    // ─── Alpha composition ────────────────────────────────────────────────────
    // Very faint outer bloom halo visible just outside the barrier.
    let outer_alpha = ring_bloom * (1.0 - inside_f) * 0.14;

    let total_alpha = clamp(
        (inside_alpha + seal_alpha + ring_alpha + outer_alpha) * intens,
        0.0, 1.0
    );

    // ─── Final colour blend ───────────────────────────────────────────────────
    // Inside the barrier: white+seal interior blends toward ring gold at the surface.
    // Outside the barrier: pure ring glow — interior never bleeds outward.
    let interior_blend = mix(inner_col_seal, ring_col, ring_mid);
    let final_col      = mix(ring_col, interior_blend, inside_f);

    return vec4<f32>(final_col, total_alpha);
}
