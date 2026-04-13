//! Retained-mode scene graph with parent-child hierarchy.
//!
//! [`SceneGraph`] stores nodes (meshes, empty pivots), materials, camera,
//! and lights in a tree structure. It bridges to ic3d's immediate-mode
//! rendering via [`to_setup`](SceneGraph::to_setup) or by implementing
//! [`Scene3DProgram`](crate::widget::Scene3DProgram) directly.
//!
//! ```rust,ignore
//! use ic3d::graph::{SceneGraph, Material};
//! use ic3d::glam::Vec3;
//! use ic3d::Mesh;
//!
//! let mut graph = SceneGraph::new();
//!
//! // Materials
//! let blue = graph.add_material(Material::new(Vec3::new(0.2, 0.6, 0.9)));
//!
//! // Nodes with hierarchy
//! let body = graph.add_mesh("body", Mesh::cube(1.0)).material(blue).id();
//! let arm = graph.add_mesh("arm", Mesh::cylinder(0.1, 1.0, 8))
//!     .material(blue)
//!     .parent(body)
//!     .position(Vec3::new(0.5, 0.0, 0.0))
//!     .id();
//!
//! // Camera and lights
//! let cam = graph.add_camera(ic3d::PerspectiveCamera::new()
//!     .position(Vec3::new(5.0, 5.0, 8.0))
//!     .target(Vec3::ZERO)
//!     .clip(0.1, 50.0));
//! graph.set_active_camera(cam);
//! graph.add_light(ic3d::DirectionalLight::new(
//!     Vec3::new(-0.5, -1.0, -0.3), Vec3::ZERO, 20.0, 40.0));
//! graph.add_light(ic3d::graph::AmbientLight::new(0.15));
//!
//! // In view() — graph implements Scene3DProgram
//! scene_3d(&graph).scene(handle.clone()).into()
//! ```
//!
//! # Scene loading
//!
//! The graph is designed to support loading scenes from other engines:
//! ```rust,ignore
//! // Parse glTF/FBX/.babylon → walk nodes
//! let root = graph.add_empty("root").id();
//! for mesh_data in parsed_meshes {
//!     let mat = graph.add_material(Material::new(mesh_data.color));
//!     graph.add_mesh(&mesh_data.name, mesh_data.mesh)
//!         .material(mat)
//!         .parent(root)
//!         .position(mesh_data.position);
//! }
//! ```

pub mod material;
pub mod node;
mod node_builder;
mod rendering;
pub mod scene_camera;
pub mod scene_light;

pub use material::{Material, MaterialId};
pub use node::{Node, NodeKind};
pub use node_builder::NodeBuilder;
pub use scene_camera::{CameraId, SceneCamera};
pub use scene_light::{AmbientLight, LightId, SceneLight};

use crate::camera::Camera;
use crate::mesh::Mesh;
use crate::overlay::base::{Overlay, OverlayContext, OverlayEvent};
use crate::scene::context::SceneHandle;
use crate::scene::object::SceneObjectId;
use crate::PerspectiveCamera;
use glam::{Mat4, Vec3};
use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Instant;

/// Unique identifier for an overlay in the scene graph.
///
/// Created automatically by [`SceneGraph::add_overlay`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OverlayId(u32);

