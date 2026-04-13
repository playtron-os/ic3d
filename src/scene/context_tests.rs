//! Tests for scene context and scene handle.

use super::*;
use crate::camera::CameraInfo;
use crate::gizmo::GizmoMode;
use glam::{Mat4, Vec2, Vec3};

fn test_camera() -> CameraInfo {
    CameraInfo {
        position: Vec3::new(0.0, 5.0, 10.0),
        forward: Vec3::NEG_Z,
        fov_y: Some(std::f32::consts::FRAC_PI_4),
        view_projection: Mat4::IDENTITY,
    }
}

fn test_context() -> SceneContext {
    let mut objects = HashMap::new();
    objects.insert(
        SceneObjectId(1),
        Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0)),
    );
    objects.insert(
        SceneObjectId(2),
        Mat4::from_translation(Vec3::new(4.0, 5.0, 6.0)),
    );
    SceneContext {
        camera: test_camera(),
        viewport_size: Vec2::new(800.0, 600.0),
        objects,
    }
}

#[test]
fn scene_context_object_position() {
    let ctx = test_context();
    let pos = ctx.object_position(SceneObjectId(1)).unwrap();
    assert!((pos.x - 1.0).abs() < 1e-6);
    assert!((pos.y - 2.0).abs() < 1e-6);
    assert!((pos.z - 3.0).abs() < 1e-6);
}

#[test]
fn scene_context_object_position_missing() {
    let ctx = test_context();
    assert!(ctx.object_position(SceneObjectId(99)).is_none());
}

#[test]
fn scene_handle_default_empty() {
    let handle = SceneHandle::new();
    assert!(handle.camera().is_none());
    assert_eq!(handle.viewport_size(), Vec2::ZERO);
    assert!(handle.object_transform(SceneObjectId(1)).is_none());
    assert!(handle.object_position(SceneObjectId(1)).is_none());
}

#[test]
fn scene_handle_update_and_read() {
    let handle = SceneHandle::new();
    handle.update_context(test_context());

    let cam = handle.camera().unwrap();
    assert!((cam.position.y - 5.0).abs() < 1e-6);
    assert_eq!(handle.viewport_size(), Vec2::new(800.0, 600.0));

    let pos = handle.object_position(SceneObjectId(2)).unwrap();
    assert!((pos.x - 4.0).abs() < 1e-6);
}

#[test]
fn scene_handle_clone_shares_state() {
    let a = SceneHandle::new();
    let b = a.clone();

    a.update_context(test_context());
    assert!(b.camera().is_some(), "clone should share state");
}

#[test]
fn scene_handle_object_transform() {
    let handle = SceneHandle::new();
    handle.update_context(test_context());

    let mat = handle.object_transform(SceneObjectId(1)).unwrap();
    assert!((mat.col(3).x - 1.0).abs() < 1e-6);
}

#[test]
fn scene_handle_select_adds_gizmo() {
    let handle = SceneHandle::new();
    assert!(handle.selected().is_empty());

    handle.select(SceneObjectId(1), GizmoMode::Translate);
    let selected = handle.selected();
    assert_eq!(selected.len(), 1);
    assert!(selected.contains(&SceneObjectId(1)));
}

#[test]
fn scene_handle_multi_select() {
    let handle = SceneHandle::new();
    handle.select(SceneObjectId(1), GizmoMode::Translate);
    handle.select(SceneObjectId(2), GizmoMode::Translate);

    let selected = handle.selected();
    assert_eq!(selected.len(), 2);
    assert!(selected.contains(&SceneObjectId(1)));
    assert!(selected.contains(&SceneObjectId(2)));
}

#[test]
fn scene_handle_select_replaces_same_id() {
    let handle = SceneHandle::new();
    handle.select(SceneObjectId(1), GizmoMode::Translate);
    handle.select(SceneObjectId(1), GizmoMode::Translate);

    assert_eq!(handle.selected().len(), 1);
}

