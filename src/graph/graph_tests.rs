use super::*;
use crate::overlay::base::{OverlayContext, OverlayEvent, OverlayInput};
use crate::scene::context::SceneHandle;
use crate::{DirectionalLight, Mesh, Overlay, PointLight, SceneContext};
use glam::{Vec2, Vec3};
use iced::Rectangle;

// ──────────── Material ────────────

#[test]
fn material_id_unique() {
    let a = MaterialId::new();
    let b = MaterialId::new();
    assert_ne!(a, b);
}

#[test]
fn material_default_grey() {
    let m = Material::default();
    assert!((m.albedo() - Vec3::new(0.8, 0.8, 0.8)).length() < 1e-6);
    assert!((m.shininess() - 32.0).abs() < 1e-6);
}

#[test]
fn material_builder() {
    let m = Material::new(Vec3::new(1.0, 0.0, 0.0))
        .with_name("red")
        .with_shininess(64.0);
    assert_eq!(m.name(), Some("red"));
    assert!((m.shininess() - 64.0).abs() < 1e-6);
}

#[test]
fn material_to_instance() {
    let m = Material::new(Vec3::new(0.1, 0.2, 0.3)).with_shininess(16.0);
    let arr = m.to_instance_material();
    assert!((arr[0] - 0.1).abs() < 1e-6);
    assert!((arr[1] - 0.2).abs() < 1e-6);
    assert!((arr[2] - 0.3).abs() < 1e-6);
    assert!((arr[3] - 16.0).abs() < 1e-6);
}

// ──────────── SceneGraph basics ────────────

#[test]
fn new_graph_has_default_material() {
    let g = SceneGraph::new();
    assert!(g.material(g.default_material()).is_some());
    assert_eq!(g.node_count(), 0);
}

#[test]
fn add_empty_node() {
    let mut g = SceneGraph::new();
    let id = g.add_empty("pivot").id();
    assert_eq!(g.node_count(), 1);
    assert_eq!(g.node(id).unwrap().name(), Some("pivot"));
    assert_eq!(g.roots(), &[id]);
}

#[test]
fn add_mesh_node() {
    let mut g = SceneGraph::new();
    let mat = g.add_material(Material::new(Vec3::ONE));
    let id = g.add_mesh("cube", Mesh::cube(1.0)).material(mat).id();
    assert_eq!(g.node_count(), 1);
    assert_eq!(g.node(id).unwrap().name(), Some("cube"));
    assert!(matches!(g.node(id).unwrap().kind(), NodeKind::Mesh { .. }));
}

#[test]
fn find_node_by_name() {
    let mut g = SceneGraph::new();
    let _a = g.add_empty("alpha").id();
    let b = g.add_empty("beta").id();
    assert_eq!(g.find_node("beta"), Some(b));
    assert_eq!(g.find_node("gamma"), None);
}

#[test]
fn find_material_by_name() {
    let mut g = SceneGraph::new();
    let id = g.add_material(Material::new(Vec3::ONE).with_name("shiny"));
    assert_eq!(g.find_material("shiny"), Some(id));
    assert_eq!(g.find_material("matte"), None);
}

// ──────────── Hierarchy ────────────

#[test]
fn set_parent_creates_hierarchy() {
    let mut g = SceneGraph::new();
    let parent = g.add_empty("parent").id();
    let child = g.add_empty("child").id();
    g.set_parent(child, parent);

    assert_eq!(g.parent(child), Some(parent));
    assert_eq!(g.children(parent), &[child]);
    // Child should no longer be a root.
    assert_eq!(g.roots(), &[parent]);
}

#[test]
fn unparent_moves_to_root() {
    let mut g = SceneGraph::new();
    let parent = g.add_empty("parent").id();
    let child = g.add_empty("child").id();
    g.set_parent(child, parent);
    g.unparent(child);

    assert_eq!(g.parent(child), None);
    assert!(g.children(parent).is_empty());
    assert!(g.roots().contains(&child));
}

#[test]
fn reparent_moves_between_parents() {
    let mut g = SceneGraph::new();
    let p1 = g.add_empty("p1").id();
    let p2 = g.add_empty("p2").id();
    let child = g.add_empty("child").id();

    g.set_parent(child, p1);
    assert_eq!(g.parent(child), Some(p1));

    g.set_parent(child, p2);
    assert_eq!(g.parent(child), Some(p2));
    assert!(g.children(p1).is_empty());
    assert_eq!(g.children(p2), &[child]);
}

