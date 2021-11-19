/*
 * Created on Wed Sep 29 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use wgpu::{Device, Queue};

use super::{
    buffer::stream::{StreamBuffer, StreamBufferAllocator},
    texture::resources::TextureResources,
};

#[derive(Debug)]
pub struct DrawContext<'a> {
    pub device: &'a Device,
    pub queue: &'a Queue,

    pub textures: &'a TextureResources,

    pub stream_allocator: &'a mut StreamBufferAllocator,
}

impl<'a> DrawContext<'a> {
    pub fn into_render_context(self) -> RenderContext<'a> {
        RenderContext {
            device: self.device,
            queue: self.queue,
            textures: self.textures,
            stream_buffer: self.stream_allocator.flush(self.device),
        }
    }

    pub fn sub_context(&self, stream_allocator: &'a mut StreamBufferAllocator) -> DrawContext<'a> {
        DrawContext {
            device: self.device,
            queue: self.queue,
            textures: self.textures,
            stream_allocator,
        }
    }
}

#[derive(Debug)]
pub struct RenderContext<'a> {
    pub device: &'a Device,
    pub queue: &'a Queue,
    pub textures: &'a TextureResources,
    pub stream_buffer: StreamBuffer<'a>,
}
