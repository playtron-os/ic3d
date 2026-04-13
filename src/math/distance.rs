//! XZ-plane distance utilities (Y ignored).

use glam::Vec3;

/// Squared distance between two points projected onto the XZ plane (Y ignored).
///
/// Useful for cheap proximity checks without a square root — compare the
/// result against `radius * radius` instead of computing `distance_xz`.
///
/// ```rust
/// # use ic3d::math::distance_xz_squared;
/// # use glam::Vec3;
/// let a = Vec3::new(1.0, 5.0, 0.0);
/// let b = Vec3::new(4.0, 9.0, 0.0);
/// assert!((distance_xz_squared(a, b) - 9.0).abs() < 1e-6);
/// ```
#[must_use]
pub fn distance_xz_squared(a: Vec3, b: Vec3) -> f32 {
    let dx = a.x - b.x;
    let dz = a.z - b.z;
    dx * dx + dz * dz
}

/// Distance between two points projected onto the XZ plane (Y ignored).
///
/// ```rust
/// # use ic3d::math::distance_xz;
/// # use glam::Vec3;
/// let a = Vec3::new(1.0, 100.0, 0.0);
/// let b = Vec3::new(4.0, 0.0, 4.0);
/// assert!((distance_xz(a, b) - 5.0).abs() < 1e-6);
/// ```
#[must_use]
pub fn distance_xz(a: Vec3, b: Vec3) -> f32 {
    distance_xz_squared(a, b).sqrt()
}

#[cfg(test)]
#[path = "distance_tests.rs"]
mod tests;
