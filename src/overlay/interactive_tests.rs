//! Tests for InteractiveOverlay trait and Interactive wrapper.

use super::*;
use crate::camera::CameraInfo;
use crate::math::HitShape;
use crate::overlay::base::{OverlayContext, OverlayEvent, OverlayInput};
use crate::scene::context::{SceneContext, SceneHandle};
use crate::scene::object::SceneObjectId;
use crate::widget::MeshDrawGroup;
use glam::{Mat4, Vec2, Vec3};
use std::collections::HashMap;

// ── Test fixture ──

#[derive(Debug, Clone)]
struct TestOverlay {
    target: Option<SceneObjectId>,
    hovered_shape: Option<usize>,
    dragging: bool,
    drag_cursors: Vec<Vec2>,
    drag_start_count: u32,
    drag_end_count: u32,
    custom_screen_size: Option<f32>,
}

impl TestOverlay {
    fn new(target: SceneObjectId) -> Self {
        Self {
            target: Some(target),
            hovered_shape: None,
            dragging: false,
            drag_cursors: Vec::new(),
            drag_start_count: 0,
            drag_end_count: 0,
            custom_screen_size: None,
        }
    }
}

impl InteractiveOverlay for TestOverlay {
    fn resolve_target(&self, _handle: &SceneHandle) -> Option<SceneObjectId> {
        self.target
    }

    fn screen_size(&self) -> f32 {
        self.custom_screen_size.unwrap_or(80.0)
    }

    fn hit_shapes(&self, ctx: &InteractiveContext) -> Vec<HitShape> {
        // 3 axis segments
        vec![
            HitShape::segment(ctx.position, ctx.position + Vec3::X * ctx.scale, 20.0),
            HitShape::segment(ctx.position, ctx.position + Vec3::Y * ctx.scale, 20.0),
            HitShape::segment(ctx.position, ctx.position + Vec3::Z * ctx.scale, 20.0),
        ]
    }

    fn on_hover(&mut self, hit: &ShapeHit) {
        self.hovered_shape = Some(hit.shape_index);
    }

    fn on_unhover(&mut self) {
        self.hovered_shape = None;
    }

    fn on_drag_start(
        &mut self,
        _hit: &ShapeHit,
        cursor: Vec2,
        _ctx: &InteractiveContext,
        _nodes: &mut OverlayContext,
    ) -> bool {
        self.dragging = true;
        self.drag_start_count += 1;
        self.drag_cursors.push(cursor);
        true
    }

    fn on_drag_continue(
        &mut self,
        cursor: Vec2,
        _ctx: &InteractiveContext,
        _nodes: &mut OverlayContext,
    ) {
        self.drag_cursors.push(cursor);
    }

    fn on_drag_end(&mut self, _nodes: &mut OverlayContext) {
        self.dragging = false;
        self.drag_end_count += 1;
    }

    fn is_dragging(&self) -> bool {
        self.dragging
    }

    fn draw(&self, _ctx: &InteractiveContext) -> Vec<MeshDrawGroup> {
        Vec::new()
    }
}

// ── Helpers ──

fn make_camera() -> CameraInfo {
    let eye = Vec3::new(0.0, 0.0, 10.0);
    let target = Vec3::ZERO;
    let view = Mat4::look_at_rh(eye, target, Vec3::Y);
    let proj = Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, 1.0, 0.1, 100.0);
    CameraInfo {
        position: eye,
        forward: (target - eye).normalize(),
        fov_y: Some(std::f32::consts::FRAC_PI_4),
        view_projection: proj * view,
    }
}

fn make_scene_context(camera: &CameraInfo, objects: HashMap<SceneObjectId, Mat4>) -> SceneContext {
    SceneContext {
        camera: *camera,
        viewport_size: Vec2::new(800.0, 600.0),
        objects,
    }
}

fn make_scene_handle_with_object(id: SceneObjectId, pos: Vec3) -> SceneHandle {
    let handle = SceneHandle::new();
    let camera = make_camera();
    let mut objects = HashMap::new();
    objects.insert(id, Mat4::from_translation(pos));
    let ctx = SceneContext {
        camera,
        viewport_size: Vec2::new(800.0, 600.0),
        objects,
    };
    handle.update_context(ctx);
    handle
}

// ── Tests: Construction ──

#[test]
fn interactive_new_defaults() {
    let overlay = TestOverlay::new(SceneObjectId(1));
    let interactive = Interactive::new(overlay);

    assert!(interactive.target().is_none());
    assert!(!interactive.is_hovered());
}

#[test]
fn interactive_inner_access() {
    let overlay = TestOverlay::new(SceneObjectId(1));
    let mut interactive = Interactive::new(overlay);

    assert_eq!(interactive.inner().target, Some(SceneObjectId(1)));
    interactive.inner_mut().target = Some(SceneObjectId(2));
    assert_eq!(interactive.inner().target, Some(SceneObjectId(2)));
}

// ── Tests: Visibility ──

