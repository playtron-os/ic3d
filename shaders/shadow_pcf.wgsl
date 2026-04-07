// ic3d: 16-tap rotated Poisson disk PCF shadow sampling.
//
// Uses a screen-space rotation to break up aliasing patterns,
// similar to Unity's soft shadow implementation.

// Poisson disk distribution — 16 well-separated points in unit circle.
const POISSON_DISK: array<vec2<f32>, 16> = array(
    vec2(-0.94201624, -0.39906216),
    vec2( 0.94558609, -0.76890725),
    vec2(-0.09418410, -0.92938870),
    vec2( 0.34495938,  0.29387760),
    vec2(-0.91588581,  0.45771432),
    vec2(-0.81544232, -0.87912464),
    vec2(-0.38277543,  0.27676845),
    vec2( 0.97484398,  0.75648379),
    vec2( 0.44323325, -0.97511554),
    vec2( 0.53742981, -0.47373420),
    vec2(-0.26496911, -0.41893023),
    vec2( 0.79197514,  0.19090188),
    vec2(-0.24188840,  0.99706507),
    vec2(-0.81409955,  0.91437590),
    vec2( 0.19984126,  0.78641367),
    vec2( 0.14383161, -0.14100790),
);

fn sample_shadow_pcf(light_clip: vec4<f32>, world_pos: vec3<f32>) -> f32 {
    let ndc = light_clip.xyz / light_clip.w;
    let shadow_uv = vec2(ndc.x * 0.5 + 0.5, -ndc.y * 0.5 + 0.5);
    let depth = ndc.z;

    // Outside shadow map — fully lit
    if shadow_uv.x < 0.0 || shadow_uv.x > 1.0 || shadow_uv.y < 0.0 || shadow_uv.y > 1.0 {
        return 1.0;
    }

    let map_size = scene.shadow_map_size;
    let texel_size = 1.0 / map_size;

    // Spread radius in texels — wider = softer shadows
    let spread = 1.5 * texel_size;

    // Screen-space rotation to break banding (cheap hash from world position)
    let rot_angle = fract(sin(dot(world_pos.xz, vec2(12.9898, 78.233))) * 43758.5453) * 6.2831853;
    let cos_r = cos(rot_angle);
    let sin_r = sin(rot_angle);

    var shadow = 0.0;
    for (var i = 0u; i < 16u; i++) {
        let p = POISSON_DISK[i];
        // Rotate each sample point
        let rotated = vec2(
            p.x * cos_r - p.y * sin_r,
            p.x * sin_r + p.y * cos_r,
        );
        let offset_uv = shadow_uv + rotated * spread;
        shadow += textureSampleCompare(shadow_map, shadow_sampler, offset_uv, depth);
    }

    return shadow / 16.0;
}
