//! Tests for the gizmo module.

use super::*;
use crate::camera::CameraInfo;
use crate::math::point_to_segment_distance;
use crate::scene::context::{SceneContext, SceneHandle};
use crate::scene::object::SceneObjectId;
use glam::{Mat4, Vec2, Vec3};
use std::collections::HashMap;

/// Helper: create a CameraInfo for a perspective camera at `cam_pos` looking along `fwd`.
fn cam_info(cam_pos: Vec3, fwd: Vec3, fov: f32) -> CameraInfo {
    let target = cam_pos + fwd;
    let view = Mat4::look_at_rh(cam_pos, target, Vec3::Y);
    let proj = Mat4::perspective_rh(fov, 1.0, 0.1, 100.0);
    CameraInfo {
        position: cam_pos,
        forward: fwd.normalize(),
        fov_y: Some(fov),
        view_projection: proj * view,
    }
}

/// Helper: create a SceneContext with optional object transforms.
fn scene_ctx(
    cam_pos: Vec3,
    fwd: Vec3,
    fov: f32,
    viewport: Vec2,
    objects: HashMap<SceneObjectId, Mat4>,
) -> SceneContext {
    SceneContext {
        camera: cam_info(cam_pos, fwd, fov),
        viewport_size: viewport,
        objects,
    }
}

/// Helper: create a SceneHandle populated with camera data.
fn populated_handle(
    cam_pos: Vec3,
    fwd: Vec3,
    fov: f32,
    viewport: Vec2,
    objects: HashMap<SceneObjectId, Mat4>,
) -> SceneHandle {
    let handle = SceneHandle::new();
    handle.update_context(scene_ctx(cam_pos, fwd, fov, viewport, objects));
    handle
}

#[test]
fn gizmo_default_position() {
    let g = Gizmo::default();
    assert_eq!(g.gizmo_position(), Vec3::ZERO);
    assert_eq!(g.mode(), GizmoMode::Translate);
    assert!(!g.is_dragging());
    assert!(g.hovered_axis().is_none());
}

#[test]
fn gizmo_builder() {
    let g = Gizmo::new(GizmoMode::Translate)
        .position(Vec3::new(1.0, 2.0, 3.0))
        .scale(2.5)
        .gizmo_size(120.0);
    assert_eq!(g.gizmo_position(), Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(g.scale, 2.5);
    assert_eq!(g.gizmo_size, 120.0);
}

#[test]
fn gizmo_interactive_default() {
    let g = Gizmo::default();
    assert!(g.is_interactive(), "gizmo should be interactive by default");
}

#[test]
fn gizmo_interactive_builder() {
    let g = Gizmo::new(GizmoMode::Translate).interactive(false);
    assert!(!g.is_interactive());
}

#[test]
fn gizmo_set_interactive() {
    let mut g = Gizmo::default();
    g.set_interactive(false);
    assert!(!g.is_interactive());
    g.set_interactive(true);
    assert!(g.is_interactive());
}

#[test]
fn gizmo_default_gizmo_size() {
    let g = Gizmo::default();
    assert_eq!(g.gizmo_size, DEFAULT_GIZMO_SIZE);
}

#[test]
fn gizmo_set_position() {
    let mut g = Gizmo::default();
    g.set_position(Vec3::new(5.0, 0.0, 0.0));
    assert_eq!(g.gizmo_position(), Vec3::new(5.0, 0.0, 0.0));
}

#[test]
fn gizmo_set_gizmo_size() {
    let mut g = Gizmo::default();
    g.set_gizmo_size(120.0);
    assert_eq!(g.gizmo_size, 120.0);
}

#[test]
fn gizmo_draw_groups_count() {
    let g = Gizmo::default();
    let groups = g.draw_groups();
    assert_eq!(groups.len(), 3, "should produce one draw group per axis");
}

#[test]
fn gizmo_draw_groups_have_instances() {
    let g = Gizmo::default();
    for group in g.draw_groups() {
        assert_eq!(
            group.instances.len(),
            1,
            "each axis should have one instance"
        );
    }
}

#[test]
fn axis_directions() {
    assert_eq!(GizmoAxis::X.direction(), Vec3::X);
    assert_eq!(GizmoAxis::Y.direction(), Vec3::Y);
    assert_eq!(GizmoAxis::Z.direction(), Vec3::Z);
}

#[test]
fn world_to_screen_identity_vp() {
    let vp = glam::Mat4::IDENTITY;
    let viewport = Vec2::new(800.0, 600.0);
    let result = world_to_screen(Vec3::ZERO, vp, viewport);
    assert!(result.is_some());
    let screen = result.unwrap();
    assert!((screen.x - 400.0).abs() < 1.0);
    assert!((screen.y - 300.0).abs() < 1.0);
}

#[test]
fn world_to_screen_behind_camera() {
    let view = glam::Mat4::look_at_rh(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO, Vec3::Y);
    let proj = glam::Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, 1.0, 0.1, 100.0);
    let vp = proj * view;
    let result = world_to_screen(Vec3::new(0.0, 0.0, 100.0), vp, Vec2::new(800.0, 600.0));
    assert!(result.is_none(), "point behind camera should return None");
}

