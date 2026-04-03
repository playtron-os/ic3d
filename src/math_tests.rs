use super::*;

#[test]
fn lerp_endpoints() {
    assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
    assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);
}

#[test]
fn lerp_midpoint() {
    assert!((lerp(0.0, 10.0, 0.5) - 5.0).abs() < f32::EPSILON);
}

#[test]
fn lerp_extrapolates() {
    assert!((lerp(0.0, 10.0, 2.0) - 20.0).abs() < f32::EPSILON);
}

#[test]
fn inverse_lerp_endpoints() {
    assert_eq!(inverse_lerp(0.0, 10.0, 0.0), 0.0);
    assert_eq!(inverse_lerp(0.0, 10.0, 10.0), 1.0);
}

#[test]
fn inverse_lerp_midpoint() {
    assert!((inverse_lerp(0.0, 10.0, 5.0) - 0.5).abs() < f32::EPSILON);
}

#[test]
fn remap_basic() {
    let result = remap(5.0, 0.0, 10.0, 100.0, 200.0);
    assert!((result - 150.0).abs() < f32::EPSILON);
}

#[test]
fn remap_identity() {
    let result = remap(3.0, 0.0, 10.0, 0.0, 10.0);
    assert!((result - 3.0).abs() < f32::EPSILON);
}

#[test]
fn distance_2d_zero() {
    assert_eq!(distance_2d(0.0, 0.0, 0.0, 0.0), 0.0);
}

#[test]
fn distance_2d_unit() {
    assert!((distance_2d(0.0, 0.0, 3.0, 4.0) - 5.0).abs() < f32::EPSILON);
}

#[test]
fn distance_2d_symmetric() {
    let d1 = distance_2d(1.0, 2.0, 4.0, 6.0);
    let d2 = distance_2d(4.0, 6.0, 1.0, 2.0);
    assert!((d1 - d2).abs() < f32::EPSILON);
}

#[test]
fn smoothstep_below_edge0() {
    assert_eq!(smoothstep(0.0, 1.0, -0.5), 0.0);
}

#[test]
fn smoothstep_above_edge1() {
    assert_eq!(smoothstep(0.0, 1.0, 1.5), 1.0);
}

#[test]
fn smoothstep_midpoint() {
    assert!((smoothstep(0.0, 1.0, 0.5) - 0.5).abs() < f32::EPSILON);
}

#[test]
fn smoothstep_at_edges() {
    assert_eq!(smoothstep(0.0, 1.0, 0.0), 0.0);
    assert_eq!(smoothstep(0.0, 1.0, 1.0), 1.0);
}

#[test]
fn ease_out_back_endpoints() {
    assert!((ease_out_back(0.0) - 0.0).abs() < 1e-6);
    assert!((ease_out_back(1.0) - 1.0).abs() < 1e-6);
}

#[test]
fn ease_out_back_overshoots() {
    // Peak should exceed 1.0 somewhere in the middle
    let peak = ease_out_back(0.65);
    assert!(peak > 1.0, "expected overshoot, got {peak}");
}

#[test]
fn ease_smooth_endpoints() {
    assert!((ease_smooth(0.0)).abs() < 1e-6);
    assert!((ease_smooth(1.0) - 1.0).abs() < 1e-6);
}

#[test]
fn ease_smooth_midpoint() {
    assert!((ease_smooth(0.5) - 0.5).abs() < 1e-6);
}

#[test]
fn ease_smooth_clamps() {
    assert!((ease_smooth(-1.0)).abs() < 1e-6);
    assert!((ease_smooth(2.0) - 1.0).abs() < 1e-6);
}

#[test]
fn ease_out_cubic_endpoints() {
    assert!((ease_out_cubic(0.0)).abs() < 1e-6);
    assert!((ease_out_cubic(1.0) - 1.0).abs() < 1e-6);
}

#[test]
fn ease_out_cubic_fast_start() {
    // Should be past halfway at t=0.5 (cubic ease-out front-loads motion)
    let mid = ease_out_cubic(0.5);
    assert!(mid > 0.5, "expected > 0.5, got {mid}");
}

#[test]
fn ease_out_cubic_clamps() {
    assert!((ease_out_cubic(-1.0)).abs() < 1e-6);
    assert!((ease_out_cubic(2.0) - 1.0).abs() < 1e-6);
}

#[test]
fn hash_f32_in_range() {
    for i in 0..20 {
        let h = hash_f32(i as f32, (i * 7) as f32, 42.0);
        assert!((0.0..1.0).contains(&h), "hash={h} out of [0, 1)");
    }
}

#[test]
fn hash_f32_deterministic() {
    let a = hash_f32(1.0, 2.0, 3.0);
    let b = hash_f32(1.0, 2.0, 3.0);
    assert_eq!(a, b);
}

#[test]
fn hash_f32_signed_in_range() {
    for i in 0..20 {
        let h = hash_f32_signed(i as f32, (i * 3) as f32, 0.0);
        assert!((-1.0..1.0).contains(&h), "hash_signed={h} out of (-1, 1)");
    }
}

#[test]
fn hash_f32_range_in_bounds() {
    for i in 0..20 {
        let h = hash_f32_range(i as f32, (i * 5) as f32, 1.0, 10.0, 20.0);
        assert!((10.0..20.0).contains(&h), "hash_range={h} out of [10, 20)");
    }
}
