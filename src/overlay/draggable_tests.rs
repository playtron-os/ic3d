use super::*;
use crate::camera::{CameraInfo, PerspectiveCamera};
use crate::graph::node::{Node, NodeKind};
use crate::overlay::base::OverlayEvent;
use crate::scene::context::SceneHandle;
use glam::Vec3;
use std::collections::HashMap;

/// Test draggable overlay that records drag deltas.
#[derive(Debug, Clone)]
struct TestDraggable {
    target: Option<SceneObjectId>,
    last_delta: Option<Vec2>,
    drag_count: usize,
    custom_hit_radius: f32,
}

impl TestDraggable {
    fn new(target: SceneObjectId) -> Self {
        Self {
            target: Some(target),
            last_delta: None,
            drag_count: 0,
            custom_hit_radius: 10000.0, // large radius so any cursor position hits
        }
    }
}

impl DraggableOverlay for TestDraggable {
    fn resolve_target(&self, _handle: &SceneHandle) -> Option<SceneObjectId> {
        self.target
    }

    fn hit_radius(&self) -> f32 {
        self.custom_hit_radius
    }

    fn on_drag(&mut self, delta: Vec2, _ctx: &mut OverlayContext) {
        self.last_delta = Some(delta);
        self.drag_count += 1;
    }

    fn draw_overlay(
        &self,
        _target: SceneObjectId,
        _state: &DragState,
        _ctx: &SceneContext,
    ) -> Vec<MeshDrawGroup> {
        Vec::new()
    }
}

/// Set up a scene handle with an object at world origin.
fn setup_handle(target: SceneObjectId) -> SceneHandle {
    let handle = SceneHandle::new();
    let cam = PerspectiveCamera::new().position(Vec3::new(0.0, 0.0, 5.0));
    let mut objects = HashMap::new();
    objects.insert(target, glam::Mat4::IDENTITY);
    handle.update_context(crate::scene::context::SceneContext {
        camera: CameraInfo::from_camera(&cam),
        viewport_size: Vec2::new(800.0, 600.0),
        objects,
    });
    handle
}

/// Helper: wrap a `TestDraggable` in `Draggable<T>`.
fn make_draggable(target: SceneObjectId) -> Draggable<TestDraggable> {
    Draggable::new(TestDraggable::new(target))
}

/// Helper: wrap with custom hit radius.
fn make_draggable_with_radius(target: SceneObjectId, radius: f32) -> Draggable<TestDraggable> {
    let mut td = TestDraggable::new(target);
    td.custom_hit_radius = radius;
    Draggable::new(td)
}

// ──────────── DragState ────────────

#[test]
fn drag_state_default_is_idle() {
    let state = DragState::default();
    assert!(!state.is_hovered());
    assert!(!state.is_dragging());
    assert!(!state.is_active());
}

#[test]
fn drag_state_reset() {
    let mut state = DragState {
        hovered: true,
        last_cursor: Some(Vec2::new(100.0, 200.0)),
    };
    assert!(state.is_active());
    state.reset();
    assert!(!state.is_hovered());
    assert!(!state.is_dragging());
}

// ──────────── Draggable wrapper ────────────

#[test]
fn draggable_new_starts_idle() {
    let target = SceneObjectId::new();
    let d = make_draggable(target);
    assert!(!d.drag_state().is_hovered());
    assert!(!d.drag_state().is_dragging());
    assert!(d.target().is_none()); // no cached target until on_input runs
}

#[test]
fn draggable_inner_access() {
    let target = SceneObjectId::new();
    let mut d = make_draggable(target);
    assert_eq!(d.inner().target, Some(target));
    d.inner_mut().custom_hit_radius = 42.0;
    assert!((d.inner().custom_hit_radius - 42.0).abs() < f32::EPSILON);
}

// ──────────── Overlay impl via Draggable ────────────

#[test]
fn hover_detected_on_hit() {
    let target = SceneObjectId::new();
    let handle = setup_handle(target);
    let mut nodes = HashMap::new();
    nodes.insert(target, Node::new(target, NodeKind::Empty));

    let mut overlay = make_draggable(target);
    let input = OverlayInput {
        cursor: Vec2::new(400.0, 300.0),
        mouse_pressed: false,
    };
    let mut ctx = OverlayContext::new(&mut nodes, &handle);
    overlay.on_input(&input, &mut ctx);

    assert!(overlay.drag_state().is_hovered());
    assert!(!overlay.drag_state().is_dragging());
}

#[test]
fn drag_starts_on_click() {
    let target = SceneObjectId::new();
    let handle = setup_handle(target);
    let mut nodes = HashMap::new();
    nodes.insert(target, Node::new(target, NodeKind::Empty));

    let mut overlay = make_draggable(target);
    let input = OverlayInput {
        cursor: Vec2::new(400.0, 300.0),
        mouse_pressed: true,
    };
    let mut ctx = OverlayContext::new(&mut nodes, &handle);
    overlay.on_input(&input, &mut ctx);

    assert!(overlay.drag_state().is_hovered());
    assert!(overlay.drag_state().is_dragging());
}

