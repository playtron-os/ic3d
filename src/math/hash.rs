//! Deterministic spatial hash functions for procedural generation.

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
#[path = "hash_tests.rs"]
mod tests;
