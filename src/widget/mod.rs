//! High-level iced `Shader` widget that wraps `RenderPipeline3D`.
//!
//! Consumers implement [`Scene3DProgram`] with just scene setup logic.
//! The built-in Blinn-Phong shader handles lighting automatically.
//! All pipeline creation, buffer management, `Primitive`/`Pipeline`
//! trait wiring is handled internally.
//!
//! # Simple scene (built-in Blinn-Phong, no shader needed)
//!
//! ```rust,ignore
//! use ic3d::widget::{Scene3DProgram, Scene3DSetup, MeshDrawGroup};
//! use ic3d::{Mesh, Scene, PerspectiveCamera, DirectionalLight, Transform};
//!
//! #[derive(Debug)]
//! struct MyScene;
//!
//! impl Scene3DProgram for MyScene {
//!     fn setup(&self, bounds: iced::Rectangle) -> Scene3DSetup {
//!         let camera = PerspectiveCamera::default();
//!         let light = DirectionalLight::new(/* ... */);
//!         let scene = Scene::new(&camera).light(&light).build();
//!         let instances = vec![Transform::default().to_instance([1.0, 0.0, 0.0, 32.0])];
//!
//!         Scene3DSetup {
//!             scene,
//!             draws: vec![MeshDrawGroup::new(Mesh::cube(1.0), instances)],
//!             overlays: vec![],
//!             custom_uniforms: None,
//!             clear_color: wgpu::Color::BLACK,
//!         }
//!     }
//! }
//!
//! // In your view:
//! ic3d::widget::scene_3d(MyScene)
//!     .width(Length::Fill)
//!     .height(Length::Fill)
//! ```
//!
//! # Custom fragment shader (power-user)
//!
//! ```rust,ignore
//! impl Scene3DProgram for MyScene {
//!     fn fragment_shader(&self) -> &str {
//!         include_str!("my_custom_effect.wgsl")
//!     }
//!     // ...
//! }
//! ```
//!
//! # Custom uniforms at `@group(1)` (power-user)
//!
//! ```rust,ignore
//! #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
//! #[repr(C)]
//! struct MyUniforms { reveal_radius: f32, _pad: [f32; 3] }
//!
//! impl Scene3DProgram for MyScene {
//!     fn fragment_shader(&self) -> &str {
//!         include_str!("my_custom_effect.wgsl")
//!     }
//!
//!     fn custom_uniforms_size(&self) -> usize {
//!         std::mem::size_of::<MyUniforms>()
//!     }
//!
//!     fn setup(&self, bounds: iced::Rectangle) -> Scene3DSetup {
//!         // ...
//!         Scene3DSetup {
//!             scene,
//!             draws: vec![/* ... */],
//!             overlays: vec![],
//!             custom_uniforms: Some(bytemuck::bytes_of(&MyUniforms {
//!                 reveal_radius: 5.0,
//!                 _pad: [0.0; 3],
//!             }).to_vec()),
//!             clear_color: wgpu::Color::BLACK,
//!         }
//!     }
//! }
//! ```

mod program;
mod render;
mod types;

pub use types::{MeshDrawGroup, PostProcessFactory, Scene3DProgram, Scene3DSetup};

use crate::gizmo::GizmoResult;
use crate::pipeline::utils::compose_shader;
use crate::scene::context::SceneHandle;
use crate::scene::object::SceneObjectId;
use iced::Length;
use program::Scene3DWidget;
use render::{
    CUSTOM_UNIFORM_SIZE, PIPELINE_CONFIG, POST_PROCESS_FACTORY, SHADER_SOURCE, WARMUP_MESHES,
};

