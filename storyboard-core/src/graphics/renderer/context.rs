use euclid::{Rect, Transform3D};
use wgpu::{DepthStencilState, Device, Queue, TextureFormat};

use crate::{
    graphics::buffer::stream::{BufferStream, StreamBuffer},
    store::{Store, StoreResources},
    unit::{LogicalPixelUnit, RenderUnit},
};

#[derive(Debug, Clone)]
pub struct BackendContext<'a> {
    pub device: &'a Device,
    pub queue: &'a Queue,

    pub screen_format: TextureFormat,

    /// Depth stencil state used for component pipeline
    pub depth_stencil: DepthStencilState,
}

/// [DrawContext] contains reference to backend, resources store, and stream for component data preparing
#[derive(Debug)]
pub struct DrawContext<'a> {
    pub backend: BackendContext<'a>,
    pub(crate) resources: &'a Store,

    pub screen: Rect<f32, LogicalPixelUnit>,
    pub pixel_density: f32,
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
            resources: self.resources,
            vertex_stream,
            index_stream,
        }
    }

    pub fn get<T: StoreResources<BackendContext<'a>> + Sized + 'static>(&self) -> &'a T {
        self.resources.get(&self.backend)
    }
}

/// [RenderContext] contains gpu device and stream for component rendering
#[derive(Debug)]
pub struct RenderContext<'a> {
    pub backend: BackendContext<'a>,

    pub(crate) resources: &'a Store,

    pub vertex_stream: StreamBuffer<'a>,
    pub index_stream: StreamBuffer<'a>,
}

impl<'a> RenderContext<'a> {
    pub fn get<T: StoreResources<BackendContext<'a>> + Sized + 'static>(&self) -> &'a T {
        self.resources.get(&self.backend)
    }
}
