use storyboard_core::{
    euclid::Transform3D,
    store::StoreResources,
    unit::{LogicalPixelUnit, RenderUnit},
};
use wgpu::{Device, Queue, TextureFormat};

use crate::{
    buffer::stream::{BufferStream, StreamBuffer},
    ScreenRect,
};

use super::RendererData;

/// Contains wgpu [`Device`], wgpu [`Queue`] and [`RendererData`]
#[derive(Debug, Clone, Copy)]
pub struct BackendContext<'a> {
    pub device: &'a Device,
    pub queue: &'a Queue,
    pub renderer_data: &'a RendererData,
}

impl<'a> BackendContext<'a> {
    #[inline]
    pub const fn new(
        device: &'a Device,
        queue: &'a Queue,
        renderer_data: &'a RendererData,
    ) -> Self {
        Self {
            device,
            queue,
            renderer_data,
        }
    }

    #[inline]
    pub const fn screen_format(&self) -> TextureFormat {
        self.renderer_data.screen_format()
    }

    #[inline]
    pub fn get<T: StoreResources<BackendContext<'a>>>(&self) -> &'a T {
        self.renderer_data.store.get(self)
    }
}

/// [DrawContext] contains reference to backend, resources store, and stream for component data preparing
#[derive(Debug)]
pub struct DrawContext<'a> {
    pub backend: BackendContext<'a>,

    pub screen: ScreenRect,
    pub screen_matrix: &'a Transform3D<f32, LogicalPixelUnit, RenderUnit>,

    pub vertex_stream: &'a mut BufferStream<'static>,
    pub index_stream: &'a mut BufferStream<'static>,
}

impl<'a> DrawContext<'a> {
    pub fn into_render_context(self) -> RenderContext<'a> {
        let vertex_stream = self
            .vertex_stream
            .finish(self.backend.device, self.backend.queue);
        let index_stream = self
            .index_stream
            .finish(self.backend.device, self.backend.queue);

        RenderContext {
            backend: self.backend,
            vertex_stream,
            index_stream,
        }
    }

    #[inline]
    pub fn get<T: StoreResources<BackendContext<'a>>>(&self) -> &'a T {
        self.backend.get::<T>()
    }
}

/// [RenderContext] contains gpu device and stream for component rendering
#[derive(Debug)]
pub struct RenderContext<'a> {
    pub backend: BackendContext<'a>,

    pub vertex_stream: StreamBuffer<'a>,
    pub index_stream: StreamBuffer<'a>,
}

impl<'a> RenderContext<'a> {
    pub fn get<T: StoreResources<BackendContext<'a>>>(&self) -> &'a T {
        self.backend.renderer_data.store.get(&self.backend)
    }
}
