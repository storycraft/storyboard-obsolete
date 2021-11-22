/*
 * Created on Sun Nov 14 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

pub mod box2d;
pub mod path;
pub mod primitive;

use dynstack::{dyn_push, DynStack};
use wgpu::{
    util::DeviceExt, BindGroup, ColorTargetState, CommandEncoder, DepthStencilState, Device,
    Extent3d, Queue, RenderPipeline, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages, TextureViewDescriptor,
};

use self::{
    box2d::{init_box_pipeline, init_box_pipeline_layout, init_box_shader},
    path::{init_path_pipeline, init_path_pipeline_layout, init_path_shader},
    primitive::{init_primitive_pipeline, init_primitive_pipeline_layout, init_primitive_shader},
};

use super::{
    buffer::index::IndexBuffer,
    context::{DrawContext, RenderContext},
    pass::StoryboardRenderPass,
    texture::{create_texture_bind_group, TextureData},
};

pub trait DrawState: Send + Sync {
    fn prepare(
        &mut self,
        ctx: &mut DrawContext,
        depth: f32,
        encoder: &mut CommandEncoder,
        state_queue: &mut RenderStateQueue,
    );
}

pub trait RenderState: Send + Sync {
    fn render<'r>(
        &'r mut self,
        context: &'r RenderContext<'r>,
        pass: &mut StoryboardRenderPass<'r>,
    );
}

pub struct StoryboardRenderer<'a> {
    draw_state_queue: DynStack<dyn DrawState + 'a>,
    render_state_queue: RenderStateQueue<'a>,
}

impl<'a> StoryboardRenderer<'a> {
    pub fn new() -> Self {
        Self {
            draw_state_queue: DynStack::new(),
            render_state_queue: RenderStateQueue::new(),
        }
    }

    pub fn append(&mut self, draw_state: impl DrawState + 'a) {
        dyn_push!(self.draw_state_queue, draw_state);
    }

    pub fn prepare(&mut self, ctx: &mut DrawContext, encoder: &mut CommandEncoder) {
        let len = self.draw_state_queue.len() as f32;

        for (i, draw_state) in self.draw_state_queue.iter_mut().enumerate() {
            let depth = 1.0 - i as f32 / len;

            draw_state.prepare(ctx, depth, encoder, &mut self.render_state_queue);
        }
    }

    pub fn render<'rpass>(
        &'rpass mut self,
        ctx: &'rpass RenderContext<'rpass>,
        pass: &mut StoryboardRenderPass<'rpass>,
    ) {
        for render_state in self.render_state_queue.0.iter_mut() {
            render_state.render(ctx, pass);
        }
    }
}

pub struct RenderStateQueue<'a>(DynStack<dyn RenderState + 'a>);

impl<'a> RenderStateQueue<'a> {
    pub fn new() -> Self {
        Self(DynStack::new())
    }

    pub fn push(&mut self, state: impl RenderState + 'a) {
        dyn_push!(self.0, state);
    }
}

#[derive(Debug)]
pub struct RenderData {
    pub empty_texture_bind_group: BindGroup,

    pub quad_index_buffer: IndexBuffer,

    pub primitive_pipeline: RenderPipeline,
    pub box_pipeline: RenderPipeline,
    pub path_pipeline: RenderPipeline,
}

impl RenderData {
    pub fn init(
        device: &Device,
        queue: &Queue,
        texture_data: &TextureData,
        fragment_targets: &[ColorTargetState],
        depth_stencil: Option<DepthStencilState>,
    ) -> Self {
        let primitive_shader = init_primitive_shader(device);
        let box_shader = init_box_shader(device);
        let path_shader = init_path_shader(device);

        let primitive_pipeline_layout =
            init_primitive_pipeline_layout(device, texture_data.bind_group_layout());
        let box_pipeline_layout =
            init_box_pipeline_layout(device, texture_data.bind_group_layout());
        let path_pipeline_layout = init_path_pipeline_layout(device);

        let primitive_pipeline = init_primitive_pipeline(
            device,
            &primitive_pipeline_layout,
            &primitive_shader,
            fragment_targets,
            depth_stencil.clone(),
        );
        let box_pipeline = init_box_pipeline(
            device,
            &box_pipeline_layout,
            &box_shader,
            fragment_targets,
            depth_stencil.clone(),
        );
        let path_pipeline = init_path_pipeline(
            device,
            &path_pipeline_layout,
            &path_shader,
            fragment_targets,
            depth_stencil,
        );

        let quad_index_buffer =
            IndexBuffer::init(device, Some("Quad index buffer"), &[0, 1, 2, 3, 0, 2]);

        let empty_texture = device.create_texture_with_data(
            queue,
            &TextureDescriptor {
                label: Some("RendererData empty texture"),
                size: Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8Unorm,
                usage: TextureUsages::TEXTURE_BINDING,
            },
            &[0xff, 0xff, 0xff, 0xff],
        );

        let empty_texture_bind_group = create_texture_bind_group(
            device,
            texture_data.bind_group_layout(),
            &empty_texture.create_view(&TextureViewDescriptor::default()),
            texture_data.default_sampler(),
        );

        Self {
            empty_texture_bind_group,

            quad_index_buffer,

            primitive_pipeline,
            box_pipeline,
            path_pipeline,
        }
    }
}
