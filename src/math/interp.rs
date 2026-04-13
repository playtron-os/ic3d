//! Interpolation utilities: lerp, inverse lerp, remap.

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

#[cfg(test)]
#[path = "interp_tests.rs"]
mod tests;