#[test]
fn point_to_segment_distance_on_segment() {
    let p = Vec2::new(0.5, 0.0);
    let a = Vec2::new(0.0, 0.0);
    let b = Vec2::new(1.0, 0.0);
    let dist = point_to_segment_distance(p, a, b);
    assert!(
        dist.abs() < 1e-6,
        "point on segment should have zero distance"
    );
}

#[test]
fn point_to_segment_distance_perpendicular() {
    let p = Vec2::new(0.5, 1.0);
    let a = Vec2::new(0.0, 0.0);
    let b = Vec2::new(1.0, 0.0);
    let dist = point_to_segment_distance(p, a, b);
    assert!(
        (dist - 1.0).abs() < 1e-6,
        "perpendicular distance should be 1.0"
    );
}

#[test]
fn point_to_segment_distance_past_end() {
    let p = Vec2::new(2.0, 0.0);
    let a = Vec2::new(0.0, 0.0);
    let b = Vec2::new(1.0, 0.0);
    let dist = point_to_segment_distance(p, a, b);
    assert!(
        (dist - 1.0).abs() < 1e-6,
        "distance past endpoint should be 1.0"
    );
}

#[test]
fn compute_scale_increases_with_distance() {
    let near = Gizmo::new(GizmoMode::Translate).position(Vec3::new(0.0, 0.0, 5.0));
    let far = Gizmo::new(GizmoMode::Translate).position(Vec3::new(0.0, 0.0, 20.0));
    let fov = std::f32::consts::FRAC_PI_4;
    let ci = cam_info(Vec3::ZERO, Vec3::Z, fov);

    let scale_near = near.compute_scale(&ci, 600.0);
    let scale_far = far.compute_scale(&ci, 600.0);
    assert!(
        scale_far > scale_near,
        "gizmo farther from camera should have a larger world-space scale"
    );
}

#[test]
fn compute_scale_viewport_invariant() {
    let g = Gizmo::new(GizmoMode::Translate).position(Vec3::new(0.0, 0.0, 10.0));
    let fov = std::f32::consts::FRAC_PI_4;
    let ci = cam_info(Vec3::ZERO, Vec3::Z, fov);

    let s600 = g.compute_scale(&ci, 600.0);
    let s1200 = g.compute_scale(&ci, 1200.0);

    let world_span = 2.0 * 10.0 * (fov * 0.5).tan();
    let px600 = s600 / world_span * 600.0;
    let px1200 = s1200 / world_span * 1200.0;
    assert!(
        (px600 - px1200).abs() < 0.01,
        "screen size should be identical (px600={px600}, px1200={px1200})"
    );
}

