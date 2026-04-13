//! Common math utilities for 3D applications.
//!
//! Organized into submodules by domain:
//! - [`interp`] — lerp, inverse lerp, remap
//! - [`easing`] — smoothstep, ease-out-back, ease-smooth, ease-out-cubic
//! - [`hash`] — deterministic spatial hashing
//! - [`screen`] — world-to-screen projection, hit testing, constant-size scaling
//! - [`curve`] — Bézier flattening, 2D distance
//!
//! All functions are re-exported at the `math` level for convenience:
//! ```rust,ignore
//! use ic3d::math::lerp;
//! // or via submodule:
//! use ic3d::math::interp::lerp;
//! ```

pub mod curve;
pub mod distance;
pub mod easing;
pub mod hash;
pub mod hex_grid;
pub mod interp;
pub mod ray;
pub mod screen;

pub use curve::*;
pub use distance::*;
pub use easing::*;
pub use hash::*;
pub use hex_grid::*;
pub use interp::*;
pub use screen::*;
