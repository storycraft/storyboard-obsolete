/*
 * Created on Fri Oct 15 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

pub mod brush;
pub mod compositor;
pub mod font;
pub mod layout;

pub use font_kit;

use bytemuck::{Pod, Zeroable};

use std::borrow::Cow;

use storyboard::{color::Srgba, component::color::ShapeColor, pipeline::PipelineTargetDescriptor, shader::{RenderShader, RenderShaderDescriptor}, wgpu::{BindGroupLayout, Device, FragmentState, PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor, ShaderSource, VertexBufferLayout, VertexState, VertexStepMode, vertex_attr_array}};

#[derive(Debug, Clone)]
pub struct TextStyle {
    pub color: ShapeColor<4>,
}

impl TextStyle {
    pub fn new() -> Self {
        Self {
            color: ShapeColor::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct TextVertex {
    pub position: [f32; 3],
    pub color: Srgba<f32>,
    pub texure_coord: [f32; 2],
}

pub fn init_text_shader(
    device: &Device,
    texture_bind_group_layout: &BindGroupLayout,
) -> RenderShader {
    RenderShader::init(
        device,
        ShaderSource::Wgsl(Cow::Borrowed(include_str!("text.wgsl"))),
        &RenderShaderDescriptor {
            label: Some("Text shader"),
        },
        &PipelineLayoutDescriptor {
            label: Some("Text shader pipeline layout"),
            bind_group_layouts: &[texture_bind_group_layout],
            push_constant_ranges: &[],
        },
    )
}

pub fn init_text_pipeline(
    device: &Device,
    shader: &RenderShader,
    pipeline_desc: PipelineTargetDescriptor,
) -> RenderPipeline {
    let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Text pipeline"),
        layout: Some(shader.pipeline_layout()),
        vertex: VertexState {
            module: shader.module(),
            entry_point: &"vs_main",
            buffers: &[VertexBufferLayout {
                array_stride: std::mem::size_of::<TextVertex>() as u64,
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
