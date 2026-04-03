// iced3d: Standard Blinn-Phong fragment shader with multi-light support.
//
// Used as the default fragment shader when Scene3DProgram::fragment_shader()
// is not overridden. Supports directional, point, and spot lights with
// Poisson-disk PCF shadow mapping on the first light.
//
// Material from instance data: (r, g, b, shininess).
// shininess: 1.0 = very rough, 256.0 = mirror-like.

// Schlick Fresnel approximation — gives realistic edge highlights.
fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (vec3(1.0) - f0) * pow(saturate(1.0 - cos_theta), 5.0);
}

// Smooth inverse-square attenuation with range cutoff.
fn attenuation_ue4(dist: f32, range: f32) -> f32 {
    let ratio = saturate(dist / range);
    let ratio2 = ratio * ratio;
    let factor = saturate(1.0 - ratio2 * ratio2);
    return factor * factor / max(dist * dist, 0.0001);
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let N = normalize(in.world_normal);
    let V = normalize(scene.camera_position - in.world_pos);
    let NdotV = max(dot(N, V), 0.0);

    // Material from instance data: (r, g, b, shininess)
    let base_color = in.material.xyz;
    let shininess = max(in.material.w, 1.0);

    // Dielectric F0 (0.04 is standard for non-metallic surfaces)
    let f0 = vec3(0.04);

    // Ambient with hemisphere approximation (sky vs ground)
    let sky_color = base_color * scene.ambient;
    let ground_color = base_color * scene.ambient * 0.3;
    let ambient = mix(ground_color, sky_color, N.y * 0.5 + 0.5);

    var color = ambient;

    // Accumulate contribution from each light
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

            // Spot cone falloff
            if light.light_type == LIGHT_SPOT {
                let cos_angle = dot(-L, normalize(light.direction));
                let spot = smoothstep(light.outer_cone_cos, light.inner_cone_cos, cos_angle);
                attenuation *= spot;
            }
        }

        let NdotL = max(dot(N, L), 0.0);

        // Diffuse (energy-conserving: reduce diffuse where specular reflects)
        let H = normalize(L + V);
        let NdotH = max(dot(N, H), 0.0);
        let HdotV = max(dot(H, V), 0.0);

        let F = fresnel_schlick(HdotV, f0);
        let diffuse = base_color * (vec3(1.0) - F) * NdotL;

        // Specular (Blinn-Phong with normalization factor)
        // Normalization: (shininess + 8) / (8 * PI) per real-time rendering convention
        let norm_factor = (shininess + 8.0) / 25.1327;
        let specular = F * norm_factor * pow(NdotH, shininess) * NdotL;

        // Shadow (first light only)
        var shadow = 1.0;
        if i == 0u {
            // Normal offset bias — push lookup along
            // surface normal, then subtract the light-direction component.
            // This shifts the sample sideways in the shadow map without
            // changing its depth, preventing self-shadow artifacts.
            var bias_vec = N * (1.0 - NdotL) * 0.05;
            bias_vec -= L * dot(L, bias_vec);
            let biased_pos = in.world_pos + bias_vec;
            let biased_clip = lights[0].shadow_projection * vec4(biased_pos, 1.0);
            shadow = sample_shadow_pcf(biased_clip, in.world_pos);
        }

        let radiance = light.color * light.intensity * attenuation;
        color += (diffuse + specular) * radiance * shadow;
    }

    // Tone mapping (Reinhard) — prevents blown-out highlights
    color = color / (color + vec3(1.0));

    // Gamma correction (linear → sRGB)
    color = pow(color, vec3(1.0 / 2.2));

    return vec4(color, 1.0);
}