#[test]
fn scene_handle_deselect_one() {
    let handle = SceneHandle::new();
    handle.select(SceneObjectId(1), GizmoMode::Translate);
    handle.select(SceneObjectId(2), GizmoMode::Translate);

    handle.deselect(SceneObjectId(1));
    let selected = handle.selected();
    assert_eq!(selected.len(), 1);
    assert!(selected.contains(&SceneObjectId(2)));
}

#[test]
fn scene_handle_deselect_all() {
    let handle = SceneHandle::new();
    handle.select(SceneObjectId(1), GizmoMode::Translate);
    handle.select(SceneObjectId(2), GizmoMode::Translate);

    handle.deselect_all();
    assert!(handle.selected().is_empty());
}

#[test]
fn scene_handle_is_selected() {
    let handle = SceneHandle::new();
    handle.select(SceneObjectId(1), GizmoMode::Translate);

    assert!(handle.is_selected(SceneObjectId(1)));
    assert!(!handle.is_selected(SceneObjectId(2)));
}

#[test]
fn scene_handle_is_dragging_empty() {
    let handle = SceneHandle::new();
    assert!(!handle.is_dragging());
}

#[test]
fn scene_handle_is_dragging_with_selection() {
    let handle = SceneHandle::new();
    handle.select(SceneObjectId(1), GizmoMode::Translate);
    // No interaction yet — not dragging.
    assert!(!handle.is_dragging());
}

#[test]
fn scene_handle_process_gizmo_empty() {
    let handle = SceneHandle::new();
    handle.update_context(test_context());
    // No selections — should return None.
    assert!(handle.process_gizmo(Vec2::ZERO, false).is_none());
}

#[test]
fn scene_handle_gizmo_overlays_empty() {
    let handle = SceneHandle::new();
    handle.update_context(test_context());
    assert!(handle.gizmo_overlays().is_empty());
}

#[test]
fn scene_handle_gizmo_overlays_with_selection() {
    let handle = SceneHandle::new();
    handle.update_context(test_context());
    handle.select(SceneObjectId(1), GizmoMode::Translate);
    // Should produce overlay draw groups (3 axes per gizmo).
    let overlays = handle.gizmo_overlays();
    assert_eq!(overlays.len(), 3);
}

#[test]
fn scene_handle_gizmo_overlays_multi_select() {
    let handle = SceneHandle::new();
    handle.update_context(test_context());
    handle.select(SceneObjectId(1), GizmoMode::Translate);
    handle.select(SceneObjectId(2), GizmoMode::Translate);
    // Two gizmos × 3 axes = 6 overlay draw groups.
    let overlays = handle.gizmo_overlays();
    assert_eq!(overlays.len(), 6);
}

#[test]
fn scene_handle_process_gizmo_skips_non_interactive() {
    let handle = SceneHandle::new();
    handle.update_context(test_context());
    // Select and make non-interactive.
    handle.select(SceneObjectId(1), GizmoMode::Translate);
    handle.modify_gizmo(SceneObjectId(1), |g| g.set_interactive(false));

    // Cursor at center — would normally hit the gizmo origin area.
    let result = handle.process_gizmo(Vec2::new(400.0, 300.0), false);
    assert!(
        result.is_none(),
        "non-interactive gizmo should not produce results"
    );
}

#[test]
fn scene_handle_non_interactive_gizmo_still_draws() {
    let handle = SceneHandle::new();
    handle.update_context(test_context());
    handle.select(SceneObjectId(1), GizmoMode::Translate);
    handle.modify_gizmo(SceneObjectId(1), |g| g.set_interactive(false));

    // Non-interactive but visible — should still produce overlay draw groups.
    let overlays = handle.gizmo_overlays();
    assert_eq!(
        overlays.len(),
        3,
        "non-interactive gizmo should still be drawn"
    );
}

#[test]
fn scene_handle_modify_gizmo() {
    let handle = SceneHandle::new();
    handle.select(SceneObjectId(1), GizmoMode::Translate);

    handle.modify_gizmo(SceneObjectId(1), |g| g.set_gizmo_size(120.0));

    // Modify non-existent gizmo should be a no-op.
    handle.modify_gizmo(SceneObjectId(99), |g| g.set_gizmo_size(999.0));
}