#[test]
#[should_panic(expected = "cycle")]
fn set_parent_detects_cycle() {
    let mut g = SceneGraph::new();
    let a = g.add_empty("a").id();
    let b = g.add_empty("b").id();
    g.set_parent(b, a);
    g.set_parent(a, b); // Would create a→b→a cycle.
}

#[test]
#[should_panic(expected = "own parent")]
fn set_parent_self_panics() {
    let mut g = SceneGraph::new();
    let a = g.add_empty("a").id();
    g.set_parent(a, a);
}

#[test]
fn is_descendant() {
    let mut g = SceneGraph::new();
    let a = g.add_empty("a").id();
    let b = g.add_empty("b").id();
    let c = g.add_empty("c").id();
    g.set_parent(b, a);
    g.set_parent(c, b);

    assert!(g.is_descendant_of(c, a));
    assert!(g.is_descendant_of(c, b));
    assert!(!g.is_descendant_of(a, c));
    assert!(!g.is_descendant_of(a, b));
}

// ──────────── World transform ────────────

#[test]
fn world_transform_root_is_local() {
    let mut g = SceneGraph::new();
    let id = g.add_empty("root").id();
    g.node_mut(id)
        .unwrap()
        .set_position(Vec3::new(1.0, 2.0, 3.0));

    let world = g.world_transform(id);
    let pos = Vec3::new(world.col(3).x, world.col(3).y, world.col(3).z);
    assert!((pos - Vec3::new(1.0, 2.0, 3.0)).length() < 1e-6);
}

#[test]
fn world_transform_inherits_parent() {
    let mut g = SceneGraph::new();
    let parent = g.add_empty("parent").id();
    let child = g.add_empty("child").id();
    g.set_parent(child, parent);

    g.node_mut(parent)
        .unwrap()
        .set_position(Vec3::new(10.0, 0.0, 0.0));
    g.node_mut(child)
        .unwrap()
        .set_position(Vec3::new(0.0, 5.0, 0.0));

    let child_world_pos = g.world_position(child);
    assert!((child_world_pos - Vec3::new(10.0, 5.0, 0.0)).length() < 1e-6);
}

#[test]
fn world_transform_three_levels() {
    let mut g = SceneGraph::new();
    let a = g.add_empty("a").id();
    let b = g.add_empty("b").id();
    let c = g.add_empty("c").id();
    g.set_parent(b, a);
    g.set_parent(c, b);

    g.node_mut(a).unwrap().set_position(Vec3::X);
    g.node_mut(b).unwrap().set_position(Vec3::Y);
    g.node_mut(c).unwrap().set_position(Vec3::Z);

    let pos = g.world_position(c);
    assert!((pos - Vec3::new(1.0, 1.0, 1.0)).length() < 1e-6);
}

// ──────────── Remove ────────────

#[test]
fn remove_node_and_descendants() {
    let mut g = SceneGraph::new();
    let a = g.add_empty("a").id();
    let b = g.add_empty("b").id();
    let c = g.add_empty("c").id();
    g.set_parent(b, a);
    g.set_parent(c, b);

    assert!(g.remove(a));
    assert_eq!(g.node_count(), 0);
    assert!(g.roots().is_empty());
}

#[test]
fn remove_child_keeps_parent() {
    let mut g = SceneGraph::new();
    let parent = g.add_empty("parent").id();
    let child = g.add_empty("child").id();
    g.set_parent(child, parent);

    assert!(g.remove(child));
    assert_eq!(g.node_count(), 1);
    assert!(g.children(parent).is_empty());
}

#[test]
fn remove_nonexistent_returns_false() {
    let mut g = SceneGraph::new();
    assert!(!g.remove(SceneObjectId::new()));
}

// ──────────── Visibility ────────────

#[test]
fn hidden_node_skipped_in_draws() {
    let mut g = SceneGraph::new();
    let id = g.add_mesh("cube", Mesh::cube(1.0)).id();
    assert_eq!(g.to_draws().len(), 1);

    g.node_mut(id).unwrap().set_visible(false);
    assert!(g.to_draws().is_empty());
}

#[test]
fn hidden_parent_hides_children() {
    let mut g = SceneGraph::new();
    let parent = g.add_empty("parent").id();
    let _child = g.add_mesh("child", Mesh::cube(1.0)).parent(parent).id();

    assert_eq!(g.to_draws().len(), 1);

    g.node_mut(parent).unwrap().set_visible(false);
    assert!(g.to_draws().is_empty());
}

// ──────────── to_draws ────────────

#[test]
fn to_draws_produces_correct_ids() {
    let mut g = SceneGraph::new();
    let a = g.add_mesh("a", Mesh::cube(1.0)).id();
    let b = g.add_mesh("b", Mesh::cube(1.0)).id();

    let draws = g.to_draws();
    assert_eq!(draws.len(), 2);
    assert_eq!(draws[0].id, Some(a));
    assert_eq!(draws[1].id, Some(b));
}

