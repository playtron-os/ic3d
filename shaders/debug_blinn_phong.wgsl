// ic3d: Debug-capable Blinn-Phong fragment shader.
//
// Extends the standard Blinn-Phong with visualization modes
// controlled via a custom uniform at @group(1) @binding(0).
//
// Modes:
//   0 — Normal lit (Blinn-Phong + shadows)
//   1 — Normals (N * 0.5 + 0.5 → RGB)
//   2 — NdotL (grayscale, primary light only)
//   3 — Shadow factor (green = lit, red = shadow)
//   4 — Lit without shadows
//   5 — Flat base color (no lighting)

struct DebugUniforms {
    mode: f32,
    _p0: f32,
    _p1: f32,
    _p2: f32,
}

@group(1) @binding(0) var<uniform> debug: DebugUniforms;

// ── Helpers (same as standard Blinn-Phong) ──

fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (vec3(1.0) - f0) * pow(saturate(1.0 - cos_theta), 5.0);
}

fn attenuation_ue4(dist: f32, range: f32) -> f32 {
    let ratio = saturate(dist / range);
    let ratio2 = ratio * ratio;
    let factor = saturate(1.0 - ratio2 * ratio2);
    return factor * factor / max(dist * dist, 0.0001);
}

// ── Fragment entry ──

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let N = normalize(in.world_normal);
    let mode = u32(debug.mode);

    // Mode 1: Normals visualization
    if mode == 1u {
        return vec4(N * 0.5 + 0.5, 1.0);
    }

    let V = normalize(scene.camera_position - in.world_pos);
    let base_color = in.material.xyz;

    // Mode 5: Flat base color
    if mode == 5u {
        return vec4(pow(base_color, vec3(1.0 / 2.2)), 1.0);
    }

    // Primary light direction (for modes 2-3)
    let L0 = normalize(-lights[0].direction);
    let NdotL0 = max(dot(N, L0), 0.0);

    // Mode 2: NdotL grayscale
    if mode == 2u {
        return vec4(vec3(NdotL0), 1.0);
    }

    // Mode 3: Shadow factor (red = shadow, green = lit)
    if mode == 3u {
        var bias_vec = N * (1.0 - NdotL0) * 0.05;
        bias_vec -= L0 * dot(L0, bias_vec);
        let biased_pos = in.world_pos + bias_vec;
        let biased_clip = lights[0].shadow_projection * vec4(biased_pos, 1.0);
        let shadow = sample_shadow_pcf(biased_clip, in.world_pos);
        return vec4(1.0 - shadow, shadow, 0.0, 1.0);
    }

    // Modes 0 and 4: Full Blinn-Phong (mode 4 skips shadow)
    let NdotV = max(dot(N, V), 0.0);
    let shininess = max(in.material.w, 1.0);
    let f0 = vec3(0.04);

    let sky_color = base_color * scene.ambient;
    let ground_color = base_color * scene.ambient * 0.3;
    let ambient = mix(ground_color, sky_color, N.y * 0.5 + 0.5);

    var color = ambient;

    for (var i = 0u; i < scene.light_count; i++) {
        let light = lights[i];

        var L: vec3<f32>;
        var attenuation = 1.0;

        if light.light_type == LIGHT_DIRECTIONAL {
            L = normalize(-light.direction);
        } else {
            let to_light = light.position - in.world_pos;
            let dist = length(to_light);
            L = to_light / dist;

            if light.range > 0.0 {
                attenuation = attenuation_ue4(dist, light.range);
            }

            if light.light_type == LIGHT_SPOT {
                let cos_angle = dot(-L, normalize(light.direction));
                let spot = smoothstep(light.outer_cone_cos, light.inner_cone_cos, cos_angle);
                attenuation *= spot;
            }
        }

        let NdotL = max(dot(N, L), 0.0);
        let H = normalize(L + V);
        let NdotH = max(dot(N, H), 0.0);
        let HdotV = max(dot(H, V), 0.0);

        let F = fresnel_schlick(HdotV, f0);
        let diffuse = base_color * (vec3(1.0) - F) * NdotL;
        let norm_factor = (shininess + 8.0) / 25.1327;
        let specular = F * norm_factor * pow(NdotH, shininess) * NdotL;

        // Shadow (first light only, skip in mode 4)
        var shadow = 1.0;
        if i == 0u && mode != 4u {
            var bias_vec = N * (1.0 - NdotL) * 0.05;
            bias_vec -= L * dot(L, bias_vec);
            let biased_pos = in.world_pos + bias_vec;
            let biased_clip = lights[0].shadow_projection * vec4(biased_pos, 1.0);
            shadow = sample_shadow_pcf(biased_clip, in.world_pos);
        }

        let radiance = light.color * light.intensity * attenuation;
        color += (diffuse + specular) * radiance * shadow;
    }

    color = color / (color + vec3(1.0));
    color = pow(color, vec3(1.0 / 2.2));
    return vec4(color, 1.0);
}
