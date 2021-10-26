/*
 * Created on Thu Oct 07 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use lyon::lyon_tessellation::{
    FillVertex, FillVertexConstructor, StrokeVertex, StrokeVertexConstructor, VertexBuffers,
};
use storyboard::color::LinSrgba;
use storyboard::pipeline::PipelineTargetDescriptor;
use storyboard::renderer::RenderStateQueue;
use storyboard::wgpu::{CommandEncoder, Device, IndexFormat, RenderPipeline};

use storyboard::{
    buffer::stream::StreamSlice,
    component::{DrawBox, DrawSpace, DrawState, Drawable, RenderState},
    context::{DrawContext, RenderContext},
    pass::StoryboardRenderPass,
};

use crate::{init_path_pipeline, init_path_shader, Path, PathInstance, PathVertex, ScalablePath};

#[derive(Debug)]
pub struct PathCompositor {
    pipeline: RenderPipeline,
}

impl PathCompositor {
    pub const fn new(pipeline: RenderPipeline) -> Self {
        Self { pipeline }
    }

    pub fn init(device: &Device, pipeline_desc: PipelineTargetDescriptor) -> Self {
        let shader = init_path_shader(device);
        let pipeline = init_path_pipeline(device, &shader, pipeline_desc);

        Self { pipeline }
    }

    pub fn path_scalable(
        &self,
        sized_path: &ScalablePath,
        draw_box: &DrawBox,
    ) -> Drawable<PathDrawState> {
        let opaque = !sized_path
            .path
            .vertices
            .iter()
            .any(|item| item.color.alpha != 1.0);

        let path = sized_path.path.clone();

        let matrix = draw_box
            .matrix
            .pre_scale(
                draw_box.rect.size.width / sized_path.rect.size.width,
                draw_box.rect.size.height / sized_path.rect.size.height,
                1.0,
            )
            .pre_translate(-sized_path.rect.origin.to_3d().to_vector())
            .to_array();

        let instance = PathInstance { matrix };

        Drawable {
            opaque,
            state: PathDrawState {
                pipeline: &self.pipeline,

                path,
                instance,
            },
        }
    }

    pub fn path(&self, path: &Path, space: &DrawSpace) -> Drawable<PathDrawState> {
        let opaque = !path.vertices.iter().any(|item| item.color.alpha != 1.0);

        let path = path.clone();

        let instance = PathInstance {
            matrix: space.matrix.to_array(),
        };

        Drawable {
            opaque,
            state: PathDrawState {
                pipeline: &self.pipeline,

                path,
                instance,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct PathDrawState<'a> {
    pub pipeline: &'a RenderPipeline,

    pub path: VertexBuffers<PathVertex, u16>,

    pub instance: PathInstance,
}

impl<'a> DrawState<'a> for PathDrawState<'a> {
    fn prepare(
        &mut self,
        context: &mut DrawContext,
        depth: f32,
        _: &mut CommandEncoder,
        state_queue: &mut RenderStateQueue<'a>,
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

            entry.write(bytemuck::bytes_of(&self.instance));

            entry.finish()
        };

        state_queue.push(PathRenderState {
            pipeline: self.pipeline,

            vertex,
            indices,
            instance,
        });
    }
}

#[derive(Debug)]
pub struct PathRenderState<'a> {
    pub pipeline: &'a RenderPipeline,

    pub vertex: StreamSlice,
    pub indices: (StreamSlice, u32),

    pub instance: StreamSlice,
}

impl RenderState for PathRenderState<'_> {
    fn render<'r>(&'r mut self, context: &'r RenderContext, pass: &mut StoryboardRenderPass<'r>) {
        pass.set_pipeline(self.pipeline);

        pass.set_vertex_buffer(0, context.stream_buffer.slice(&self.vertex));
        pass.set_vertex_buffer(1, context.stream_buffer.slice(&self.instance));

        pass.set_index_buffer(
            context.stream_buffer.slice(&self.indices.0),
            IndexFormat::Uint16,
        );

        pass.draw_indexed(0..self.indices.1, 0, 0..1);
    }
}

#[derive(Debug, Clone)]
pub struct PathFiller {
    pub color: LinSrgba<f32>,
}

impl FillVertexConstructor<PathVertex> for PathFiller {
    fn new_vertex(&mut self, vertex: FillVertex) -> PathVertex {
        let pos = vertex.position().to_array();
        PathVertex {
            position: [pos[0], pos[1], 0.0],
            color: self.color,
        }
    }
}

impl StrokeVertexConstructor<PathVertex> for PathFiller {
    fn new_vertex(&mut self, vertex: StrokeVertex) -> PathVertex {
        let pos = vertex.position().to_array();

        PathVertex {
            position: [pos[0], pos[1], 0.0],
            color: self.color,
        }
    }
}