#[test]
fn visible_when_target_set() {
    let overlay = TestOverlay::new(SceneObjectId(1));
    let mut interactive = Interactive::new(overlay);
    interactive.target = Some(SceneObjectId(1));
    assert!(Overlay::visible(&interactive));
}

#[test]
fn always_visible_so_on_input_can_resolve_target() {
    let overlay = TestOverlay::new(SceneObjectId(1));
    let interactive = Interactive::new(overlay);
    // Always visible — on_input resolves target; draw/hit_shapes return
    // empty when target is None.
    assert!(Overlay::visible(&interactive));
}

// ── Tests: Hit shapes delegation ──

#[test]
fn hit_shapes_delegates_to_inner() {
    let id = SceneObjectId(1);
    let overlay = TestOverlay::new(id);
    let mut interactive = Interactive::new(overlay);
    interactive.target = Some(id);

    let camera = make_camera();
    let mut objects = HashMap::new();
    objects.insert(id, Mat4::from_translation(Vec3::ZERO));
    let ctx = make_scene_context(&camera, objects);

    let shapes = Overlay::hit_shapes(&interactive, &ctx);
    assert_eq!(shapes.len(), 3);
}

#[test]
fn hit_shapes_empty_without_target() {
    let overlay = TestOverlay::new(SceneObjectId(1));
    let interactive = Interactive::new(overlay);

    let camera = make_camera();
    let ctx = make_scene_context(&camera, HashMap::new());

    let shapes = Overlay::hit_shapes(&interactive, &ctx);
    assert!(shapes.is_empty());
}

// ── Tests: Draw delegation ──

#[test]
fn draw_delegates_to_inner() {
    let id = SceneObjectId(1);
    let overlay = TestOverlay::new(id);
    let mut interactive = Interactive::new(overlay);
    interactive.target = Some(id);

    let camera = make_camera();
    let mut objects = HashMap::new();
    objects.insert(id, Mat4::from_translation(Vec3::ZERO));
    let ctx = make_scene_context(&camera, objects);

    // TestOverlay.draw returns empty vec, but it doesn't crash
    let groups = Overlay::draw(&interactive, &ctx);
    assert!(groups.is_empty());
}

// ── Tests: Context building ──

#[test]
fn build_context_returns_none_without_target() {
    let overlay = TestOverlay::new(SceneObjectId(1));
    let interactive = Interactive::new(overlay);

    let camera = make_camera();
    let ctx = make_scene_context(&camera, HashMap::new());

    assert!(interactive.build_context(&ctx).is_none());
}

#[test]
fn build_context_returns_some_with_valid_target() {
    let id = SceneObjectId(1);
    let overlay = TestOverlay::new(id);
    let mut interactive = Interactive::new(overlay);
    interactive.target = Some(id);

    let camera = make_camera();
    let mut objects = HashMap::new();
    objects.insert(id, Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0)));
    let ctx = make_scene_context(&camera, objects);

    let ictx = interactive.build_context(&ctx).unwrap();
    assert_eq!(ictx.position, Vec3::new(1.0, 2.0, 3.0));
    assert!(ictx.scale > 0.0);
    assert_eq!(ictx.viewport, Vec2::new(800.0, 600.0));
}

// ── Tests: Input - hover ──

#[test]
fn on_input_hover_near_axis() {
    let id = SceneObjectId(1);
    let overlay = TestOverlay::new(id);
    let mut interactive = Interactive::new(overlay);
    let handle = make_scene_handle_with_object(id, Vec3::ZERO);

    // Project the X axis endpoint to screen to find a cursor position on it.
    let camera = make_camera();
    let vp = Vec2::new(800.0, 600.0);
    let center_screen =
        crate::math::world_to_screen(Vec3::ZERO, camera.view_projection, vp).unwrap();

    // Cursor slightly right of center should hit the X-axis segment.
    let cursor = center_screen + Vec2::new(5.0, 0.0);
    let input = OverlayInput {
        cursor,
        mouse_pressed: false,
    };
    let mut nodes = HashMap::new();
    let mut ctx = OverlayContext::new(&mut nodes, &handle);
    let events = interactive.on_input(&input, &mut ctx);

    assert!(interactive.is_hovered());
    assert!(events.iter().any(|e| matches!(e, OverlayEvent::HoverStart)));
    assert_eq!(interactive.inner().hovered_shape, Some(0)); // X axis
}

#[test]
fn on_input_unhover_far_cursor() {
    let id = SceneObjectId(1);
    let overlay = TestOverlay::new(id);
    let mut interactive = Interactive::new(overlay);
    interactive.hovered = true;
    let handle = make_scene_handle_with_object(id, Vec3::ZERO);

    // Cursor very far from any axis.
    let input = OverlayInput {
        cursor: Vec2::new(10.0, 10.0),
        mouse_pressed: false,
    };
    let mut nodes = HashMap::new();
    let mut ctx = OverlayContext::new(&mut nodes, &handle);
    let events = interactive.on_input(&input, &mut ctx);

    assert!(!interactive.is_hovered());
    assert!(events.iter().any(|e| matches!(e, OverlayEvent::HoverEnd)));
}

