// Crystalline hexagonal column field — FRAGMENT SHADER.
//
// Custom fragment shader for the `crystal` example. Renders hexagonal
// prisms with iridescent chromatic material, pulsating energy veins,
// and a radial cursor-reactive highlight.
//
// The ic3d prelude (SceneUniforms, VertexIn/Out, vs_main, shadow PCF)
// is prepended at runtime via `compose_shader()`.

// ──────────────────── Custom Uniforms (group 1) ────────────────────

struct CrystalUniforms {
    cursor_world: vec2<f32>,
    cursor_active: f32,
    _pad: f32,
}

@group(1) @binding(0) var<uniform> crystal: CrystalUniforms;

// ──────────────────── Hash / Noise ────────────────────

fn hash21(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

fn hash31(p: vec3<f32>) -> f32 {
    var q = fract(p * 0.1031);
    q += dot(q, q.yzx + 31.32);
    return fract((q.x + q.y) * q.z);
}

fn noise2d(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    let a = hash21(i);
    let b = hash21(i + vec2(1.0, 0.0));
    let c = hash21(i + vec2(0.0, 1.0));
    let d = hash21(i + vec2(1.0, 1.0));
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

// Fractal Brownian motion — 4 octaves
fn fbm(p: vec2<f32>) -> f32 {
    var v = 0.0;
    var a = 0.5;
    var q = p;
    for (var i = 0; i < 4; i++) {
        v += a * noise2d(q);
        q *= 2.01;
        a *= 0.5;
    }
    return v;
}

// ──────────────────── Iridescence ────────────────────

// Thin-film interference approximation — maps angle → rainbow shift.
fn iridescence(NdotV: f32, thickness: f32) -> vec3<f32> {
    let phase = 2.0 * NdotV * thickness;

    // Cosine palette: three phase-shifted cosines for RGB
    let r = 0.5 + 0.5 * cos(6.2832 * (phase + 0.00));
    let g = 0.5 + 0.5 * cos(6.2832 * (phase + 0.33));
    let b = 0.5 + 0.5 * cos(6.2832 * (phase + 0.67));

    return vec3(r, g, b);
}

// ──────────────────── Energy Veins ────────────────────

fn energy_veins(world_pos: vec3<f32>, time: f32) -> f32 {
    // Vertical veins traveling upward on side faces
    let vein_coord = world_pos.xy * 3.0 + vec2(0.0, -time * 0.8);
    let vein_noise = fbm(vein_coord * 2.5);

    // Sharp vein lines
    let vein_raw = smoothstep(0.42, 0.48, vein_noise);
    let vein_glow = smoothstep(0.35, 0.50, vein_noise) * 0.4;

    return vein_raw + vein_glow;
}

// ──────────────────── Fragment Shader ────────────────────

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let N = normalize(in.world_normal);
    let V = normalize(scene.camera_position - in.world_pos);
    let NdotV = max(dot(N, V), 0.0);

    // Unpack per-instance material data
    let height_01 = in.material.x;       // normalized column height [0..1]
    let color_seed = in.material.y;      // random hue offset per column
    let is_ground = in.material.z;       // 1.0 = ground, 0.0 = column
    let column_id = in.material.w;       // unique column hash

    // ────────── Ground Plane ──────────
    if (is_ground > 0.5) {
        let grid = fbm(in.world_pos.xz * 0.8 + scene.time * 0.05);
        let base = vec3(0.02, 0.025, 0.04);
        let highlight = vec3(0.04, 0.06, 0.10);
        let ground_color = mix(base, highlight, grid * 0.6);

        // Cursor glow on ground
        if (crystal.cursor_active > 0.5) {
            let d = distance(in.world_pos.xz, crystal.cursor_world);
            let glow = exp(-d * d * 0.15) * 0.08;
            return vec4(ground_color + vec3(0.1, 0.2, 0.4) * glow, 1.0);
        }

        return vec4(ground_color, 1.0);
    }

    // ────────── Lighting Setup ──────────

    let L = normalize(-lights[0].direction);
    let H = normalize(L + V);
    let NdotL = max(dot(N, L), 0.0);
    let NdotH = max(dot(N, H), 0.0);

    let is_top = N.y > 0.7;
    let is_side = !is_top;

    // ────────── Iridescent Base Color ──────────

    // Thin-film thickness varies per column and with view angle
    let thickness = 0.6 + color_seed * 0.8 + height_01 * 0.3;
    let iri = iridescence(NdotV, thickness);

    // Deep crystal base color
    let deep_crystal = vec3(0.06, 0.08, 0.18);
    let base_color = mix(deep_crystal, iri * 0.7, 0.4 + NdotV * 0.4);

    // ────────── Specular (Glass-like) ──────────

    // Primary sharp specular
    let spec_power = select(180.0, 320.0, is_top);
    let spec = pow(NdotH, spec_power);

    // Secondary broad specular with iridescent tint
    let spec2 = pow(NdotH, 24.0) * 0.25;
    let spec_color = mix(vec3(1.0), iri, 0.5);

    // ────────── Fresnel (Edge Glow) ──────────

    let fresnel = pow(1.0 - NdotV, 4.0);
    let rim_color = mix(iri * 0.6, vec3(0.5, 0.7, 1.0), 0.5);

    // ────────── Energy Veins (sides only) ──────────

    var vein_emit = 0.0;
    var vein_color = vec3(0.0);
    if (is_side) {
        let vein_intensity = energy_veins(in.world_pos, scene.time);
        vein_emit = vein_intensity * 0.6;

        // Veins pulse with a slow beat
        let pulse = 0.7 + 0.3 * sin(scene.time * 2.5 + column_id * 6.28);
        vein_emit *= pulse;

        // Vein color: electric blue-purple tinted by column's iridescence
        vein_color = mix(vec3(0.3, 0.5, 1.0), iri, 0.3) * vein_emit;
    }

    // ────────── Shadow ──────────

    // Normal-offset bias: push sample position sideways along the surface
    // normal (scaled by grazing angle), then subtract the light-direction
    // component so we shift in the shadow map without changing depth.
    // Prevents self-shadow artifacts on column sides.
    var bias_vec = N * (1.0 - NdotL) * 0.05;
    bias_vec -= L * dot(L, bias_vec);
    let biased_pos = in.world_pos + bias_vec;
    let biased_clip = lights[0].shadow_projection * vec4(biased_pos, 1.0);
    let shadow = sample_shadow_pcf(biased_clip, in.world_pos);
    let shadow_factor = mix(0.2, 1.0, shadow);

    // ────────── Diffuse ──────────

    let diffuse = base_color * (scene.ambient * 0.5 + NdotL * 0.6) * shadow_factor;

    // ────────── Combine ──────────

    var color = diffuse;

    // Add specular (attenuated by shadow)
    color += spec_color * spec * 0.5 * shadow;
    color += spec_color * spec2 * shadow;

    // Add fresnel rim glow (always visible, unaffected by shadow)
    color += rim_color * fresnel * 0.35;

    // Add energy veins (emissive, unaffected by shadow)
    color += vein_color;

    // ────────── Top Face Crystal Caustics ──────────

    if (is_top) {
        let caustic_uv = in.world_pos.xz * 3.0 + scene.time * 0.15;
        let c1 = fbm(caustic_uv);
        let c2 = fbm(caustic_uv * 1.4 + 3.7);
        let caustic = smoothstep(0.4, 0.6, c1) * smoothstep(0.3, 0.55, c2);
        color += iri * caustic * 0.15;
    }

    // ────────── Cursor Proximity Highlight ──────────

    if (crystal.cursor_active > 0.5) {
        let d = distance(in.world_pos.xz, crystal.cursor_world);
        let proximity = exp(-d * d * 0.3);

        // Brightening near cursor
        color += iri * proximity * 0.2;

        // Extra rim near cursor
        color += rim_color * fresnel * proximity * 0.3;
    }

    // ────────── Height-based atmospheric glow ──────────

    let height_glow = smoothstep(0.0, 1.0, height_01) * 0.06;
    let atmo_color = mix(vec3(0.2, 0.3, 0.8), iri, 0.4);
    color += atmo_color * height_glow;

    // ────────── Tone mapping (subtle S-curve) ──────────

    color = color / (color + vec3(1.0));

    return vec4(color, 1.0);
}