impl OverlayId {
    /// Create a new unique overlay ID.
    #[must_use]
    pub fn new() -> Self {
        static NEXT: AtomicU32 = AtomicU32::new(1);
        Self(NEXT.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for OverlayId {
    fn default() -> Self {
        Self::new()
    }
}

/// Retained-mode scene graph with parent-child transform hierarchy.
///
/// Stores nodes, materials, camera config, and lights. Generates
/// [`Scene3DSetup`](crate::widget::Scene3DSetup) each frame via
/// [`to_setup`](Self::to_setup) or by plugging directly into
/// [`scene_3d()`](crate::widget::scene_3d) as a
/// [`Scene3DProgram`](crate::widget::Scene3DProgram).
///
/// # Hierarchy
///
/// Nodes form a tree. Each node has a local transform relative to its parent.
/// The world transform is computed by multiplying the chain of local
/// transforms from root to leaf: `world = parent_world * local`.
///
/// # Named lookup
///
/// Nodes and materials can have optional string names for BabylonJS-style
/// `find_node("arm")` queries.
#[derive(Clone)]
pub struct SceneGraph {
    // ── Node hierarchy ──
    pub(crate) nodes: HashMap<SceneObjectId, Node>,
    pub(crate) children: HashMap<SceneObjectId, Vec<SceneObjectId>>,
    pub(crate) parents: HashMap<SceneObjectId, SceneObjectId>,
    pub(crate) roots: Vec<SceneObjectId>,

    // ── Materials ──
    pub(crate) materials: HashMap<MaterialId, Material>,
    pub(crate) default_material: MaterialId,

    // ── Cameras ──
    pub(crate) cameras: HashMap<CameraId, Box<dyn SceneCamera>>,
    pub(crate) active_camera: CameraId,

    // ── Lights ──
    pub(crate) lights: HashMap<LightId, Box<dyn SceneLight>>,

    // ── Overlays ──
    pub(crate) overlays: HashMap<OverlayId, Box<dyn CloneableOverlay>>,

    // ── Time ──
    pub(crate) last_tick: Option<Instant>,
    pub(crate) elapsed: f32,
}

// ── Clonable overlay wrapper (internal) ──

/// Internal trait adding `Clone` support to [`Overlay`] for storage in
/// `SceneGraph`.  Any `T: Overlay + Clone + 'static` satisfies this
/// automatically via the blanket impl.
pub(crate) trait CloneableOverlay: Overlay {
    fn clone_overlay(&self) -> Box<dyn CloneableOverlay>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Overlay + Clone + 'static> CloneableOverlay for T {
    fn clone_overlay(&self) -> Box<dyn CloneableOverlay> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Clone for Box<dyn CloneableOverlay> {
    fn clone(&self) -> Self {
        self.clone_overlay()
    }
}

impl SceneGraph {
    /// Create an empty scene graph with default camera and a default grey material.
    #[must_use]
    pub fn new() -> Self {
        let default_material = MaterialId::new();
        let mut materials = HashMap::new();
        materials.insert(default_material, Material::default());

        let default_camera_id = CameraId::new();
        let mut cameras: HashMap<CameraId, Box<dyn SceneCamera>> = HashMap::new();
        cameras.insert(default_camera_id, Box::new(PerspectiveCamera::new()));

        Self {
            nodes: HashMap::new(),
            children: HashMap::new(),
            parents: HashMap::new(),
            roots: Vec::new(),
            materials,
            default_material,
            cameras,
            active_camera: default_camera_id,
            lights: HashMap::new(),
            overlays: HashMap::new(),
            last_tick: None,
            elapsed: 0.0,
        }
    }

    // ──────────── Time ────────────

    /// Advance the scene clock and return delta time in seconds.
    ///
    /// Call once per frame in your `update()`. The first call returns `0.0`.
    /// The internal elapsed time (accessible via [`elapsed`](Self::elapsed))
    /// is also updated and automatically passed to the shader's `time`
    /// uniform each frame.
    ///
    /// ```rust,ignore
    /// fn update(&mut self, message: Message) {
    ///     match message {
    ///         Message::Tick => {
    ///             let dt = self.graph.tick();
    ///             // animate using dt and self.graph.elapsed()
    ///         }
    ///     }
    /// }
    /// ```
    pub fn tick(&mut self) -> f32 {
        let now = Instant::now();
        let dt = self
            .last_tick
            .map_or(0.0, |last| now.duration_since(last).as_secs_f32());
        self.last_tick = Some(now);
        self.elapsed += dt;
        dt
    }

    /// Total elapsed time in seconds since the first [`tick`](Self::tick) call.
    #[must_use]
    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    // ──────────── Materials ────────────

    /// Add a material and return its ID.
    pub fn add_material(&mut self, material: Material) -> MaterialId {
        let id = MaterialId::new();
        self.materials.insert(id, material);
        id
    }

    /// Get a material by ID.
    #[must_use]
    pub fn material(&self, id: MaterialId) -> Option<&Material> {
        self.materials.get(&id)
    }

    /// Get a mutable reference to a material.
    pub fn material_mut(&mut self, id: MaterialId) -> Option<&mut Material> {
        self.materials.get_mut(&id)
    }

    /// The default material ID (grey, shininess 32).
    #[must_use]
    pub fn default_material(&self) -> MaterialId {
        self.default_material
    }

    /// Find a material by name. Returns the first match.
    #[must_use]
    pub fn find_material(&self, name: &str) -> Option<MaterialId> {
        self.materials
            .iter()
            .find(|(_, m)| m.name() == Some(name))
            .map(|(&id, _)| id)
    }

    // ──────────── Nodes ────────────

    /// Add an empty grouping/pivot node at the root level.
    ///
    /// Returns a [`NodeBuilder`] for chaining position, parent, etc.
    /// Call [`.id()`](NodeBuilder::id) to get the [`SceneObjectId`].
    ///
    /// ```rust,ignore
    /// let pivot = graph.add_empty("pivot")
    ///     .position(Vec3::new(0.0, 2.0, 0.0))
    ///     .id();
    /// ```
    pub fn add_empty(&mut self, name: &str) -> NodeBuilder<'_> {
        let id = SceneObjectId::new();
        let mut node = Node::new(id, NodeKind::Empty);
        node.set_name(name);
        self.nodes.insert(id, node);
        self.roots.push(id);
        NodeBuilder { graph: self, id }
    }

    /// Add a mesh node at the root level using the default material.
    ///
    /// Returns a [`NodeBuilder`] for chaining material, position, scale, etc.
    /// Call [`.id()`](NodeBuilder::id) to get the [`SceneObjectId`].
    ///
    /// ```rust,ignore
    /// let ground = graph.add_mesh("ground", Mesh::plane(20.0, 20.0))
    ///     .material(ground_mat)
    ///     .position(Vec3::new(0.0, -0.01, 0.0))
    ///     .id();
    ///
    /// // Without material — uses default grey
    /// let cube = graph.add_mesh("cube", Mesh::cube(1.0))
    ///     .position(Vec3::new(0.0, 1.0, 0.0))
    ///     .id();
    /// ```
    pub fn add_mesh(&mut self, name: &str, mesh: Mesh) -> NodeBuilder<'_> {
        let id = SceneObjectId::new();
        let material = self.default_material;
        let mut node = Node::new(id, NodeKind::Mesh { mesh, material });
        node.set_name(name);
        self.nodes.insert(id, node);
        self.roots.push(id);
        NodeBuilder { graph: self, id }
    }

    /// Get a node by ID.
    #[must_use]
    pub fn node(&self, id: SceneObjectId) -> Option<&Node> {
        self.nodes.get(&id)
    }

    /// Get a mutable reference to a node for updating transform, visibility, etc.
    pub fn node_mut(&mut self, id: SceneObjectId) -> Option<&mut Node> {
        self.nodes.get_mut(&id)
    }

    /// Find a node by name. Returns the first match.
    #[must_use]
    pub fn find_node(&self, name: &str) -> Option<SceneObjectId> {
        self.nodes
            .iter()
            .find(|(_, n)| n.name() == Some(name))
            .map(|(&id, _)| id)
    }

    /// Remove a node and all its descendants from the graph.
    ///
    /// Returns `true` if the node existed.
    pub fn remove(&mut self, id: SceneObjectId) -> bool {
        if !self.nodes.contains_key(&id) {
            return false;
        }

        // Save parent before we start removing.
        let parent_id = self.parents.get(&id).copied();

        // Collect all descendants first.
        let mut to_remove = Vec::new();
        self.collect_descendants(id, &mut to_remove);
        to_remove.push(id);

        for &nid in &to_remove {
            self.nodes.remove(&nid);
            self.children.remove(&nid);
            self.parents.remove(&nid);
        }

        // Remove from parent's children list or roots.
        if let Some(pid) = parent_id {
            if let Some(siblings) = self.children.get_mut(&pid) {
                siblings.retain(|&c| c != id);
            }
        }
        self.roots.retain(|&r| r != id);

        true
    }

    fn collect_descendants(&self, id: SceneObjectId, out: &mut Vec<SceneObjectId>) {
        if let Some(kids) = self.children.get(&id) {
            for &kid in kids {
                out.push(kid);
                self.collect_descendants(kid, out);
            }
        }
    }

    /// The number of nodes in the graph.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// All root-level node IDs.
    #[must_use]
    pub fn roots(&self) -> &[SceneObjectId] {
        &self.roots
    }

    /// The children of a node.
    #[must_use]
    pub fn children(&self, id: SceneObjectId) -> &[SceneObjectId] {
        self.children.get(&id).map_or(&[], |v| v.as_slice())
    }

    /// The parent of a node, if any.
    #[must_use]
    pub fn parent(&self, id: SceneObjectId) -> Option<SceneObjectId> {
        self.parents.get(&id).copied()
    }

    // ──────────── Hierarchy ────────────

    /// Set the parent of a node. Moves it from root/previous parent.
    ///
    /// # Panics
    ///
    /// Panics if `child == parent` or if either ID doesn't exist.
    pub fn set_parent(&mut self, child: SceneObjectId, parent: SceneObjectId) {
        assert_ne!(child, parent, "ic3d: node cannot be its own parent");
        assert!(
            self.nodes.contains_key(&child),
            "ic3d: child node not found"
        );
        assert!(
            self.nodes.contains_key(&parent),
            "ic3d: parent node not found"
        );

        // Check for cycles: parent must not be a descendant of child.
        assert!(
            !self.is_descendant_of(parent, child),
            "ic3d: setting parent would create a cycle"
        );

        // Remove from old parent's children or roots.
        if let Some(&old_parent) = self.parents.get(&child) {
            if let Some(siblings) = self.children.get_mut(&old_parent) {
                siblings.retain(|&c| c != child);
            }
        }
        self.roots.retain(|&r| r != child);

        // Set new parent.
        self.parents.insert(child, parent);
        self.children.entry(parent).or_default().push(child);
    }

    /// Remove the parent of a node, making it a root node.
    pub fn unparent(&mut self, child: SceneObjectId) {
        if let Some(old_parent) = self.parents.remove(&child) {
            if let Some(siblings) = self.children.get_mut(&old_parent) {
                siblings.retain(|&c| c != child);
            }
            self.roots.push(child);
        }
    }

    /// Check if `node` is a descendant of `ancestor`.
    #[must_use]
    pub fn is_descendant_of(&self, node: SceneObjectId, ancestor: SceneObjectId) -> bool {
        let mut current = node;
        while let Some(&parent) = self.parents.get(&current) {
            if parent == ancestor {
                return true;
            }
            current = parent;
        }
        false
    }

    /// Compute the world transform of a node (walks up the parent chain).
    #[must_use]
    pub fn world_transform(&self, id: SceneObjectId) -> Mat4 {
        let Some(node) = self.nodes.get(&id) else {
            return Mat4::IDENTITY;
        };
        let local = node.local_transform().matrix();
        match self.parents.get(&id) {
            Some(&parent_id) => self.world_transform(parent_id) * local,
            None => local,
        }
    }

    /// The world-space position of a node (translation column of world transform).
    #[must_use]
    pub fn world_position(&self, id: SceneObjectId) -> Vec3 {
        let m = self.world_transform(id);
        Vec3::new(m.col(3).x, m.col(3).y, m.col(3).z)
    }

    // ──────────── Camera ────────────

    /// Add a camera and return its [`CameraId`].
    ///
    /// Any type implementing `Camera + Clone + Debug + Send + Sync + 'static`
    /// is accepted — including [`PerspectiveCamera`],
    /// [`OrthographicCamera`](crate::OrthographicCamera),
    /// or your own custom camera type.
    ///
    /// The first camera added becomes the active camera for rendering.
    ///
    /// ```rust,ignore
    /// let cam = graph.add_camera(PerspectiveCamera::new()
    ///     .position(Vec3::new(5.0, 5.0, 8.0))
    ///     .target(Vec3::new(0.0, 1.0, 0.0))
    ///     .clip(0.1, 50.0));
    /// ```
    pub fn add_camera(&mut self, camera: impl SceneCamera + 'static) -> CameraId {
        let id = CameraId::new();
        // Activate the first user-added camera (replaces the default).
        if self.cameras.len() == 1 {
            self.active_camera = id;
        }
        self.cameras.insert(id, Box::new(camera));
        id
    }

    /// Set the active camera for rendering.
    ///
    /// # Panics
    ///
    /// Panics if the camera ID doesn't exist.
    pub fn set_active_camera(&mut self, id: CameraId) {
        assert!(self.cameras.contains_key(&id), "ic3d: camera ID not found");
        self.active_camera = id;
    }

    /// The active camera ID.
    #[must_use]
    pub fn active_camera_id(&self) -> CameraId {
        self.active_camera
    }

    /// Get the active camera as a trait object.
    #[must_use]
    pub fn active_camera(&self) -> &dyn Camera {
        self.cameras[&self.active_camera].as_ref()
    }

    /// Get a camera by ID, downcasting to the concrete type.
    ///
    /// Returns `None` if the ID doesn't exist or the type is wrong.
    ///
    /// ```rust,ignore
    /// if let Some(cam) = graph.camera::<PerspectiveCamera>(cam_id) {
    ///     let pos = cam.camera_position();
    /// }
    /// ```
    #[must_use]
    pub fn camera<T: SceneCamera + 'static>(&self, id: CameraId) -> Option<&T> {
        self.cameras.get(&id)?.as_any().downcast_ref()
    }

    /// Get a mutable reference to a camera by ID, downcasting to the concrete type.
    ///
    /// Returns `None` if the ID doesn't exist or the type is wrong.
    ///
    /// ```rust,ignore
    /// graph.camera_mut::<PerspectiveCamera>(cam_id)
    ///     .unwrap()
    ///     .set_position(Vec3::new(10.0, 5.0, 8.0));
    /// ```
    pub fn camera_mut<T: SceneCamera + 'static>(&mut self, id: CameraId) -> Option<&mut T> {
        self.cameras.get_mut(&id)?.as_any_mut().downcast_mut()
    }

