//! Common math utilities for 3D applications.

/// Linear interpolation between `a` and `b` by factor `t`.
///
/// Returns `a` when `t = 0`, `b` when `t = 1`.
#[must_use]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Inverse of [`lerp`]: returns the `t` that would produce `x` in `[a, b]`.
///
/// Returns 0 when `x == a`, 1 when `x == b`. Not clamped — can return
/// values outside 0..1 when `x` is outside the range.
#[must_use]
pub fn inverse_lerp(a: f32, b: f32, x: f32) -> f32 {
    (x - a) / (b - a)
}

/// Remap `x` from range `[from_min, from_max]` to `[to_min, to_max]`.
#[must_use]
pub fn remap(x: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    lerp(to_min, to_max, inverse_lerp(from_min, from_max, x))
}

/// 2D Euclidean distance between `(x1, y1)` and `(x2, y2)`.
#[must_use]
pub fn distance_2d(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    ((x1 - x2).powi(2) + (y1 - y2).powi(2)).sqrt()
}

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

/// Deterministic spatial hash — stable per-cell randomness from 2D coords + seed.
///
/// Returns a value in `[0, 1)`.
#[must_use]
pub fn hash_f32(x: f32, y: f32, seed: f32) -> f32 {
    let p = x * 127.1 + y * 311.7 + seed * 74.7;
    (p.sin() * 43_758.547).fract().abs()
}

/// Signed spatial hash — returns a value in `(-1, 1)`.
///
/// Equivalent to `hash_f32(x, y, seed) * 2.0 - 1.0`.
#[must_use]
pub fn hash_f32_signed(x: f32, y: f32, seed: f32) -> f32 {
    hash_f32(x, y, seed) * 2.0 - 1.0
}

/// Spatial hash mapped to a range — returns a value in `[min, max)`.
///
/// Equivalent to `lerp(min, max, hash_f32(x, y, seed))`.
#[must_use]
pub fn hash_f32_range(x: f32, y: f32, seed: f32, min: f32, max: f32) -> f32 {
    min + hash_f32(x, y, seed) * (max - min)
}

#[cfg(test)]
#[path = "math_tests.rs"]
mod tests;
