//! Prop and States implemention for storyboard app.
use std::{sync::Arc, time::Duration};

use storyboard_core::{euclid::Size2D, store::StoreResources, unit::PhyiscalPixelUnit};
use storyboard_render::{
    backend::StoryboardBackend,
    component::Drawable,
    shared::{
        BackendScope, BackendScopeContext, BackendShared, RenderScope, RenderScopeContext,
        RenderShared,
    },
    task::RenderTask,
    texture::{SizedTexture2D, TextureView2D},
    wgpu::{Sampler, TextureFormat, TextureUsages},
};
use storyboard_texture::render::{data::TextureData, RenderTexture2D};
use winit::{event::Event, event_loop::ControlFlow, window::Window};

pub trait StoryboardApp {
    fn load(&mut self, prop: &StoryboardAppProp);
    fn unload(&mut self, prop: &StoryboardAppProp);

    fn update(&mut self, prop: &StoryboardAppProp, state: &mut StoryboardAppState);
}

/// System properties for [StoryboardState].
///
/// Contains [winit::window::Window], [GraphicsData] of app
#[derive(Debug)]
pub struct StoryboardAppProp {
    pub backend: Arc<StoryboardBackend>,
    pub backend_shared: Arc<BackendShared>,
    pub render_shared: Arc<RenderShared>,

    pub window: Arc<Window>,

    pub elapsed: Duration,
}

impl StoryboardAppProp {
    #[inline]
    pub fn texture_format(&self) -> TextureFormat {
        self.render_shared.texture_format()
    }

    #[inline]
    fn backend_scope_context(&self) -> BackendScopeContext {
        BackendScopeContext {
            device: self.backend.device(),
            queue: self.backend.queue(),
        }
    }

    #[inline]
    pub fn backend_scope(&self) -> BackendScope {
        self.backend_shared.scope(self.backend_scope_context())
    }

    #[inline]
    pub fn render_scope(&self) -> RenderScope {
        self.backend_scope().render_scope(&self.render_shared)
    }

    #[inline]
    pub fn backend_get<T: for<'ctx> StoreResources<BackendScopeContext<'ctx>>>(&self) -> &T {
        self.backend_shared.get(self.backend_scope_context())
    }

    #[inline]
    pub fn render_get<T: for<'ctx> StoreResources<RenderScopeContext<'ctx>>>(&self) -> &T {
        self.render_shared.get(self.backend_scope())
    }

    pub fn texture_data(&self) -> &TextureData {
        self.backend_get()
    }

    /// Create [SizedTexture2D] from descriptor
    pub fn create_texture(
        &self,
        label: Option<&str>,
        size: Size2D<u32, PhyiscalPixelUnit>,
        format: TextureFormat,
        usage: TextureUsages,
    ) -> SizedTexture2D {
        SizedTexture2D::init(self.backend.device(), label, size, format, usage)
    }

    /// Create [SizedTexture2D] from descriptor and upload entire data
    pub fn create_texture_with_data(
        &self,
        label: Option<&str>,
        size: Size2D<u32, PhyiscalPixelUnit>,
        format: TextureFormat,
        usage: TextureUsages,
        data: &[u8],
    ) -> SizedTexture2D {
        let tex = SizedTexture2D::init(self.backend.device(), label, size, format, usage);
        tex.write(self.backend.queue(), None, data);

        tex
    }

    /// Create Framebuffer capable texture, having same texture format as surface
    pub fn create_frame_buffer_texture(
        &self,
        label: Option<&str>,
        size: Size2D<u32, PhyiscalPixelUnit>,
    ) -> SizedTexture2D {
        SizedTexture2D::init(
            self.backend.device(),
            label,
            size,
            self.texture_format(),
            TextureUsages::TEXTURE_BINDING
                | TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::COPY_SRC,
        )
    }

    /// Create [RenderTexture2D] from [TextureView2D] using texture_data
    pub fn create_render_texture(
        &self,
        view: TextureView2D,
        sampler: Option<&Sampler>,
    ) -> RenderTexture2D {
        let texture_data = self.texture_data();

        RenderTexture2D::init(
            self.backend.device(),
            view,
            texture_data.bind_group_layout(),
            sampler.unwrap_or_else(|| texture_data.default_sampler()),
        )
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw()
    }
}

/// Mutable system state.
///
/// Contains event.
#[derive(Debug)]
pub struct StoryboardAppState<'a> {
    pub event: Event<'a, ()>,

    pub control_flow: &'a mut ControlFlow,

    pub render_task: &'a mut RenderTask,
}

impl<'a> StoryboardAppState<'a> {
    #[inline]
    pub fn draw(&mut self, drawable: impl Drawable + 'static) {
        self.render_task.push(drawable);
    }

    #[inline]
    pub fn render(&mut self) {
        self.render_task.submit();
    }
}