    /// Remove a camera. Cannot remove the active camera.
    ///
    /// Returns `true` if the camera existed and was removed.
    ///
    /// # Panics
    ///
    /// Panics if trying to remove the active camera.
    pub fn remove_camera(&mut self, id: CameraId) -> bool {
        assert_ne!(
            id, self.active_camera,
            "ic3d: cannot remove the active camera"
        );
        self.cameras.remove(&id).is_some()
    }

    /// Current active camera position (convenience getter).
    #[must_use]
    pub fn camera_position(&self) -> Vec3 {
        self.active_camera().camera_position()
    }

    /// Current active camera target (convenience getter).
    #[must_use]
    pub fn camera_target(&self) -> Vec3 {
        self.active_camera().camera_target()
    }

    // ──────────── Lights ────────────

    /// Add a light and return its [`LightId`].
    ///
    /// Any type implementing `Light + Clone + Debug + Send + Sync + 'static`
    /// is accepted — including [`DirectionalLight`](crate::DirectionalLight),
    /// [`PointLight`](crate::PointLight), [`SpotLight`](crate::SpotLight),
    /// [`AmbientLight`], or your own custom light type.
    ///
    /// ```rust,ignore
    /// let sun = graph.add_light(DirectionalLight::new(...));
    /// let ambient = graph.add_light(AmbientLight::new(0.15));
    /// ```
    pub fn add_light(&mut self, light: impl SceneLight + 'static) -> LightId {
        let id = LightId::new();
        self.lights.insert(id, Box::new(light));
        id
    }

