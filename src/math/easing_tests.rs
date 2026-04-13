use super::*;

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