// ── Tests: Input - drag lifecycle ──

#[test]
fn on_input_drag_start_on_click() {
    let id = SceneObjectId(1);
    let overlay = TestOverlay::new(id);
    let mut interactive = Interactive::new(overlay);
    let handle = make_scene_handle_with_object(id, Vec3::ZERO);

    let camera = make_camera();
    let vp = Vec2::new(800.0, 600.0);
    let center_screen =
        crate::math::world_to_screen(Vec3::ZERO, camera.view_projection, vp).unwrap();
    let cursor = center_screen + Vec2::new(5.0, 0.0);

    let input = OverlayInput {
        cursor,
        mouse_pressed: true,
    };
    let mut nodes = HashMap::new();
    let mut ctx = OverlayContext::new(&mut nodes, &handle);
    let events = interactive.on_input(&input, &mut ctx);

    assert!(interactive.inner().is_dragging());
    assert_eq!(interactive.inner().drag_start_count, 1);
    assert!(events.iter().any(|e| matches!(e, OverlayEvent::DragStart)));
}

#[test]
fn on_input_drag_continue() {
    let id = SceneObjectId(1);
    let mut overlay = TestOverlay::new(id);
    overlay.dragging = true;
    let mut interactive = Interactive::new(overlay);
    let handle = make_scene_handle_with_object(id, Vec3::ZERO);

    let input = OverlayInput {
        cursor: Vec2::new(410.0, 310.0),
        mouse_pressed: true,
    };
    let mut nodes = HashMap::new();
    let mut ctx = OverlayContext::new(&mut nodes, &handle);
    let events = interactive.on_input(&input, &mut ctx);

    assert!(interactive.inner().is_dragging());
    assert!(events
        .iter()
        .any(|e| matches!(e, OverlayEvent::DragMove(_))));
}

#[test]
fn on_input_drag_end_on_release() {
    let id = SceneObjectId(1);
    let mut overlay = TestOverlay::new(id);
    overlay.dragging = true;
    let mut interactive = Interactive::new(overlay);
    let handle = make_scene_handle_with_object(id, Vec3::ZERO);

    let input = OverlayInput {
        cursor: Vec2::new(400.0, 300.0),
        mouse_pressed: false,
    };
    let mut nodes = HashMap::new();
    let mut ctx = OverlayContext::new(&mut nodes, &handle);
    let events = interactive.on_input(&input, &mut ctx);

    assert!(!interactive.inner().is_dragging());
    assert_eq!(interactive.inner().drag_end_count, 1);
    assert!(events.iter().any(|e| matches!(e, OverlayEvent::DragEnd)));
}

// ── Tests: Target resolution ──

#[test]
fn on_input_clears_state_when_target_disappears() {
    let id = SceneObjectId(1);
    let overlay = TestOverlay::new(id);
    let mut interactive = Interactive::new(overlay);
    interactive.hovered = true;
    let handle = make_scene_handle_with_object(id, Vec3::ZERO);

    // Make target disappear.
    interactive.inner_mut().target = None;

    let input = OverlayInput {
        cursor: Vec2::new(400.0, 300.0),
        mouse_pressed: false,
    };
    let mut nodes = HashMap::new();
    let mut ctx = OverlayContext::new(&mut nodes, &handle);
    let events = interactive.on_input(&input, &mut ctx);

    assert!(!interactive.is_hovered());
    assert!(events.iter().any(|e| matches!(e, OverlayEvent::HoverEnd)));
}

// ── Tests: Screen size ──

#[test]
fn default_screen_size_is_80() {
    let overlay = TestOverlay::new(SceneObjectId(1));
    assert!((overlay.screen_size() - 80.0).abs() < f32::EPSILON);
}

#[test]
fn custom_screen_size() {
    let mut overlay = TestOverlay::new(SceneObjectId(1));
    overlay.custom_screen_size = Some(120.0);
    assert!((overlay.screen_size() - 120.0).abs() < f32::EPSILON);
}

// ── Tests: ShapeHit ──

#[test]
fn shape_hit_fields() {
    let hit = ShapeHit {
        shape_index: 2,
        distance: 5.5,
    };
    assert_eq!(hit.shape_index, 2);
    assert!((hit.distance - 5.5).abs() < f32::EPSILON);
}

// ── Tests: InteractiveContext ──

#[test]
fn interactive_context_fields() {
    let ctx = InteractiveContext {
        position: Vec3::new(1.0, 2.0, 3.0),
        scale: 0.5,
        camera: make_camera(),
        viewport: Vec2::new(1920.0, 1080.0),
    };
    assert_eq!(ctx.position, Vec3::new(1.0, 2.0, 3.0));
    assert!((ctx.scale - 0.5).abs() < f32::EPSILON);
    assert_eq!(ctx.viewport, Vec2::new(1920.0, 1080.0));
}