    /// Get a light by ID, downcasting to the concrete type.
    ///
    /// Returns `None` if the ID doesn't exist or the type is wrong.
    #[must_use]
    pub fn light<T: SceneLight + 'static>(&self, id: LightId) -> Option<&T> {
        self.lights.get(&id)?.as_any().downcast_ref()
    }

    /// Get a mutable reference to a light by ID, downcasting to the concrete type.
    ///
    /// Returns `None` if the ID doesn't exist or the type is wrong.
    ///
    /// ```rust,ignore
    /// graph.light_mut::<DirectionalLight>(sun_id)
    ///     .unwrap()
    ///     .set_intensity(2.0);
    /// ```
    pub fn light_mut<T: SceneLight + 'static>(&mut self, id: LightId) -> Option<&mut T> {
        self.lights.get_mut(&id)?.as_any_mut().downcast_mut()
    }

    /// Remove a light. Returns `true` if the light existed.
    pub fn remove_light(&mut self, id: LightId) -> bool {
        self.lights.remove(&id).is_some()
    }

    /// The number of lights in the scene.
    #[must_use]
    pub fn light_count(&self) -> usize {
        self.lights.len()
    }

    // ──────────── Overlays ────────────

    /// Register an overlay to be rendered on top of the scene.
    ///
    /// Returns an [`OverlayId`] for later access via
    /// [`overlay_mut`](Self::overlay_mut).
    ///
    /// ```rust,ignore
    /// let gizmo_id = graph.add_overlay(my_gizmo);
    /// graph.overlay_mut::<ScaleGizmo>(gizmo_id).unwrap().set_target(Some(cube));
    /// ```
    pub fn add_overlay(&mut self, overlay: impl Overlay + Clone + 'static) -> OverlayId {
        let id = OverlayId::new();
        self.overlays.insert(id, Box::new(overlay));
        id
    }

    /// Remove a specific overlay by ID.
    ///
    /// Returns `true` if the overlay existed.
    pub fn remove_overlay(&mut self, id: OverlayId) -> bool {
        self.overlays.remove(&id).is_some()
    }

    /// Remove all registered overlays.
    pub fn clear_overlays(&mut self) {
        self.overlays.clear();
    }

    /// Number of registered overlays.
    #[must_use]
    pub fn overlay_count(&self) -> usize {
        self.overlays.len()
    }

    /// Downcast an overlay by ID (immutable).
    ///
    /// Returns `None` if the ID doesn't exist or the type doesn't match.
    #[must_use]
    pub fn overlay<T: Overlay + 'static>(&self, id: OverlayId) -> Option<&T> {
        self.overlays.get(&id)?.as_any().downcast_ref::<T>()
    }

    /// Downcast an overlay by ID (mutable).
    ///
    /// Allows mutating overlay state (hover, drag, target) in place:
    ///
    /// ```rust,ignore
    /// graph.overlay_mut::<ScaleGizmo>(gizmo_id).unwrap().set_target(Some(id));
    /// ```
    pub fn overlay_mut<T: Overlay + 'static>(&mut self, id: OverlayId) -> Option<&mut T> {
        self.overlays.get_mut(&id)?.as_any_mut().downcast_mut::<T>()
    }

    /// Process input for all registered overlays.
    ///
    /// Each overlay's [`on_input`](Overlay::on_input) is called with the
    /// current input state and mutable access to scene nodes, allowing
    /// overlays to apply their own mutations (scaling, translating, etc.)
    /// directly.
    ///
    /// The input (cursor position, mouse pressed) is read from the
    /// [`SceneHandle`], which the widget populates automatically each
    /// event. No manual cursor/mouse tracking needed.
    ///
    /// Returns events emitted by overlays this frame, keyed by
    /// [`OverlayId`]. Use these to react to hover/drag lifecycle:
    ///
    /// ```rust,ignore
    /// for (id, event) in self.graph.process_input(&self.handle) {
    ///     match event {
    ///         OverlayEvent::DragEnd => { /* commit undo */ }
    ///         _ => {}
    ///     }
    /// }
    /// ```
    pub fn process_input(&mut self, handle: &SceneHandle) -> Vec<(OverlayId, OverlayEvent)> {
        let input = handle.input();
        let mut overlays = std::mem::take(&mut self.overlays);
        let mut all_events = Vec::new();
        for (&id, overlay) in &mut overlays {
            if !overlay.visible() || !overlay.interactive() {
                continue;
            }
            let mut ctx = OverlayContext::new(&mut self.nodes, handle);
            let events = overlay.on_input(&input, &mut ctx);
            all_events.extend(events.into_iter().map(|e| (id, e)));
        }
        self.overlays = overlays;
        all_events
    }
}

impl Default for SceneGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for SceneGraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SceneGraph")
            .field("node_count", &self.nodes.len())
            .field("material_count", &self.materials.len())
            .field("camera_count", &self.cameras.len())
            .field("light_count", &self.lights.len())
            .field("overlay_count", &self.overlays.len())
            .field("root_count", &self.roots.len())
            .field("elapsed", &self.elapsed)
            .finish()
    }
}

#[cfg(test)]
#[path = "graph_tests.rs"]
mod tests;