#[test]
fn drag_produces_delta() {
    let target = SceneObjectId::new();
    let handle = setup_handle(target);
    let mut nodes = HashMap::new();
    nodes.insert(target, Node::new(target, NodeKind::Empty));

    let mut overlay = make_draggable(target);

    // Start drag.
    let mut ctx = OverlayContext::new(&mut nodes, &handle);
    overlay.on_input(
        &OverlayInput {
            cursor: Vec2::new(400.0, 300.0),
            mouse_pressed: true,
        },
        &mut ctx,
    );
    assert!(overlay.drag_state().is_dragging());

    // Move cursor — should produce delta.
    let mut ctx = OverlayContext::new(&mut nodes, &handle);
    overlay.on_input(
        &OverlayInput {
            cursor: Vec2::new(410.0, 280.0),
            mouse_pressed: true,
        },
        &mut ctx,
    );

    assert_eq!(overlay.inner().drag_count, 1);
    let delta = overlay.inner().last_delta.unwrap();
    assert!((delta.x - 10.0).abs() < 1e-6);
    assert!((delta.y - (-20.0)).abs() < 1e-6);
}

#[test]
fn drag_ends_on_release() {
    let target = SceneObjectId::new();
    let handle = setup_handle(target);
    let mut nodes = HashMap::new();
    nodes.insert(target, Node::new(target, NodeKind::Empty));

    let mut overlay = make_draggable(target);

    // Start drag.
    let mut ctx = OverlayContext::new(&mut nodes, &handle);
    overlay.on_input(
        &OverlayInput {
            cursor: Vec2::new(400.0, 300.0),
            mouse_pressed: true,
        },
        &mut ctx,
    );
    assert!(overlay.drag_state().is_dragging());

    // Release.
    let mut ctx = OverlayContext::new(&mut nodes, &handle);
    overlay.on_input(
        &OverlayInput {
            cursor: Vec2::new(400.0, 300.0),
            mouse_pressed: false,
        },
        &mut ctx,
    );

    assert!(!overlay.drag_state().is_dragging());
    assert!(overlay.drag_state().is_hovered()); // cursor still over target
}

#[test]
fn no_target_is_noop() {
    let target = SceneObjectId::new();
    let handle = setup_handle(target);
    let mut nodes = HashMap::new();

    let mut td = TestDraggable::new(target);
    td.target = None;
    let mut overlay = Draggable::new(td);

    let mut ctx = OverlayContext::new(&mut nodes, &handle);
    overlay.on_input(
        &OverlayInput {
            cursor: Vec2::new(400.0, 300.0),
            mouse_pressed: true,
        },
        &mut ctx,
    );

    assert!(!overlay.drag_state().is_hovered());
    assert!(!overlay.drag_state().is_dragging());
}

#[test]
fn draw_delegates_to_draw_overlay() {
    let target = SceneObjectId::new();
    let handle = setup_handle(target);
    let mut nodes = HashMap::new();
    nodes.insert(target, Node::new(target, NodeKind::Empty));

    let mut overlay = make_draggable(target);

    // Run on_input first to cache the target.
    let mut ctx = OverlayContext::new(&mut nodes, &handle);
    overlay.on_input(
        &OverlayInput {
            cursor: Vec2::new(400.0, 300.0),
            mouse_pressed: false,
        },
        &mut ctx,
    );

    let cam = PerspectiveCamera::new();
    let ctx = crate::scene::context::SceneContext {
        camera: CameraInfo::from_camera(&cam),
        viewport_size: Vec2::new(800.0, 600.0),
        objects: HashMap::new(),
    };
    let groups = Overlay::draw(&overlay, &ctx);
    assert!(groups.is_empty());
}

#[test]
fn draw_returns_empty_without_target() {
    let target = SceneObjectId::new();

    let mut td = TestDraggable::new(target);
    td.target = None;
    let overlay = Draggable::new(td);

    let cam = PerspectiveCamera::new();
    let ctx = crate::scene::context::SceneContext {
        camera: CameraInfo::from_camera(&cam),
        viewport_size: Vec2::new(800.0, 600.0),
        objects: HashMap::new(),
    };
    let groups = Overlay::draw(&overlay, &ctx);
    assert!(groups.is_empty());
}

