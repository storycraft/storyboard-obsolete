/*
 * Created on Thu Oct 07 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

pub mod compositor;
pub mod rectangle;
pub mod triangle;

use std::borrow::Cow;

use storyboard::color::Srgba;

use storyboard::shader::RenderShaderDescriptor;
use storyboard::{
    component::{color::ShapeColor, texture::ComponentTexture},
    pipeline::PipelineTargetDescriptor,
    shader::RenderShader,
};

use bytemuck::{Pod, Zeroable};
use storyboard::wgpu::{BindGroupLayout, Device, FragmentState, PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor, ShaderSource, VertexBufferLayout, VertexState, VertexStepMode, vertex_attr_array};

pub type QuadShapeColor = ShapeColor<4>;

#[derive(Debug, Clone)]
pub struct PrimitiveStyle {
    pub fill_color: QuadShapeColor,
    pub opacity: f32,
    pub texture: Option<ComponentTexture>,
}

impl Default for PrimitiveStyle {
    fn default() -> Self {
        Self {
            fill_color: ShapeColor::default(),
            opacity: 1.0,
            texture: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct PrimitiveVertex {
    pub position: [f32; 3],
    pub color: Srgba<f32>,
    pub texure_coord: [f32; 2],
}

pub fn init_primitive_shader(
    device: &Device,
    texture_bind_group_layout: &BindGroupLayout,
) -> RenderShader {
    RenderShader::init(
        device,
        ShaderSource::Wgsl(Cow::Borrowed(include_str!("primitive.wgsl"))),
        &RenderShaderDescriptor {
            label: Some("Pritmitive shader"),
        },
        &PipelineLayoutDescriptor {
            label: Some("Primitive shader pipeline layout"),
            bind_group_layouts: &[texture_bind_group_layout],
            push_constant_ranges: &[],
        },
    )
}

pub fn init_primitive_pipeline(
    device: &Device,
    shader: &RenderShader,
    pipeline_desc: PipelineTargetDescriptor,
) -> RenderPipeline {
    let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Primitive pipeline"),
        layout: Some(shader.pipeline_layout()),
        vertex: VertexState {
            module: shader.module(),
            entry_point: &"vs_main",
            buffers: &[VertexBufferLayout {
                array_stride: std::mem::size_of::<PrimitiveVertex>() as u64,
                step_mode: VertexStepMode::Vertex,
                attributes: &vertex_attr_array![0 => Float32x3, 1 => Float32x4, 2 => Float32x2],
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
