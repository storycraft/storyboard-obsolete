//! Prop and States implemention for storyboard app.
use std::{sync::Arc, time::Duration};

use storyboard_core::{euclid::Size2D, unit::PhyiscalPixelUnit};
use storyboard_render::{
    backend::StoryboardBackend,
    component::Drawable,
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
    pub screen_format: TextureFormat,
    pub texture_data: Arc<TextureData>,
    pub window: Window,

    pub elapsed: Duration,
}

impl StoryboardAppProp {
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
            self.screen_format,
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
        RenderTexture2D::init(
            self.backend.device(),
            view,
            self.texture_data.bind_group_layout(),
            sampler.unwrap_or(self.texture_data.default_sampler()),
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