#[test]
fn to_draws_applies_world_transform() {
    let mut g = SceneGraph::new();
    let parent = g.add_empty("parent").id();
    let _child = g.add_mesh("child", Mesh::cube(1.0)).parent(parent).id();

    g.node_mut(parent)
        .unwrap()
        .set_position(Vec3::new(3.0, 0.0, 0.0));

    let draws = g.to_draws();
    assert_eq!(draws.len(), 1);
    // Instance model matrix should have the parent offset baked in.
    let model = draws[0].instances[0].model;
    // Translation is in column 3: model[3][0..3].
    assert!((model[3][0] - 3.0).abs() < 1e-6);
}

// ──────────── Camera ────────────

#[test]
fn camera_defaults() {
    let g = SceneGraph::new();
    assert!((g.camera_position() - Vec3::new(0.0, 0.0, 5.0)).length() < 1e-6);
    assert!((g.camera_target() - Vec3::ZERO).length() < 1e-6);
}

#[test]
fn add_camera_and_activate() {
    let mut g = SceneGraph::new();
    let cam = g.add_camera(
        crate::PerspectiveCamera::new()
            .position(Vec3::new(1.0, 2.0, 3.0))
            .target(Vec3::Y),
    );
    g.set_active_camera(cam);
    assert!((g.camera_position() - Vec3::new(1.0, 2.0, 3.0)).length() < 1e-6);
    assert!((g.camera_target() - Vec3::Y).length() < 1e-6);
}

#[test]
fn camera_mut_downcast() {
    let mut g = SceneGraph::new();
    let cam = g.add_camera(crate::PerspectiveCamera::new());
    g.set_active_camera(cam);
    g.camera_mut::<crate::PerspectiveCamera>(cam)
        .unwrap()
        .set_position(Vec3::new(10.0, 0.0, 0.0));
    g.camera_mut::<crate::PerspectiveCamera>(cam)
        .unwrap()
        .set_target(Vec3::new(0.0, 5.0, 0.0));
    assert!((g.camera_position() - Vec3::new(10.0, 0.0, 0.0)).length() < 1e-6);
    assert!((g.camera_target() - Vec3::new(0.0, 5.0, 0.0)).length() < 1e-6);
}

#[test]
fn camera_wrong_type_returns_none() {
    let g = SceneGraph::new();
    let default_cam = g.active_camera_id();
    // Default is PerspectiveCamera, so downcasting to Orthographic fails.
    assert!(g.camera::<crate::OrthographicCamera>(default_cam).is_none());
}

#[test]
fn add_orthographic_camera() {
    let mut g = SceneGraph::new();
    let cam = g.add_camera(
        crate::OrthographicCamera::new()
            .position(Vec3::new(0.0, 10.0, 0.0))
            .target(Vec3::ZERO),
    );
    g.set_active_camera(cam);
    assert!((g.camera_position() - Vec3::new(0.0, 10.0, 0.0)).length() < 1e-6);
    assert!(g.camera::<crate::OrthographicCamera>(cam).is_some());
}

#[test]
fn multiple_cameras_switch() {
    let mut g = SceneGraph::new();
    let cam_a = g.add_camera(crate::PerspectiveCamera::new().position(Vec3::new(1.0, 0.0, 0.0)));
    let cam_b = g.add_camera(crate::PerspectiveCamera::new().position(Vec3::new(0.0, 0.0, 5.0)));
    g.set_active_camera(cam_a);
    assert!((g.camera_position() - Vec3::new(1.0, 0.0, 0.0)).length() < 1e-6);
    g.set_active_camera(cam_b);
    assert!((g.camera_position() - Vec3::new(0.0, 0.0, 5.0)).length() < 1e-6);
}

#[test]
fn remove_camera() {
    let mut g = SceneGraph::new();
    let default_cam = g.active_camera_id();
    let cam = g.add_camera(crate::PerspectiveCamera::new());
    // First user camera becomes active; switch back so we can remove it.
    g.set_active_camera(default_cam);
    assert!(g.remove_camera(cam));
    assert!(!g.remove_camera(cam)); // already removed
}

#[test]
#[should_panic(expected = "cannot remove the active camera")]
fn remove_active_camera_panics() {
    let mut g = SceneGraph::new();
    let cam = g.add_camera(crate::PerspectiveCamera::new());
    g.set_active_camera(cam);
    g.remove_camera(cam);
}

