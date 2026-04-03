//! TRS transform → model + normal matrices for instanced rendering.

use glam::{Mat3, Mat4, Quat, Vec3};

use crate::gpu_types::InstanceData;

/// TRS transform. `Model = Translate × Rotate × Scale`.
pub struct Transform {
    /// World-space position.
    pub position: Vec3,
    /// Rotation quaternion.
    pub rotation: Quat,
    /// Non-uniform scale.
    pub scale: Vec3,
}

impl Transform {
    /// Identity transform (origin, no rotation, unit scale).
    #[must_use]
    pub fn new() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    #[must_use]
    pub fn position(mut self, pos: Vec3) -> Self {
        self.position = pos;
        self
    }

    #[must_use]
    pub fn rotation(mut self, rot: Quat) -> Self {
        self.rotation = rot;
        self
    }

    #[must_use]
    pub fn scale(mut self, scale: Vec3) -> Self {
        self.scale = scale;
        self
    }

    #[must_use]
    pub fn uniform_scale(mut self, s: f32) -> Self {
        self.scale = Vec3::splat(s);
        self
    }

    /// 4×4 model matrix: `Translate × Rotate × Scale`.
    #[must_use]
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_translation(self.position)
            * Mat4::from_quat(self.rotation)
            * Mat4::from_scale(self.scale)
    }

    /// 3×3 normal matrix (inverse-transpose of upper-left 3×3).
    #[must_use]
    pub fn normal_matrix(&self) -> Mat3 {
        let inv_scale = Vec3::new(1.0 / self.scale.x, 1.0 / self.scale.y, 1.0 / self.scale.z);
        Mat3::from_quat(self.rotation) * Mat3::from_diagonal(inv_scale)
    }

    /// Convert to [`InstanceData`] with the given material `vec4`.
    #[must_use]
    pub fn to_instance(&self, material: [f32; 4]) -> InstanceData {
        let model = self.matrix();
        let normal = self.normal_matrix();
        InstanceData {
            model: model.to_cols_array_2d(),
            normal_mat: [
                normal.x_axis.to_array(),
                normal.y_axis.to_array(),
                normal.z_axis.to_array(),
            ],
            _pad: [0.0; 3],
            material,
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}
