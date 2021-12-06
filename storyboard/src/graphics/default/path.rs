/*
 * Created on Sat Nov 20 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use euclid::Transform3D;
use lyon::lyon_tessellation::{FillVertex, FillVertexConstructor, StrokeVertex, StrokeVertexConstructor, VertexBuffers};
use palette::LinSrgba;
use wgpu::{CommandEncoder, IndexFormat};

use crate::graphics::{
    buffer::stream::StreamSlice,
    context::{DrawContext, RenderContext},
    pass::StoryboardRenderPass,
    renderer::{
        path::{PathInstance, PathVertex},
        DrawState, RenderState, RenderStateQueue,
    },
    PixelUnit, RenderUnit,
};

#[derive(Debug, Clone)]
pub struct PathVertexBuilder {
    pub color: LinSrgba<f32>,
}

impl FillVertexConstructor<PathVertex> for PathVertexBuilder {
    fn new_vertex(&mut self, vertex: FillVertex) -> PathVertex {
        let pos = vertex.position().to_array();
        PathVertex {
            position: [pos[0], pos[1], 0.0],
            color: self.color,
        }
    }
}

impl StrokeVertexConstructor<PathVertex> for PathVertexBuilder {
    fn new_vertex(&mut self, vertex: StrokeVertex) -> PathVertex {
        let pos = vertex.position().to_array();

        PathVertex {
            position: [pos[0], pos[1], 0.0],
            color: self.color,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PathDrawState {
    pub path: VertexBuffers<PathVertex, u16>,

    pub matrix: Transform3D<f32, PixelUnit, RenderUnit>,
}

impl DrawState for PathDrawState {
    fn prepare(
        &mut self,
        context: &mut DrawContext,
        depth: f32,
        _: &mut CommandEncoder,
        state_queue: &mut RenderStateQueue,
    ) {
        let vertex = {
            let mut entry = context.stream_allocator.start_entry();

            self.path
                .vertices
                .iter_mut()
                .for_each(|item| item.position[2] = depth);

            entry.write(bytemuck::cast_slice(&self.path.vertices));

            entry.finish()
        };

        let indices = {
            let mut entry = context.stream_allocator.start_entry();

            entry.write(bytemuck::cast_slice(&self.path.indices));

            (entry.finish(), self.path.indices.len() as u32)
        };

        let instance = {
            let mut entry = context.stream_allocator.start_entry();

            entry.write(bytemuck::bytes_of(&PathInstance {
                matrix: self.matrix.to_array(),
            }));

            entry.finish()
        };

        state_queue.push(PathRenderState {
            vertex,
            indices,
            instance,
        });
    }
}

#[derive(Debug)]
pub struct PathRenderState {
    pub vertex: StreamSlice,
    pub indices: (StreamSlice, u32),

    pub instance: StreamSlice,
}

impl RenderState for PathRenderState {
    fn render<'r>(&'r self, context: &RenderContext<'r>, pass: &mut StoryboardRenderPass<'r>) {
        pass.set_pipeline(&context.render_data.path_pipeline);

        pass.set_vertex_buffer(0, context.stream_buffer.slice(&self.vertex));
        pass.set_vertex_buffer(1, context.stream_buffer.slice(&self.instance));

        pass.set_index_buffer(
            context.stream_buffer.slice(&self.indices.0),
            IndexFormat::Uint16,
        );

        pass.draw_indexed(0..self.indices.1, 0, 0..1);
    }
}