#[test]
fn compute_scale_matches_expected_formula() {
    let g = Gizmo::new(GizmoMode::Translate).position(Vec3::new(0.0, 0.0, 10.0));
    let fov = std::f32::consts::FRAC_PI_4;
    let vh = 600.0;
    let ci = cam_info(Vec3::ZERO, Vec3::Z, fov);

    let scale = g.compute_scale(&ci, vh);
    let world_span = 2.0 * 10.0 * (fov * 0.5).tan();
    let screen_px = scale / world_span * vh;
    let expected_px = 1.75 * 80.0; // 140

    assert!(
        (screen_px - expected_px).abs() < 0.1,
        "screen px should be 1.75×gizmo_size=140 (got {screen_px})"
    );
}

#[test]
fn compute_scale_gizmo_size_proportional() {
    let g80 = Gizmo::new(GizmoMode::Translate)
        .position(Vec3::new(0.0, 0.0, 10.0))
        .gizmo_size(80.0);
    let g160 = Gizmo::new(GizmoMode::Translate)
        .position(Vec3::new(0.0, 0.0, 10.0))
        .gizmo_size(160.0);
    let fov = std::f32::consts::FRAC_PI_4;
    let ci = cam_info(Vec3::ZERO, Vec3::Z, fov);

    let s80 = g80.compute_scale(&ci, 1200.0);
    let s160 = g160.compute_scale(&ci, 1200.0);
    assert!(
        (s160 / s80 - 2.0).abs() < 1e-4,
        "doubling gizmo_size should double the scale"
    );
}

#[test]
fn compute_scale_no_shrink_on_large_viewport() {
    let g = Gizmo::new(GizmoMode::Translate).position(Vec3::new(0.0, 0.0, 10.0));
    let fov = std::f32::consts::FRAC_PI_4;
    let ci = cam_info(Vec3::ZERO, Vec3::Z, fov);

    let world_span = 2.0 * 10.0 * (fov * 0.5).tan();
    let s600 = g.compute_scale(&ci, 600.0);
    let s800 = g.compute_scale(&ci, 800.0);
    let px600 = s600 / world_span * 600.0;
    let px800 = s800 / world_span * 800.0;
    assert!(
        (px600 - px800).abs() < 0.1,
        "gizmo should be same px on large viewports (px600={px600}, px800={px800})"
    );
}

#[test]
fn compute_scale_shrinks_on_small_viewport() {
    let g = Gizmo::new(GizmoMode::Translate).position(Vec3::new(0.0, 0.0, 10.0));
    let fov = std::f32::consts::FRAC_PI_4;
    let ci = cam_info(Vec3::ZERO, Vec3::Z, fov);

    let world_span = 2.0 * 10.0 * (fov * 0.5).tan();
    let s200 = g.compute_scale(&ci, 200.0);
    let px200 = s200 / world_span * 200.0;
    let expected = 0.35 * 200.0;
    assert!(
        (px200 - expected).abs() < 0.1,
        "gizmo should clamp to 35% of viewport (got {px200}, expected {expected})"
    );
}

#[test]
fn compute_scale_fallback_on_zero_viewport() {
    let g = Gizmo::new(GizmoMode::Translate).scale(1.5);
    let ci = cam_info(Vec3::ZERO, Vec3::Z, std::f32::consts::FRAC_PI_4);
    let result = g.compute_scale(&ci, 0.0);
    assert_eq!(result, 1.5, "should return current scale for zero viewport");
}

#[test]
fn compute_scale_fallback_on_orthographic() {
    let g = Gizmo::new(GizmoMode::Translate).scale(2.0);
    let ci = CameraInfo {
        position: Vec3::ZERO,
        forward: Vec3::NEG_Z,
        fov_y: None,
        view_projection: Mat4::IDENTITY,
    };
    let result = g.compute_scale(&ci, 600.0);
    assert_eq!(result, 2.0, "should return current scale for ortho camera");
}

