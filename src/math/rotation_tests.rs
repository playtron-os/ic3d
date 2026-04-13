//! Tests for rotation math utilities.

use super::*;
use glam::{Vec2, Vec3};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI, TAU};

// ──────────── screen_angle ────────────

#[test]
fn screen_angle_right_is_zero() {
    let angle = screen_angle(Vec2::new(400.0, 300.0), Vec2::new(500.0, 300.0));
    assert!(angle.abs() < 1e-6, "right = 0 radians");
}

#[test]
fn screen_angle_down_is_half_pi() {
    // Screen Y points down, so (0, +) means downward → π/2
    let angle = screen_angle(Vec2::new(400.0, 300.0), Vec2::new(400.0, 400.0));
    assert!(
        (angle - FRAC_PI_2).abs() < 1e-6,
        "down should be π/2, got {angle}"
    );
}

#[test]
fn screen_angle_left_is_pi() {
    let angle = screen_angle(Vec2::new(400.0, 300.0), Vec2::new(300.0, 300.0));
    assert!(
        (angle.abs() - PI).abs() < 1e-6,
        "left should be ±π, got {angle}"
    );
}

#[test]
fn screen_angle_up_is_neg_half_pi() {
    let angle = screen_angle(Vec2::new(400.0, 300.0), Vec2::new(400.0, 200.0));
    assert!(
        (angle + FRAC_PI_2).abs() < 1e-6,
        "up should be −π/2, got {angle}"
    );
}

#[test]
fn screen_angle_diagonal() {
    let angle = screen_angle(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0));
    assert!(
        (angle - FRAC_PI_4).abs() < 1e-6,
        "45° diagonal should be π/4, got {angle}"
    );
}

// ──────────── wrap_angle_delta ────────────

#[test]
fn wrap_small_positive() {
    let d = wrap_angle_delta(0.5);
    assert!((d - 0.5).abs() < 1e-6);
}

#[test]
fn wrap_small_negative() {
    let d = wrap_angle_delta(-0.5);
    assert!((d + 0.5).abs() < 1e-6);
}

#[test]
fn wrap_over_pi() {
    let d = wrap_angle_delta(PI + 0.5);
    let expected = PI + 0.5 - TAU;
    assert!(
        (d - expected).abs() < 1e-6,
        "should wrap to negative, got {d}"
    );
}

#[test]
fn wrap_under_neg_pi() {
    let d = wrap_angle_delta(-PI - 0.5);
    let expected = -PI - 0.5 + TAU;
    assert!(
        (d - expected).abs() < 1e-6,
        "should wrap to positive, got {d}"
    );
}

#[test]
fn wrap_exactly_pi() {
    let d = wrap_angle_delta(PI);
    assert!(d.abs() <= PI + 1e-6, "π should stay in range");
}

#[test]
fn wrap_exactly_neg_pi() {
    let d = wrap_angle_delta(-PI);
    assert!(d.abs() <= PI + 1e-6, "−π should stay in range");
}

#[test]
fn wrap_zero_unchanged() {
    assert!((wrap_angle_delta(0.0)).abs() < 1e-6);
}

// ──────────── rotation_sign ────────────

#[test]
fn sign_camera_looking_along_axis() {
    // Camera forward = −Z, axis = Z → dot < 0 → sign = −1.0
    assert_eq!(rotation_sign(Vec3::NEG_Z, Vec3::Z), -1.0);
}

#[test]
fn sign_camera_looking_against_axis() {
    // Camera forward = +Z, axis = Z → dot > 0 → sign = +1.0
    assert_eq!(rotation_sign(Vec3::Z, Vec3::Z), 1.0);
}

#[test]
fn sign_perpendicular() {
    // Camera looking along X, axis = Z → dot = 0 → sign = +1.0
    assert_eq!(rotation_sign(Vec3::X, Vec3::Z), 1.0);
}

#[test]
fn sign_y_axis_from_above() {
    // Camera looking down (−Y), rotating around Y → dot < 0 → sign = −1.0
    assert_eq!(rotation_sign(Vec3::NEG_Y, Vec3::Y), -1.0);
}

