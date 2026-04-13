//! Gizmo type definitions: mode, axis, and interaction result.

use glam::{Quat, Vec3};

/// The type of manipulation a gizmo performs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoMode {
    /// Move the object along axes.
    Translate,
}

/// An axis that can be hovered or dragged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoAxis {
    /// +X axis (red).
    X,
    /// +Y axis (green).
    Y,
    /// +Z axis (blue).
    Z,
}

impl GizmoAxis {
    /// Unit direction vector for this axis.
    #[must_use]
    pub fn direction(self) -> Vec3 {
        match self {
            Self::X => Vec3::X,
            Self::Y => Vec3::Y,
            Self::Z => Vec3::Z,
        }
    }

    /// Material color for this axis.
    ///
    /// X = red (0.96, 0.20, 0.32), Y = green (0.53, 0.84, 0.01),
    /// Z = blue (0.16, 0.55, 0.96).
    #[must_use]
    pub(crate) fn color(self) -> [f32; 4] {
        match self {
            Self::X => [0.96, 0.20, 0.32, 1.0],
            Self::Y => [0.53, 0.84, 0.01, 1.0],
            Self::Z => [0.16, 0.55, 0.96, 1.0],
        }
    }

    /// Brighter highlight color when hovered or active.
    ///
    /// Desaturated to ~25% saturation with value boosted to 1.0.
    #[must_use]
    pub(crate) fn highlight_color(self) -> [f32; 4] {
        match self {
            Self::X => [1.0, 0.75, 0.78, 1.0],
            Self::Y => [0.88, 1.0, 0.75, 1.0],
            Self::Z => [0.75, 0.86, 1.0, 1.0],
        }
    }

    /// Rotation to orient a +Y arrow along this axis.
    #[must_use]
    pub(crate) fn rotation(self) -> Quat {
        match self {
            Self::X => Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2),
            Self::Y => Quat::IDENTITY,
            Self::Z => Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
        }
    }

    /// All three axes.
    pub(crate) const ALL: [Self; 3] = [Self::X, Self::Y, Self::Z];
}

/// Result of a gizmo interaction for the current frame.
#[derive(Debug, Clone, Copy)]
pub enum GizmoResult {
    /// The cursor is hovering over an axis handle (no drag).
    Hover(GizmoAxis),
    /// The cursor moved off the gizmo after previously hovering.
    Unhover,
    /// A translation drag produced a world-space delta this frame.
    Translate(Vec3),
}