#[test]
fn custom_camera_type() {
    /// A minimal custom camera for testing trait-based storage.
    #[derive(Debug, Clone)]
    struct FixedCamera {
        pos: Vec3,
    }

    impl crate::Camera for FixedCamera {
        fn view_matrix(&self) -> glam::Mat4 {
            glam::Mat4::look_at_rh(self.pos, Vec3::ZERO, Vec3::Y)
        }
        fn projection_matrix(&self) -> glam::Mat4 {
            glam::Mat4::perspective_rh(1.0, 1.0, 0.1, 100.0)
        }
        fn camera_position(&self) -> Vec3 {
            self.pos
        }
    }

    let mut g = SceneGraph::new();
    let cam = g.add_camera(FixedCamera {
        pos: Vec3::new(99.0, 0.0, 0.0),
    });
    g.set_active_camera(cam);
    assert!((g.camera_position() - Vec3::new(99.0, 0.0, 0.0)).length() < 1e-6);

    // Downcast to concrete type works.
    let custom = g.camera::<FixedCamera>(cam).unwrap();
    assert!((custom.pos - Vec3::new(99.0, 0.0, 0.0)).length() < 1e-6);
}

// ──────────── Lights ────────────

#[test]
fn add_lights() {
    let mut g = SceneGraph::new();
    g.add_light(DirectionalLight::new(
        Vec3::new(-0.5, -1.0, -0.3),
        Vec3::ZERO,
        20.0,
        40.0,
    ));
    g.add_light(PointLight::new(Vec3::Y, 10.0));
    assert_eq!(g.light_count(), 2);
}

#[test]
fn add_ambient_light() {
    let mut g = SceneGraph::new();
    let id = g.add_light(AmbientLight::new(0.2));
    assert_eq!(g.light_count(), 1);
    let ambient = g.light::<AmbientLight>(id).unwrap();
    assert!((ambient.level() - 0.2).abs() < 1e-6);
}

#[test]
fn light_mut_downcast() {
    let mut g = SceneGraph::new();
    let id = g.add_light(AmbientLight::new(0.1));
    g.light_mut::<AmbientLight>(id).unwrap().set_level(0.5);
    assert!((g.light::<AmbientLight>(id).unwrap().level() - 0.5).abs() < 1e-6);
}

#[test]
fn remove_light() {
    let mut g = SceneGraph::new();
    let id = g.add_light(PointLight::new(Vec3::ZERO, 5.0));
    assert!(g.remove_light(id));
    assert!(!g.remove_light(id));
    assert_eq!(g.light_count(), 0);
}

#[test]
fn ambient_summed_in_setup() {
    let mut g = SceneGraph::new();
    g.add_light(AmbientLight::new(0.1));
    g.add_light(AmbientLight::new(0.2));
    let bounds = Rectangle {
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
    };
    let setup = g.to_setup(bounds);
    assert!((setup.scene.uniforms.ambient - 0.3).abs() < 1e-6);
}

// ──────────── to_setup ────────────

#[test]
fn to_setup_produces_valid_scene() {
    let mut g = SceneGraph::new();
    let mat = g.add_material(Material::new(Vec3::new(0.2, 0.6, 0.9)));
    let _ = g.add_mesh("cube", Mesh::cube(1.0)).material(mat).id();
    g.add_light(DirectionalLight::new(Vec3::NEG_Y, Vec3::ZERO, 20.0, 40.0));

    let bounds = Rectangle {
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
    };
    let setup = g.to_setup(bounds);
    assert_eq!(setup.draws.len(), 1);
    assert_eq!(setup.scene.lights.len(), 1);
}

// ──────────── Debug ────────────

#[test]
fn debug_format() {
    let g = SceneGraph::new();
    let dbg = format!("{g:?}");
    assert!(dbg.contains("SceneGraph"));
    assert!(dbg.contains("node_count"));
}

// ──────────── Node accessors ────────────

#[test]
fn node_set_uniform_scale() {
    let mut g = SceneGraph::new();
    let id = g.add_empty("test").id();
    g.node_mut(id).unwrap().set_uniform_scale(2.0);
    let scale = g.node(id).unwrap().local_transform().scale;
    assert!((scale - Vec3::splat(2.0)).length() < 1e-6);
}

// ──────────── process_input ────────────

/// Test overlay that scales its target node when mouse is pressed.
#[derive(Debug, Clone)]
struct InputTestOverlay {
    target: SceneObjectId,
    called: bool,
}

impl Overlay for InputTestOverlay {
    fn draw(&self, _ctx: &SceneContext) -> Vec<crate::widget::MeshDrawGroup> {
        Vec::new()
    }

