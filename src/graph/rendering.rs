//! Scene graph → render pipeline bridge.
//!
//! Converts the retained-mode [`SceneGraph`] into immediate-mode
//! [`Scene3DSetup`] data for the render pipeline.

use super::scene_camera::SceneCamera;
use super::SceneGraph;
use crate::graph::node::NodeKind;
use crate::overlay::base::Overlay;
use crate::pipeline::gpu_types::InstanceData;
use crate::scene::builder::Scene;
use crate::scene::object::SceneObjectId;
use crate::scene::transform::Transform;
use crate::widget::{MeshDrawGroup, Scene3DProgram, Scene3DSetup};
use glam::Mat4;
use iced::Rectangle;

impl SceneGraph {
    /// Generate [`MeshDrawGroup`]s from the scene graph.
    ///
    /// Traverses all visible nodes, computes world transforms via the
    /// parent-child hierarchy, and produces draw groups for rendering.
    /// Each mesh node generates one draw group tagged with its
    /// [`SceneObjectId`] so gizmos and overlays can attach to it.
    #[must_use]
    pub fn to_draws(&self) -> Vec<MeshDrawGroup> {
        let mut draws = Vec::new();
        for &root_id in &self.roots {
            self.collect_draws(root_id, Mat4::IDENTITY, &mut draws);
        }
        draws
    }

    fn collect_draws(&self, id: SceneObjectId, parent_world: Mat4, draws: &mut Vec<MeshDrawGroup>) {
        let Some(node) = self.nodes.get(&id) else {
            return;
        };
        if !node.visible() {
            return;
        }

        let world = parent_world * node.local_transform().matrix();

        if let NodeKind::Mesh { ref mesh, material } = *node.kind() {
            let material_data = self
                .materials
                .get(&material)
                .unwrap_or_else(|| self.materials.get(&self.default_material).unwrap());

            let normal = Transform::new().normal_matrix();
            let world_normal = {
                let m3 = glam::Mat3::from_cols(
                    world.col(0).truncate(),
                    world.col(1).truncate(),
                    world.col(2).truncate(),
                );
                let inv = m3.inverse().transpose();
                [
                    inv.x_axis.to_array(),
                    inv.y_axis.to_array(),
                    inv.z_axis.to_array(),
                ]
            };
            let _ = normal; // Use computed world normal instead.

            let instance = InstanceData {
                model: world.to_cols_array_2d(),
                normal_mat: world_normal,
                _pad: [0.0; 3],
                material: material_data.to_instance_material(),
            };

            draws.push(MeshDrawGroup::new(mesh.clone(), vec![instance]).with_id(id));
        }

        // Recurse into children.
        if let Some(kids) = self.children.get(&id) {
            for &kid in kids {
                self.collect_draws(kid, world, draws);
            }
        }
    }

    /// Build a camera ready for rendering at the given aspect ratio.
    fn build_camera_for_viewport(&self, aspect: f32) -> Box<dyn SceneCamera> {
        let mut camera = self.cameras[&self.active_camera].clone_camera();
        camera.set_aspect(aspect);
        camera
    }

    /// Generate a complete [`Scene3DSetup`] for rendering this frame.
    ///
    /// This is the bridge between the retained-mode scene graph and ic3d's
    /// immediate-mode render pipeline. Call from [`Scene3DProgram::setup`] or
    /// use the built-in `Scene3DProgram` impl directly.
    ///
    /// - `bounds`: viewport rectangle (used for camera aspect ratio)
    #[must_use]
    pub fn to_setup(&self, bounds: Rectangle) -> Scene3DSetup {
        let aspect = bounds.width / bounds.height.max(1.0);
        let camera = self.build_camera_for_viewport(aspect);

        let mut scene = Scene::new(&*camera).screen_size(bounds.width, bounds.height);

        // Collect ambient level from all ambient lights, and add
        // non-ambient lights to the GPU light array.
        let mut ambient = 0.0_f32;
        for light in self.lights.values() {
            if let Some(level) = light.ambient_level() {
                ambient += level;
            }
            if let Some(gpu) = light.to_gpu_light() {
                scene = scene.gpu_light(gpu);
            }
        }
        scene = scene.ambient(ambient.min(1.0));
        scene = scene.time(self.elapsed);

        Scene3DSetup {
            scene: scene.build(),
            draws: self.to_draws(),
            overlays: self
                .overlays
                .values()
                .map(|o| -> Box<dyn Overlay> { o.clone_overlay() })
                .collect(),
            custom_uniforms: None,
        }
    }
}

impl Scene3DProgram for SceneGraph {
    fn setup(&self, bounds: Rectangle) -> Scene3DSetup {
        self.to_setup(bounds)
    }
}
