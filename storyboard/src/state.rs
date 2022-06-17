//! Prop and States implemention for storyboard app.
use std::{sync::Arc, time::Duration};

use storyboard_core::{
    euclid::Size2D,
    graphics::{
        backend::StoryboardBackend,
        component::Drawable,
        texture::{SizedTexture2D, TextureView2D},
    },
    state::{StateData, StateStatus},
    store::{Store, StoreResources},
    unit::PhyiscalPixelUnit,
    wgpu::{Sampler, TextureFormat, TextureUsages},
};
use winit::{event::Event, window::Window};

use crate::{
    graphics::texture::{data::TextureData, RenderTexture2D},
    task::render::RenderTask,
};

/// System properties for [StoryboardState].
///
/// Contains [winit::window::Window], [GraphicsData] of app
#[derive(Debug)]
pub struct StoryboardSystemProp {
    pub backend: Arc<StoryboardBackend>,
    pub screen_format: TextureFormat,
    pub texture_data: Arc<TextureData>,
    pub window: Window,

    pub elapsed: Duration,

    pub(crate) store: Arc<Store>,
}

impl StoryboardSystemProp {
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

    pub const fn store(&self) -> &Arc<Store> {
        &self.store
    }

    pub fn get<'a, T: StoreResources<GlobalStoreContext<'a>> + Sized + 'static>(
        &'a mut self,
    ) -> &'a T {
        self.store.get(&GlobalStoreContext {
            backend: &self.backend,
            texture_data: &self.texture_data,
        })
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw()
    }
}

#[derive(Debug)]
pub struct GlobalStoreContext<'a> {
    pub backend: &'a Arc<StoryboardBackend>,
    pub texture_data: &'a Arc<TextureData>,
}

/// Mutable system state for [StoryboardState].
///
/// Contains event.
#[derive(Debug)]
pub struct StoryboardSystemState<'a> {
    pub event: Event<'a, ()>,
    
    pub(crate) render_task: &'a mut RenderTask,
}

impl<'a> StoryboardSystemState<'a> {
    pub const fn render_task(&self) -> &RenderTask {
        self.render_task
    }

    #[inline]
    pub fn draw(&mut self, drawable: impl Drawable + 'static) {
        self.render_task.push(drawable);
    }
}

pub struct StoryboardStateData {}

impl StateData for StoryboardStateData {
    type Prop<'p> = StoryboardSystemProp;
    type State<'s> = StoryboardSystemState<'s>;
}

pub type StoryboardStateStatus = StateStatus<StoryboardStateData>;
