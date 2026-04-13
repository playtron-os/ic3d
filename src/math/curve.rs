//! Curve flattening and 2D distance utilities.

/// 2D Euclidean distance between `(x1, y1)` and `(x2, y2)`.
#[must_use]
pub fn distance_2d(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    ((x1 - x2).powi(2) + (y1 - y2).powi(2)).sqrt()
}

/// Flatten a cubic Bezier curve into line segments.
///
/// Evaluates the curve defined by control points `p0..p3` at
/// `segments` uniformly-spaced parameter values and appends the
/// resulting 2D points to `out`. The start point (`t = 0`) is NOT
/// emitted — callers typically already have it as the current pen
/// position.
///
/// ```rust,ignore
/// let mut pts = vec![[0.0, 0.0]]; // start point
/// flatten_cubic([0.0, 0.0], [0.5, 1.0], [1.0, 1.0], [1.5, 0.0], 8, &mut pts);
/// assert_eq!(pts.len(), 9); // start + 8 segments
/// ```
#[allow(clippy::cast_precision_loss)]
pub fn flatten_cubic(
    p0: [f32; 2],
    p1: [f32; 2],
    p2: [f32; 2],
    p3: [f32; 2],
    segments: usize,
    out: &mut Vec<[f32; 2]>,
) {
    for seg in 1..=segments {
        let t = seg as f32 / segments as f32;
        let u = 1.0 - t;
        let x = u * u * u * p0[0]
            + 3.0 * u * u * t * p1[0]
            + 3.0 * u * t * t * p2[0]
            + t * t * t * p3[0];
        let y = u * u * u * p0[1]
            + 3.0 * u * u * t * p1[1]
            + 3.0 * u * t * t * p2[1]
            + t * t * t * p3[1];
        out.push([x, y]);
    }
}

#[cfg(test)]
#[path = "curve_tests.rs"]
mod tests;
