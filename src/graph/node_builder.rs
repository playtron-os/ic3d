//! Builder for scene graph nodes.

use super::SceneGraph;
use crate::graph::material::MaterialId;
use crate::scene::object::SceneObjectId;
use glam::Vec3;
use std::fmt;

/// Builder returned by [`SceneGraph::add_mesh`] and [`SceneGraph::add_empty`].
///
/// Chains optional configuration before extracting the [`SceneObjectId`] via
/// [`.id()`](Self::id). The node is already inserted when the builder is
/// created — each method just mutates it in place.
///
/// ```rust,ignore
/// let cube = graph.add_mesh("cube", Mesh::cube(1.0))
///     .material(blue)
///     .position(Vec3::new(0.0, 1.0, 0.0))
///     .id();
/// ```
pub struct NodeBuilder<'a> {
    pub(super) graph: &'a mut SceneGraph,
    pub(super) id: SceneObjectId,
}

impl<'a> NodeBuilder<'a> {
    /// Set the material (mesh nodes only; no-op for empty nodes).
    #[must_use]
    pub fn material(self, material: MaterialId) -> Self {
        if let Some(node) = self.graph.nodes.get_mut(&self.id) {
            node.set_material(material);
        }
        self
    }

    /// Set the local position.
    #[must_use]
    pub fn position(self, pos: Vec3) -> Self {
        if let Some(node) = self.graph.nodes.get_mut(&self.id) {
            node.set_position(pos);
        }
        self
    }

    /// Set the local scale.
    #[must_use]
    pub fn scale(self, scale: Vec3) -> Self {
        if let Some(node) = self.graph.nodes.get_mut(&self.id) {
            node.set_scale(scale);
        }
        self
    }

    /// Set uniform scale to a single value.
    #[must_use]
    pub fn uniform_scale(self, s: f32) -> Self {
        if let Some(node) = self.graph.nodes.get_mut(&self.id) {
            node.set_uniform_scale(s);
        }
        self
    }

    /// Set the local rotation.
    #[must_use]
    pub fn rotation(self, rot: glam::Quat) -> Self {
        if let Some(node) = self.graph.nodes.get_mut(&self.id) {
            node.set_rotation(rot);
        }
        self
    }

    /// Set the parent node (moves out of root level).
    #[must_use]
    pub fn parent(self, parent: SceneObjectId) -> Self {
        self.graph.set_parent(self.id, parent);
        self
    }

    /// Set initial visibility.
    #[must_use]
    pub fn visible(self, visible: bool) -> Self {
        if let Some(node) = self.graph.nodes.get_mut(&self.id) {
            node.set_visible(visible);
        }
        self
    }

    /// Extract the [`SceneObjectId`].
    #[must_use]
    pub fn id(self) -> SceneObjectId {
        self.id
    }
}

impl fmt::Debug for NodeBuilder<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NodeBuilder").field("id", &self.id).finish()
    }
}

impl From<NodeBuilder<'_>> for SceneObjectId {
    fn from(builder: NodeBuilder<'_>) -> Self {
        builder.id
    }
}
