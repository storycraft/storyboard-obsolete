/*
 * Created on Wed Sep 29 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use storyboard_core::{
    euclid::Transform3D,
    graphics::buffer::stream::{BufferStream, StreamBuffer},
    store::Store,
    unit::{PixelUnit, RenderUnit},
    wgpu::{DepthStencilState, Device, Queue},
};

use super::texture::TextureData;

#[derive(Debug, Clone)]
pub struct BackendContext<'a> {
    pub device: &'a Device,
    pub queue: &'a Queue,

    pub textures: &'a TextureData,
    pub depth_stencil: Option<&'a DepthStencilState>,
}

/// [DrawContext] contains reference to gpu backend, resources store, and stream for component data preparing
#[derive(Debug)]
pub struct DrawContext<'a, 'label> {
    pub backend: BackendContext<'a>,

    pub resources: &'a Store<BackendContext<'a>>,

    pub screen_matrix: &'a Transform3D<f32, PixelUnit, RenderUnit>,

    pub vertex_stream: &'a mut BufferStream<'label>,
    pub index_stream: &'a mut BufferStream<'label>,
}

impl<'a, 'label> DrawContext<'a, 'label> {
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
}

/// [RenderContext] contains gpu device and stream for component rendering
#[derive(Debug)]
pub struct RenderContext<'a> {
    pub backend: BackendContext<'a>,

    pub resources: &'a Store<BackendContext<'a>>,

    pub vertex_stream: StreamBuffer<'a>,
    pub index_stream: StreamBuffer<'a>,
}