/// Create a [`Scene3DBuilder`] for a 3D scene widget.
///
/// This is the main entry point. Implement [`Scene3DProgram`] and pass it
/// here. Call `.scene()` and `.on_gizmo()` to enable managed gizmo support.
///
/// # Without gizmo (simple usage)
///
/// ```rust,ignore
/// scene_3d(MyScene).width(Length::Fill).height(Length::Fill)
/// ```
///
/// # With managed gizmo
///
/// ```rust,ignore
/// scene_3d(MyScene)
///     .scene(self.scene_handle.clone())
///     .on_gizmo(Message::Gizmo)
///     .width(Length::Fill)
///     .height(Length::Fill)
/// ```
#[must_use]
pub fn scene_3d<Message: 'static>(
    program: impl Scene3DProgram + 'static,
) -> Scene3DBuilder<Message> {
    Scene3DBuilder {
        program: Box::new(program),
        scene_handle: None,
        on_gizmo: None,
        width: Length::Fill,
        height: Length::Fill,
    }
}

/// Builder for a 3D scene widget, returned by [`scene_3d()`].
///
/// Call `.scene()` and `.on_gizmo()` to enable managed gizmo support.
/// Convert to an iced `Element` with `.into()`.
pub struct Scene3DBuilder<Message> {
    program: Box<dyn Scene3DProgram>,
    scene_handle: Option<SceneHandle>,
    on_gizmo: Option<Box<dyn Fn(SceneObjectId, GizmoResult) -> Message>>,
    width: Length,
    height: Length,
}

impl<Message: 'static> Scene3DBuilder<Message> {
    /// Attach a [`SceneHandle`] for cross-frame state and managed gizmo support.
    ///
    /// The widget populates the handle each frame with camera, viewport, and
    /// object transform data. When combined with [`on_gizmo`](Self::on_gizmo),
    /// the widget handles all gizmo input and rendering automatically.
    #[must_use]
    pub fn scene(mut self, handle: SceneHandle) -> Self {
        self.scene_handle = Some(handle);
        self
    }

    /// Set the callback for managed gizmo events.
    ///
    /// When the user interacts with the managed gizmo (hover, translate),
    /// this callback converts the result into an iced `Message`. Use with
    /// [`SceneHandle::select`] to activate the managed gizmo on an object.
    ///
    /// ```rust,ignore
    /// enum Message {
    ///     Gizmo(SceneObjectId, GizmoResult),
    /// }
    ///
    /// scene_3d(my_scene).on_gizmo(Message::Gizmo)
    /// ```
    #[must_use]
    pub fn on_gizmo(mut self, f: impl Fn(SceneObjectId, GizmoResult) -> Message + 'static) -> Self {
        self.on_gizmo = Some(Box::new(f));
        self
    }

    /// Set the widget width (default: `Length::Fill`).
    #[must_use]
    pub fn width(mut self, w: impl Into<Length>) -> Self {
        self.width = w.into();
        self
    }

    /// Set the widget height (default: `Length::Fill`).
    #[must_use]
    pub fn height(mut self, h: impl Into<Length>) -> Self {
        self.height = h.into();
        self
    }
}

impl<'a, Message: 'static> From<Scene3DBuilder<Message>> for iced::Element<'a, Message> {
    fn from(builder: Scene3DBuilder<Message>) -> Self {
        // Stash data for Pipeline::new() to pick up.
        *SHADER_SOURCE.lock() = Some(compose_shader(builder.program.fragment_shader()));
        *PIPELINE_CONFIG.lock() = Some(builder.program.pipeline_config());
        *CUSTOM_UNIFORM_SIZE.lock() = {
            let size = builder.program.custom_uniforms_size();
            if size > 0 {
                Some(size)
            } else {
                None
            }
        };

        let warmup = builder.program.warmup_meshes();
        *WARMUP_MESHES.lock() = if warmup.is_empty() {
            None
        } else {
            Some(warmup)
        };

        *POST_PROCESS_FACTORY.lock() = builder.program.post_process_factory();

        iced::widget::Shader::new(Scene3DWidget {
            program: builder.program,
            scene_handle: builder.scene_handle,
            on_gizmo: builder.on_gizmo,
        })
        .width(builder.width)
        .height(builder.height)
        .into()
    }
}
