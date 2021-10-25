/*
 * Created on Mon Sep 20 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use storyboard::renderer::RenderStateQueue;

use std::sync::Arc;

use storyboard::wgpu::{CommandEncoder, RenderPipeline};

use storyboard::{
    buffer::{index::IndexBuffer, stream::StreamSlice},
    component::{DrawState, RenderState},
    context::{DrawContext, RenderContext},
    pass::StoryboardRenderPass,
    texture::Texture2D,
};

use super::PrimitiveVertex;

#[derive(Debug, Clone)]
pub struct RectDrawState<'a> {
    pub pipeline: &'a RenderPipeline,
    pub quad_index_buffer: &'a IndexBuffer,
    pub texture: Option<Arc<Texture2D>>,
    pub primitive: [PrimitiveVertex; 4],
}

impl<'a> DrawState<'a> for RectDrawState<'a> {
    fn prepare(
        &mut self,
        context: &mut DrawContext,
        depth: f32,
        _: &mut CommandEncoder,
        state_queue: &mut RenderStateQueue<'a>
    ) {
        let primitive = {
            let mut primitive = self.primitive;

            primitive[0].position[2] = depth;
            primitive[1].position[2] = depth;
            primitive[2].position[2] = depth;
            primitive[3].position[2] = depth;

            primitive
        };

        let slice = bytemuck::cast_slice(&primitive);
        let mut entry = context.stream_allocator.start_entry();
        entry.write(slice);

        state_queue.push(RectRenderState {
            pipeline: self.pipeline,
            quad_index_buffer: self.quad_index_buffer,
            vertex: entry.finish(),
            texture: self.texture.clone(),
        });
    }
}

#[derive(Debug)]
pub struct RectRenderState<'a> {
    pub pipeline: &'a RenderPipeline,
    pub quad_index_buffer: &'a IndexBuffer,
    pub vertex: StreamSlice,
    pub texture: Option<Arc<Texture2D>>,
}

impl RenderState for RectRenderState<'_> {
    fn render<'r>(&'r mut self, context: &'r RenderContext, pass: &mut StoryboardRenderPass<'r>) {
        pass.set_pipeline(self.pipeline);
        pass.set_vertex_buffer(0, context.stream_buffer.slice(&self.vertex));
        pass.set_index_buffer(self.quad_index_buffer.slice(), IndexBuffer::FORMAT);
        pass.set_bind_group(
            0,
            context
                .textures
                .texture_bind_group_or_empty(self.texture.as_deref()),
            &[],
        );

        pass.draw_indexed(0..6, 0, 0..1);
    }
}