#[test]
fn overlay_draw_produces_groups() {
    let g = Gizmo::new(GizmoMode::Translate).position(Vec3::new(0.0, 0.0, 5.0));
    let ctx = scene_ctx(
        Vec3::ZERO,
        Vec3::Z,
        std::f32::consts::FRAC_PI_4,
        Vec2::new(800.0, 600.0),
        HashMap::new(),
    );
    let groups = g.draw(&ctx);
    assert_eq!(
        groups.len(),
        3,
        "Overlay::draw should produce 3 axis groups"
    );
}

// ──────────── attach_to ────────────

#[test]
fn gizmo_attach_to_builder() {
    let id = SceneObjectId(42);
    let g = Gizmo::new(GizmoMode::Translate).attach_to(id);
    assert_eq!(g.attached_to(), Some(id));
}

#[test]
fn gizmo_detach() {
    let mut g = Gizmo::new(GizmoMode::Translate).attach_to(SceneObjectId(1));
    g.detach();
    assert!(g.attached_to().is_none());
}

#[test]
fn gizmo_set_attached_to() {
    let mut g = Gizmo::default();
    g.set_attached_to(Some(SceneObjectId(5)));
    assert_eq!(g.attached_to(), Some(SceneObjectId(5)));
}

#[test]
fn overlay_draw_attached_uses_object_position() {
    let target_id = SceneObjectId(1);
    let object_pos = Vec3::new(3.0, 4.0, 5.0);
    let mut objects = HashMap::new();
    objects.insert(target_id, Mat4::from_translation(object_pos));

    let g = Gizmo::new(GizmoMode::Translate).attach_to(target_id);
    let ctx = scene_ctx(
        Vec3::ZERO,
        Vec3::Z,
        std::f32::consts::FRAC_PI_4,
        Vec2::new(800.0, 600.0),
        objects,
    );
    let groups = g.draw(&ctx);
    assert_eq!(groups.len(), 3);

    // Verify the instance transform uses the object's position, not origin.
    let model = groups[0].instances[0].model;
    let tx = model[3][0]; // translation X from the model matrix
                          // The translation comes from position * scale, so just verify it's non-zero
                          // (the gizmo was at origin, object at 3.0)
    assert!(
        tx.abs() > 0.1,
        "gizmo should render at object position, not origin"
    );
}

#[test]
fn overlay_draw_unattached_uses_self_position() {
    let g = Gizmo::new(GizmoMode::Translate).position(Vec3::new(7.0, 0.0, 5.0));
    let ctx = scene_ctx(
        Vec3::ZERO,
        Vec3::Z,
        std::f32::consts::FRAC_PI_4,
        Vec2::new(800.0, 600.0),
        HashMap::new(),
    );
    let groups = g.draw(&ctx);
    // X axis arrow: translation should reflect the gizmo position (7.0)
    let model = groups[0].instances[0].model;
    let tx = model[3][0];
    assert!(tx.abs() > 0.1, "gizmo should render at self.position");
}

// ──────────── SceneHandle-based update ────────────

#[test]
fn update_returns_none_before_first_frame() {
    let mut g = Gizmo::default();
    let handle = SceneHandle::new(); // empty — no draw() has happened yet
    let result = g.update(Vec2::ZERO, false, &handle);
    assert!(
        result.is_none(),
        "should return None when no camera data available"
    );
}

#[test]
fn update_syncs_position_from_attached_object() {
    let target_id = SceneObjectId(1);
    let object_pos = Vec3::new(10.0, 20.0, 30.0);

    let mut objects = HashMap::new();
    objects.insert(target_id, Mat4::from_translation(object_pos));
    let handle = populated_handle(
        Vec3::ZERO,
        Vec3::Z,
        std::f32::consts::FRAC_PI_4,
        Vec2::new(800.0, 600.0),
        objects,
    );

    let mut g = Gizmo::new(GizmoMode::Translate).attach_to(target_id);
    assert_eq!(g.gizmo_position(), Vec3::ZERO, "starts at origin");

    g.update(Vec2::ZERO, false, &handle);
    assert_eq!(
        g.gizmo_position(),
        object_pos,
        "should sync to object position"
    );
}
