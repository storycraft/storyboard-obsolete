/*
 * Created on Mon Sep 20 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::sync::Arc;
use storyboard_graphics::renderer::RenderStateQueue;
use storyboard_graphics::wgpu::{CommandEncoder, RenderPipeline};

use storyboard_graphics::{
    buffer::stream::StreamSlice,
    component::{DrawState, RenderState},
    context::{DrawContext, RenderContext},
    pass::StoryboardRenderPass,
    texture::Texture2D,
};

use super::PrimitiveVertex;

#[derive(Debug, Clone)]
pub struct TriangleDrawState<'a> {
    pub pipeline: &'a RenderPipeline,
    pub texture: Option<Arc<Texture2D>>,
    pub primitive: [PrimitiveVertex; 3],
}

impl<'a> DrawState<'a> for TriangleDrawState<'a> {
    fn prepare(
        &mut self,
        context: &mut DrawContext,
        depth: f32,
        _: &mut CommandEncoder,
        state_queue: &mut RenderStateQueue<'a>
    ) {
        let primitive = {
            let mut pritmitive = self.primitive;

            pritmitive[0].position[2] = depth;
            pritmitive[1].position[2] = depth;
            pritmitive[2].position[2] = depth;

            pritmitive
        };

        let slice = bytemuck::cast_slice(&primitive);
        let mut entry = context.stream_allocator.start_entry();
        entry.write(slice);

        state_queue.push(TriangleRenderState {
            pipeline: self.pipeline,
            vertex: entry.finish(),
            texture: self.texture.clone(),
        });
    }
}

#[derive(Debug)]
pub struct TriangleRenderState<'a> {
    pub pipeline: &'a RenderPipeline,
    pub vertex: StreamSlice,
    pub texture: Option<Arc<Texture2D>>,
}

impl RenderState for TriangleRenderState<'_> {
    fn render<'r>(&'r mut self, context: &'r RenderContext, pass: &mut StoryboardRenderPass<'r>) {
        pass.set_pipeline(self.pipeline);
        pass.set_vertex_buffer(0, context.stream_buffer.slice(&self.vertex));
        pass.set_bind_group(
            0,
            context
                .textures
                .texture_bind_group_or_empty(self.texture.as_deref()),
            &[],
        );

        pass.draw(0..3, 0..1);
    }
}
