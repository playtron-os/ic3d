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

#[test]
fn ease_out_elastic_endpoints() {
    assert!((ease_out_elastic(0.0)).abs() < 1e-6);
    assert!((ease_out_elastic(1.0) - 1.0).abs() < 1e-6);
}

#[test]
fn ease_out_elastic_overshoots() {
    // Early in the curve it should overshoot 1.0
    let vals: Vec<f32> = (1..10).map(|i| ease_out_elastic(i as f32 * 0.1)).collect();
    let max = vals.iter().copied().fold(0.0_f32, f32::max);
    assert!(max > 1.0, "expected overshoot, got {max}");
}

#[test]
fn ease_out_elastic_clamps_negative_input() {
    assert!((ease_out_elastic(-1.0)).abs() < 1e-6);
}

#[test]
fn ease_out_elastic_clamps_above_one() {
    assert!((ease_out_elastic(2.0) - 1.0).abs() < 1e-6);
}

#[test]
fn ease_out_elastic_near_end_close_to_one() {
    // At t=0.9, should be very close to 1.0
    let val = ease_out_elastic(0.9);
    assert!(
        (val - 1.0).abs() < 0.05,
        "expected near 1.0 at t=0.9, got {val}"
    );
}
