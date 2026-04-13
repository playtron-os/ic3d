//! Easing functions for animations and transitions.

/// GLSL-style smoothstep: Hermite interpolation between 0 and 1.
///
/// Returns 0 when `x <= edge0`, 1 when `x >= edge1`, and smooth
/// interpolation between.
#[must_use]
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Overshoot bounce easing — scales up past 1.0 then settles back.
///
/// Peak ~1.15 at t ≈ 0.65, settles to 1.0 at t = 1.0.
/// Good for pop-in / spring animations.
#[must_use]
pub fn ease_out_back(t: f32) -> f32 {
    let c1 = 1.70158_f32;
    let c3 = c1 + 1.0;
    let x = t - 1.0;
    1.0 + c3 * x * x * x + c1 * x * x
}

/// Smooth Hermite interpolation on `[0, 1]` — equivalent to `smoothstep(0, 1, t)`.
///
/// Good default easing for animations that need smooth start and end.
#[must_use]
pub fn ease_smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Cubic ease-out: fast start, slow finish.
///
/// `1 - (1 - t)^3` — decelerates toward the end.
#[must_use]
pub fn ease_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

#[cfg(test)]
#[path = "easing_tests.rs"]
mod tests;
