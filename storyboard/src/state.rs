/*
 * Created on Mon May 02 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

//! Prop and States implemention for storyboard app.
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use storyboard_core::{
    euclid::Size2D,
    graphics::texture::{view::TextureView2D, SizedTexture2D},
    state::{StateData, StateStatus},
    unit::PixelUnit,
    wgpu::{Sampler, TextureFormat, TextureUsages},
};
use winit::{event::Event, window::Window};

use crate::{
    graphics::{
        backend::StoryboardBackend,
        component::Drawable,
        texture::{RenderTexture2D, TextureData},
    },
    task::render::SurfaceRenderTask,
};

/// System properties for [StoryboardState].
///
/// Contains [winit::window::Window], [GraphicsData] of app
#[derive(Debug)]
pub struct StoryboardSystemProp<'a> {
    pub backend: Arc<StoryboardBackend>,
    pub texture_data: Arc<TextureData>,
    pub window: Window,
    pub elapsed: Duration,

    pub(crate) render_task: Arc<Mutex<SurfaceRenderTask<'a>>>,
}

impl StoryboardSystemProp<'_> {
    /// Create [SizedTexture2D] from descriptor
    pub fn create_texture(
        &self,
        label: Option<&str>,
        size: Size2D<u32, PixelUnit>,
        format: TextureFormat,
        usage: TextureUsages,
    ) -> SizedTexture2D {
        SizedTexture2D::init(self.backend.device(), label, size, format, usage)
    }

    /// Create [SizedTexture2D] from descriptor and upload entire data
    pub fn create_texture_with_data(
        &self,
        label: Option<&str>,
        size: Size2D<u32, PixelUnit>,
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
        size: Size2D<u32, PixelUnit>,
    ) -> SizedTexture2D {
        SizedTexture2D::init(
            self.backend.device(),
            label,
            size,
            self.texture_data.framebuffer_texture_format(),
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

    pub fn draw(&self, drawable: impl Drawable + 'static) {
        self.render_task.lock().unwrap().push(drawable);
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw()
    }
}

/// Mutable system state for [StoryboardState].
///
/// Contains event.
#[derive(Debug)]
pub struct StoryboardSystemState<'a> {
    pub event: Event<'a, ()>,
}

impl<'a> StoryboardSystemState<'a> {}

pub struct StoryboardStateData {}

impl StateData for StoryboardStateData {
    type Prop<'p> = StoryboardSystemProp<'p>;
    type State<'s> = StoryboardSystemState<'s>;
}

pub type StoryboardStateStatus = StateStatus<StoryboardStateData>;