#[test]
fn small_hit_radius_misses() {
    let target = SceneObjectId::new();
    let handle = setup_handle(target);
    let mut nodes = HashMap::new();
    nodes.insert(target, Node::new(target, NodeKind::Empty));

    let mut overlay = make_draggable_with_radius(target, 1.0);

    // Cursor far from projected center.
    let mut ctx = OverlayContext::new(&mut nodes, &handle);
    overlay.on_input(
        &OverlayInput {
            cursor: Vec2::new(0.0, 0.0),
            mouse_pressed: true,
        },
        &mut ctx,
    );

    assert!(!overlay.drag_state().is_hovered());
    assert!(!overlay.drag_state().is_dragging());
}

#[test]
fn tiny_delta_ignored() {
    let target = SceneObjectId::new();
    let handle = setup_handle(target);
    let mut nodes = HashMap::new();
    nodes.insert(target, Node::new(target, NodeKind::Empty));

    let mut overlay = make_draggable(target);

    // Start drag.
    let mut ctx = OverlayContext::new(&mut nodes, &handle);
    overlay.on_input(
        &OverlayInput {
            cursor: Vec2::new(400.0, 300.0),
            mouse_pressed: true,
        },
        &mut ctx,
    );

    // Move by a tiny amount (below threshold).
    let mut ctx = OverlayContext::new(&mut nodes, &handle);
    overlay.on_input(
        &OverlayInput {
            cursor: Vec2::new(400.0, 300.000_000_1),
            mouse_pressed: true,
        },
        &mut ctx,
    );

    assert_eq!(overlay.inner().drag_count, 0); // on_drag not called
}

#[test]
fn target_cached_after_on_input() {
    let target = SceneObjectId::new();
    let handle = setup_handle(target);
    let mut nodes = HashMap::new();
    nodes.insert(target, Node::new(target, NodeKind::Empty));

    let mut overlay = make_draggable(target);
    assert!(overlay.target().is_none()); // not cached yet

    let mut ctx = OverlayContext::new(&mut nodes, &handle);
    overlay.on_input(
        &OverlayInput {
            cursor: Vec2::new(400.0, 300.0),
            mouse_pressed: false,
        },
        &mut ctx,
    );

    assert_eq!(overlay.target(), Some(target));
}

// ──────────── Event lifecycle ────────────

#[test]
fn events_hover_start_on_enter() {
    let target = SceneObjectId::new();
    let handle = setup_handle(target);
    let mut nodes = HashMap::new();
    nodes.insert(target, Node::new(target, NodeKind::Empty));

    let mut overlay = make_draggable(target);

    let mut ctx = OverlayContext::new(&mut nodes, &handle);
    let events = overlay.on_input(
        &OverlayInput {
            cursor: Vec2::new(400.0, 300.0),
            mouse_pressed: false,
        },
        &mut ctx,
    );

    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], OverlayEvent::HoverStart));
}

#[test]
fn events_no_repeat_while_hovering() {
    let target = SceneObjectId::new();
    let handle = setup_handle(target);
    let mut nodes = HashMap::new();
    nodes.insert(target, Node::new(target, NodeKind::Empty));

    let mut overlay = make_draggable(target);

    // First hover → HoverStart
    let mut ctx = OverlayContext::new(&mut nodes, &handle);
    overlay.on_input(
        &OverlayInput {
            cursor: Vec2::new(400.0, 300.0),
            mouse_pressed: false,
        },
        &mut ctx,
    );

    // Stay hovering → no events
    let mut ctx = OverlayContext::new(&mut nodes, &handle);
    let events = overlay.on_input(
        &OverlayInput {
            cursor: Vec2::new(401.0, 300.0),
            mouse_pressed: false,
        },
        &mut ctx,
    );

    assert!(events.is_empty());
}

#[test]
fn events_hover_end_on_leave() {
    let target = SceneObjectId::new();
    let handle = setup_handle(target);
    let mut nodes = HashMap::new();
    nodes.insert(target, Node::new(target, NodeKind::Empty));

    let mut overlay = make_draggable_with_radius(target, 10.0);

    // Hover near center
    let mut ctx = OverlayContext::new(&mut nodes, &handle);
    overlay.on_input(
        &OverlayInput {
            cursor: Vec2::new(400.0, 300.0),
            mouse_pressed: false,
        },
        &mut ctx,
    );

    // Move far away → HoverEnd
    let mut ctx = OverlayContext::new(&mut nodes, &handle);
    let events = overlay.on_input(
        &OverlayInput {
            cursor: Vec2::new(0.0, 0.0),
            mouse_pressed: false,
        },
        &mut ctx,
    );

    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], OverlayEvent::HoverEnd));
}

#[test]
fn events_drag_start_on_click() {
    let target = SceneObjectId::new();
    let handle = setup_handle(target);
    let mut nodes = HashMap::new();
    nodes.insert(target, Node::new(target, NodeKind::Empty));

    let mut overlay = make_draggable(target);

    // Click while over target
    let mut ctx = OverlayContext::new(&mut nodes, &handle);
    let events = overlay.on_input(
        &OverlayInput {
            cursor: Vec2::new(400.0, 300.0),
            mouse_pressed: true,
        },
        &mut ctx,
    );

    assert!(events.iter().any(|e| matches!(e, OverlayEvent::HoverStart)));
    assert!(events.iter().any(|e| matches!(e, OverlayEvent::DragStart)));
}

