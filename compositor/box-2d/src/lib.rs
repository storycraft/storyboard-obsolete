/*
 * Created on Thu Oct 07 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

pub mod compositor;

use std::{borrow::Cow, sync::Arc};

use bytemuck::{Pod, Zeroable};

use storyboard_graphics::component::extent::ExtentUnit;
use storyboard_graphics::renderer::RenderStateQueue;
use storyboard_graphics::wgpu::{
    vertex_attr_array, BindGroupLayout, BufferAddress, CommandEncoder, Device, FragmentState,
    PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology, RenderPipeline,
    RenderPipelineDescriptor, ShaderSource, VertexBufferLayout, VertexState, VertexStepMode,
};

use storyboard_graphics::{
    buffer::{index::IndexBuffer, stream::StreamSlice},
    color::Srgba,
    component::{color::ShapeColor, texture::ComponentTexture, DrawState, RenderState},
    context::{DrawContext, RenderContext},
    pass::StoryboardRenderPass,
    pipeline::PipelineTargetDescriptor,
    shader::{RenderShader, RenderShaderDescriptor},
    texture::Texture2D,
};

pub type BoxShapeColor = ShapeColor<4>;

#[derive(Debug, Clone)]
pub struct BoxStyle {
    pub fill_color: BoxShapeColor,
    pub border_color: BoxShapeColor,

    pub border_thickness: f32,
    pub border_radius: ExtentUnit,

    pub opacity: f32,
    pub texture: Option<ComponentTexture>,
}

impl Default for BoxStyle {
    fn default() -> Self {
        Self {
            fill_color: ShapeColor::default(),
            border_color: ShapeColor::default(),

            border_thickness: 0.0,
            border_radius: ExtentUnit::default(),

            opacity: 1.0,
            texture: None,
        }
    }
}

pub struct BoxDrawState<'a> {
    pub pipeline: &'a RenderPipeline,
    pub quad_index_buffer: &'a IndexBuffer,

    pub texture: Option<Arc<Texture2D>>,

    pub quad: [BoxVertex; 4],
    pub instance: BoxInstance,
}

impl<'a> DrawState<'a> for BoxDrawState<'a> {
    fn prepare(
        &mut self,
        context: &mut DrawContext,
        depth: f32,
        _: &mut CommandEncoder,
        state_queue: &mut RenderStateQueue<'a>,
    ) {
        let quad = {
            let mut quad = self.quad;

            quad[0].position[2] = depth;
            quad[1].position[2] = depth;
            quad[2].position[2] = depth;
            quad[3].position[2] = depth;

            quad
        };

        let vertex = {
            let mut entry = context.stream_allocator.start_entry();
            entry.write(bytemuck::cast_slice(&quad));

            entry.finish()
        };

        let instance = {
            let mut entry = context.stream_allocator.start_entry();
            entry.write(bytemuck::bytes_of(&self.instance));

            entry.finish()
        };

        state_queue.push(BoxRenderState {
            pipeline: self.pipeline,
            quad_index_buffer: self.quad_index_buffer,

            vertex,
            instance,
            texture: self.texture.clone(),
        });
    }
}

#[derive(Debug)]
pub struct BoxRenderState<'a> {
    pub pipeline: &'a RenderPipeline,
    pub quad_index_buffer: &'a IndexBuffer,

    pub vertex: StreamSlice,
    pub instance: StreamSlice,
    pub texture: Option<Arc<Texture2D>>,
}

impl RenderState for BoxRenderState<'_> {
    fn render<'r>(&'r mut self, context: &'r RenderContext, pass: &mut StoryboardRenderPass<'r>) {
        pass.set_pipeline(self.pipeline);

        pass.set_vertex_buffer(0, context.stream_buffer.slice(&self.vertex));
        pass.set_vertex_buffer(1, context.stream_buffer.slice(&self.instance));

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

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct BoxVertex {
    pub position: [f32; 3],
    pub fill_color: Srgba<f32>,
    pub border_color: Srgba<f32>,
    pub tex_coord: [f32; 2],
    pub rect_coord: [f32; 2],
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct BoxInstance {
    pub rect: [f32; 2],
    pub border_radius: f32,
    pub border_thickness: f32,
}

pub fn init_box_shader(
    device: &Device,
    texture_bind_group_layout: &BindGroupLayout,
) -> RenderShader {
    RenderShader::init(
        device,
        ShaderSource::Wgsl(Cow::Borrowed(include_str!("box.wgsl"))),
        &RenderShaderDescriptor {
            label: Some("Box2D shader"),
        },
        &PipelineLayoutDescriptor {
            label: Some("Box2D shader pipeline layout"),
            bind_group_layouts: &[texture_bind_group_layout],
            push_constant_ranges: &[],
        },
    )
}

pub fn init_box_pipeline(
    device: &Device,
    shader: &RenderShader,
    pipeline_desc: PipelineTargetDescriptor,
) -> RenderPipeline {
    let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Box2D pipeline"),
        layout: Some(shader.pipeline_layout()),
        vertex: VertexState {
            module: shader.module(),
            entry_point: &"vs_main",
            buffers: &[VertexBufferLayout {
                array_stride: std::mem::size_of::<BoxVertex>() as BufferAddress,
                step_mode: VertexStepMode::Vertex,
                attributes: &vertex_attr_array![0 => Float32x3, 1 => Float32x4, 2 => Float32x4, 3 => Float32x2, 4 => Float32x2],
            },
            VertexBufferLayout {
                array_stride: std::mem::size_of::<BoxInstance>() as BufferAddress,
                step_mode: VertexStepMode::Instance,
                attributes: &vertex_attr_array![5 => Float32x2, 6 => Float32, 7 => Float32],
            }],
        },
        fragment: Some(FragmentState {
            module: shader.module(),
            entry_point: &"fs_main",
            targets: pipeline_desc.fragments_targets,
        }),
        depth_stencil: pipeline_desc.depth_stencil,
        primitive: PrimitiveState {
            topology: pipeline_desc.topology.unwrap_or(PrimitiveTopology::TriangleList),
            polygon_mode: pipeline_desc.polygon_mode,
            ..PrimitiveState::default()
        },
        multisample: pipeline_desc.multisample,
    });

    pipeline
}