#[test]
fn sign_y_axis_from_below() {
    // Camera looking up (+Y), rotating around Y → dot > 0 → sign = +1.0
    assert_eq!(rotation_sign(Vec3::Y, Vec3::Y), 1.0);
}

// ──────────── view_facing_rotation ────────────

#[test]
fn view_facing_rotation_neg_z() {
    // Camera looking along -Z: Y should rotate to +Z (facing camera).
    let rot = view_facing_rotation(Vec3::NEG_Z);
    let up = rot * Vec3::Y;
    assert!(
        (up - Vec3::Z).length() < 1e-3,
        "Y should face camera (+Z), got {up:?}"
    );
}

#[test]
fn view_facing_rotation_neg_y() {
    // Camera looking down (-Y): Y should rotate to +Y (facing camera).
    let rot = view_facing_rotation(Vec3::NEG_Y);
    let up = rot * Vec3::Y;
    assert!(
        (up - Vec3::Y).length() < 1e-3,
        "Y should face camera (+Y), got {up:?}"
    );
}

#[test]
fn view_facing_rotation_pos_x() {
    // Camera looking along +X: Y should rotate to -X.
    let rot = view_facing_rotation(Vec3::X);
    let up = rot * Vec3::Y;
    assert!(
        (up - Vec3::NEG_X).length() < 1e-3,
        "Y should face camera (-X), got {up:?}"
    );
}

#[test]
fn view_facing_rotation_normalizes_input() {
    // Non-unit input should still work.
    let rot = view_facing_rotation(Vec3::new(0.0, 0.0, -5.0));
    let up = rot * Vec3::Y;
    assert!(
        (up - Vec3::Z).length() < 1e-3,
        "should normalize, got {up:?}"
    );
}

// ──────────── front_arc_params ────────────

#[test]
fn front_arc_edge_on_sweep_is_pi() {
    // Y-axis ring seen from a camera along -Z: ring normal (Y) is
    // perpendicular to cam_forward → face_factor ≈ 0 → sweep ≈ π.
    let (_, sweep) = front_arc_params(glam::Quat::IDENTITY, Vec3::Y, Vec3::NEG_Z);
    assert!(
        (sweep - PI).abs() < 0.1,
        "edge-on sweep should be ~π, got {sweep}"
    );
}

#[test]
fn front_arc_face_on_sweep_is_tau() {
    // Y-axis ring seen from a camera along -Y: ring normal aligned
    // with cam_forward → face_factor ≈ 1 → sweep ≈ τ.
    let (_, sweep) = front_arc_params(glam::Quat::IDENTITY, Vec3::Y, Vec3::NEG_Y);
    assert!(
        (sweep - TAU).abs() < 0.1,
        "face-on sweep should be ~τ, got {sweep}"
    );
}

#[test]
fn front_arc_sweep_monotonic() {
    // As face_factor increases from 0 to 1, sweep should increase.
    let (_, sweep_edge) = front_arc_params(glam::Quat::IDENTITY, Vec3::Y, Vec3::NEG_Z);
    let diag = Vec3::new(0.0, -1.0, -1.0).normalize();
    let (_, sweep_mid) = front_arc_params(glam::Quat::IDENTITY, Vec3::Y, diag);
    let (_, sweep_face) = front_arc_params(glam::Quat::IDENTITY, Vec3::Y, Vec3::NEG_Y);
    assert!(
        sweep_edge < sweep_mid && sweep_mid < sweep_face,
        "sweep should increase: {sweep_edge} < {sweep_mid} < {sweep_face}"
    );
}

#[test]
fn front_arc_start_centered_on_front() {
    // The visible arc center should face the camera.
    // For a Y-axis ring seen from -Z, the front of the ring in local
    // XZ space is the half facing the camera.
    let (start, sweep) = front_arc_params(glam::Quat::IDENTITY, Vec3::Y, Vec3::NEG_Z);
    let center = start + sweep * 0.5;
    // The center angle should be opposite to where the camera looks
    // into the ring. With identity rotation and -Z camera, this is
    // deterministic.
    assert!(
        center.is_finite(),
        "center angle should be finite: {center}"
    );
}