    fn on_input(&mut self, input: &OverlayInput, ctx: &mut OverlayContext) -> Vec<OverlayEvent> {
        self.called = true;
        if input.mouse_pressed {
            if let Some(node) = ctx.node_mut(self.target) {
                node.add_uniform_scale(1.0);
            }
        }
        Vec::new()
    }
}

#[test]
fn process_input_calls_overlay_on_input() {
    let mut g = SceneGraph::new();
    let cube = g.add_mesh("cube", Mesh::cube(1.0)).id();
    let overlay = InputTestOverlay {
        target: cube,
        called: false,
    };
    let oid = g.add_overlay(overlay);
    let handle = SceneHandle::new();

    g.process_input(&handle);

    assert!(g.overlay::<InputTestOverlay>(oid).unwrap().called);
}

#[test]
fn process_input_overlay_mutates_node() {
    let mut g = SceneGraph::new();
    let cube = g.add_mesh("cube", Mesh::cube(1.0)).id();
    let overlay = InputTestOverlay {
        target: cube,
        called: false,
    };
    g.add_overlay(overlay);
    let handle = SceneHandle::new();
    handle.update_input(OverlayInput {
        cursor: Vec2::ZERO,
        mouse_pressed: true,
    });

    g.process_input(&handle);

    let scale = g.node(cube).unwrap().uniform_scale();
    assert!((scale - 2.0).abs() < 1e-6);
}

#[test]
fn process_input_with_no_overlays() {
    let mut g = SceneGraph::new();
    let _ = g.add_mesh("cube", Mesh::cube(1.0)).id();
    let handle = SceneHandle::new();

    // Should not panic.
    g.process_input(&handle);
}

/// Test overlay that emits a Custom event on mouse press.
#[derive(Debug, Clone)]
struct EventTestOverlay {
    _target: SceneObjectId,
}

impl Overlay for EventTestOverlay {
    fn draw(&self, _ctx: &SceneContext) -> Vec<crate::widget::MeshDrawGroup> {
        Vec::new()
    }

    fn on_input(&mut self, input: &OverlayInput, _ctx: &mut OverlayContext) -> Vec<OverlayEvent> {
        if input.mouse_pressed {
            vec![OverlayEvent::Custom("clicked".into())]
        } else {
            Vec::new()
        }
    }
}

#[test]
fn process_input_collects_events() {
    let mut g = SceneGraph::new();
    let cube = g.add_mesh("cube", Mesh::cube(1.0)).id();
    let oid = g.add_overlay(EventTestOverlay { _target: cube });
    let handle = SceneHandle::new();
    handle.update_input(OverlayInput {
        cursor: Vec2::ZERO,
        mouse_pressed: true,
    });

    let events = g.process_input(&handle);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].0, oid);
    assert!(matches!(events[0].1, OverlayEvent::Custom(ref s) if s == "clicked"));
}

#[test]
fn process_input_no_events_when_idle() {
    let mut g = SceneGraph::new();
    let cube = g.add_mesh("cube", Mesh::cube(1.0)).id();
    g.add_overlay(EventTestOverlay { _target: cube });
    let handle = SceneHandle::new();

    let events = g.process_input(&handle);
    assert!(events.is_empty());
}

// ──────────── interactive ────────────

/// Non-interactive overlay: visible but should not receive on_input.
#[derive(Debug, Clone)]
struct NonInteractiveOverlay {
    called: bool,
}

impl Overlay for NonInteractiveOverlay {
    fn interactive(&self) -> bool {
        false
    }

    fn draw(&self, _ctx: &SceneContext) -> Vec<crate::widget::MeshDrawGroup> {
        Vec::new()
    }

    fn on_input(&mut self, _input: &OverlayInput, _ctx: &mut OverlayContext) -> Vec<OverlayEvent> {
        self.called = true;
        vec![OverlayEvent::Custom("should not happen".into())]
    }
}

#[test]
fn process_input_skips_non_interactive_overlay() {
    let mut g = SceneGraph::new();
    let _ = g.add_mesh("cube", Mesh::cube(1.0)).id();
    let oid = g.add_overlay(NonInteractiveOverlay { called: false });
    let handle = SceneHandle::new();
    handle.update_input(OverlayInput {
        cursor: Vec2::ZERO,
        mouse_pressed: true,
    });

    let events = g.process_input(&handle);
    assert!(
        events.is_empty(),
        "non-interactive overlay should produce no events"
    );
    assert!(
        !g.overlay::<NonInteractiveOverlay>(oid).unwrap().called,
        "on_input should not be called for non-interactive overlay"
    );
}
