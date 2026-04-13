use super::*;
use crate::camera::{CameraInfo, PerspectiveCamera};
use crate::scene::context::{SceneContext, SceneHandle};
use crate::scene::object::SceneObjectId;
use crate::widget::MeshDrawGroup;
use crate::{Mesh, Transform};
use glam::{Vec2, Vec3};
use std::collections::HashMap;

/// A simple test overlay that produces a fixed number of draw groups.
#[derive(Debug)]
struct TestOverlay {
    count: usize,
}

impl Overlay for TestOverlay {
    fn draw(&self, _ctx: &SceneContext) -> Vec<MeshDrawGroup> {
        (0..self.count)
            .map(|_| {
                MeshDrawGroup::new(
                    Mesh::cube(1.0),
                    vec![Transform::new().to_instance([1.0, 0.0, 0.0, 1.0])],
                )
            })
            .collect()
    }
}

/// An interactive overlay that scales its target node on input.
#[derive(Debug, Clone)]
struct ScaleOverlay {
    target: SceneObjectId,
    activated: bool,
}

impl Overlay for ScaleOverlay {
    fn draw(&self, _ctx: &SceneContext) -> Vec<MeshDrawGroup> {
        Vec::new()
    }

    fn on_input(&mut self, input: &OverlayInput, ctx: &mut OverlayContext) -> Vec<OverlayEvent> {
        if input.mouse_pressed {
            self.activated = true;
            if let Some(node) = ctx.node_mut(self.target) {
                node.add_uniform_scale(0.5);
            }
        }
        Vec::new()
    }
}

fn test_ctx() -> SceneContext {
    let cam = PerspectiveCamera::new().position(Vec3::new(0.0, 0.0, 5.0));
    SceneContext {
        camera: CameraInfo::from_camera(&cam),
        viewport_size: Vec2::new(800.0, 600.0),
        objects: HashMap::new(),
    }
}

#[test]
fn overlay_draw_returns_groups() {
    let overlay = TestOverlay { count: 3 };
    let groups = overlay.draw(&test_ctx());
    assert_eq!(groups.len(), 3);
}

#[test]
fn overlay_is_object_safe() {
    let overlay: Box<dyn Overlay> = Box::new(TestOverlay { count: 1 });
    let groups = overlay.draw(&test_ctx());
    assert_eq!(groups.len(), 1);
}

#[test]
fn on_input_default_is_noop() {
    let mut overlay = TestOverlay { count: 1 };
    let handle = SceneHandle::new();
    let mut nodes = HashMap::new();
    let input = OverlayInput {
        cursor: Vec2::ZERO,
        mouse_pressed: false,
    };
    let mut ctx = OverlayContext::new(&mut nodes, &handle);
    overlay.on_input(&input, &mut ctx);
    // No panic, no mutation — default is a no-op.
}

#[test]
fn on_input_can_mutate_nodes() {
    use crate::graph::node::{Node, NodeKind};

    let target = SceneObjectId::new();
    let mut nodes = HashMap::new();
    nodes.insert(target, Node::new(target, NodeKind::Empty));
    let handle = SceneHandle::new();

    let mut overlay = ScaleOverlay {
        target,
        activated: false,
    };
    let input = OverlayInput {
        cursor: Vec2::ZERO,
        mouse_pressed: true,
    };
    let mut ctx = OverlayContext::new(&mut nodes, &handle);
    overlay.on_input(&input, &mut ctx);

    assert!(overlay.activated);
    assert!((nodes[&target].uniform_scale() - 1.5).abs() < 1e-6);
}

#[test]
fn overlay_context_node_returns_none_for_missing_id() {
    let handle = SceneHandle::new();
    let mut nodes = HashMap::new();
    let mut ctx = OverlayContext::new(&mut nodes, &handle);
    assert!(ctx.node(SceneObjectId::new()).is_none());
    assert!(ctx.node_mut(SceneObjectId::new()).is_none());
}

#[test]
fn overlay_context_exposes_handle() {
    let handle = SceneHandle::new();
    let mut nodes = HashMap::new();
    let ctx = OverlayContext::new(&mut nodes, &handle);
    // Handle is accessible and returns default viewport.
    assert_eq!(ctx.handle().viewport_size(), Vec2::ZERO);
}