#[test]
fn events_drag_move_on_cursor_move() {
    let target = SceneObjectId::new();
    let handle = setup_handle(target);
    let mut nodes = HashMap::new();
    nodes.insert(target, Node::new(target, NodeKind::Empty));

    let mut overlay = make_draggable(target);

    // Start drag
    let mut ctx = OverlayContext::new(&mut nodes, &handle);
    overlay.on_input(
        &OverlayInput {
            cursor: Vec2::new(400.0, 300.0),
            mouse_pressed: true,
        },
        &mut ctx,
    );

    // Move while dragging
    let mut ctx = OverlayContext::new(&mut nodes, &handle);
    let events = overlay.on_input(
        &OverlayInput {
            cursor: Vec2::new(410.0, 310.0),
            mouse_pressed: true,
        },
        &mut ctx,
    );

    assert_eq!(events.len(), 1);
    match &events[0] {
        OverlayEvent::DragMove(delta) => {
            assert!((delta.x - 10.0).abs() < 0.001);
            assert!((delta.y - 10.0).abs() < 0.001);
        }
        other => panic!("expected DragMove, got {other:?}"),
    }
}

#[test]
fn events_drag_end_on_release() {
    let target = SceneObjectId::new();
    let handle = setup_handle(target);
    let mut nodes = HashMap::new();
    nodes.insert(target, Node::new(target, NodeKind::Empty));

    let mut overlay = make_draggable(target);

    // Start drag
    let mut ctx = OverlayContext::new(&mut nodes, &handle);
    overlay.on_input(
        &OverlayInput {
            cursor: Vec2::new(400.0, 300.0),
            mouse_pressed: true,
        },
        &mut ctx,
    );

    // Release
    let mut ctx = OverlayContext::new(&mut nodes, &handle);
    let events = overlay.on_input(
        &OverlayInput {
            cursor: Vec2::new(400.0, 300.0),
            mouse_pressed: false,
        },
        &mut ctx,
    );

    assert!(events.iter().any(|e| matches!(e, OverlayEvent::DragEnd)));
}

#[test]
fn events_full_lifecycle() {
    let target = SceneObjectId::new();
    let handle = setup_handle(target);
    let mut nodes = HashMap::new();
    nodes.insert(target, Node::new(target, NodeKind::Empty));

    let mut overlay = make_draggable_with_radius(target, 10.0);

    let mut all_events = Vec::new();

    // 1. Hover → HoverStart
    let mut ctx = OverlayContext::new(&mut nodes, &handle);
    all_events.extend(overlay.on_input(
        &OverlayInput {
            cursor: Vec2::new(400.0, 300.0),
            mouse_pressed: false,
        },
        &mut ctx,
    ));

    // 2. Click → DragStart
    let mut ctx = OverlayContext::new(&mut nodes, &handle);
    all_events.extend(overlay.on_input(
        &OverlayInput {
            cursor: Vec2::new(400.0, 300.0),
            mouse_pressed: true,
        },
        &mut ctx,
    ));

    // 3. Drag → DragMove
    let mut ctx = OverlayContext::new(&mut nodes, &handle);
    all_events.extend(overlay.on_input(
        &OverlayInput {
            cursor: Vec2::new(420.0, 310.0),
            mouse_pressed: true,
        },
        &mut ctx,
    ));

    // 4. Release → DragEnd
    let mut ctx = OverlayContext::new(&mut nodes, &handle);
    all_events.extend(overlay.on_input(
        &OverlayInput {
            cursor: Vec2::new(420.0, 310.0),
            mouse_pressed: false,
        },
        &mut ctx,
    ));

    // 5. Move away → HoverEnd
    let mut ctx = OverlayContext::new(&mut nodes, &handle);
    all_events.extend(overlay.on_input(
        &OverlayInput {
            cursor: Vec2::new(0.0, 0.0),
            mouse_pressed: false,
        },
        &mut ctx,
    ));

    let event_types: Vec<&str> = all_events
        .iter()
        .map(|e| match e {
            OverlayEvent::HoverStart => "HoverStart",
            OverlayEvent::HoverEnd => "HoverEnd",
            OverlayEvent::DragStart => "DragStart",
            OverlayEvent::DragMove(_) => "DragMove",
            OverlayEvent::DragEnd => "DragEnd",
            OverlayEvent::Custom(_) => "Custom",
        })
        .collect();

    assert_eq!(
        event_types,
        vec!["HoverStart", "DragStart", "DragMove", "DragEnd", "HoverEnd"]
    );
}
