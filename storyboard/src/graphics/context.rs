/*
 * Created on Wed Sep 29 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use wgpu::{Device, Queue};

use super::{
    buffer::stream::{StreamBuffer, StreamBufferAllocator},
    renderer::RenderData,
    texture::TextureData,
};

#[derive(Debug)]
pub struct DrawContext<'a> {
    pub device: &'a Device,
    pub queue: &'a Queue,

    pub textures: &'a TextureData,

    pub stream_allocator: &'a mut StreamBufferAllocator,
}

impl<'a> DrawContext<'a> {
    pub fn into_render_context(self, render_data: &'a RenderData) -> RenderContext<'a> {
        RenderContext {
            device: self.device,
            queue: self.queue,
            texture_data: self.textures,
            render_data,
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

    pub render_data: &'a RenderData,
    pub texture_data: &'a TextureData,

    pub stream_buffer: StreamBuffer<'a>,
}
