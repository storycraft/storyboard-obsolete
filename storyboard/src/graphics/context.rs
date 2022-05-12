/*
 * Created on Wed Sep 29 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use storyboard_core::{
    graphics::buffer::stream::{BufferStream, StreamBuffer},
    wgpu::{Device, Queue},
};

/// [DrawContext] contains gpu device and stream for component data preparing
#[derive(Debug)]
pub struct DrawContext<'a, 'label> {
    pub device: &'a Device,
    pub queue: &'a Queue,

    pub vertex_stream: &'a mut BufferStream<'label>,
    pub index_stream: &'a mut BufferStream<'label>,
}

impl<'a, 'label> DrawContext<'a, 'label> {
    pub fn into_render_context(self) -> RenderContext<'a> {
        RenderContext {
            device: self.device,
            queue: self.queue,
            vertex_stream: self.vertex_stream.finish(self.device),
            index_stream: self.index_stream.finish(self.device),
        }
    }
}

/// [RenderContext] contains gpu device and stream for component rendering
#[derive(Debug)]
pub struct RenderContext<'a> {
    pub device: &'a Device,
    pub queue: &'a Queue,

    pub vertex_stream: StreamBuffer<'a>,
    pub index_stream: StreamBuffer<'a>,
}
