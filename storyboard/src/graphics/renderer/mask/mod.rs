/*
 * Created on Fri Nov 26 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::borrow::Cow;

use bytemuck::{Pod, Zeroable};
use euclid::Size2D;
use palette::{LinSrgba, Mix};
use wgpu::{
    vertex_attr_array, BindGroupLayout, ColorTargetState, DepthStencilState, Device, FragmentState,
    MultisampleState, PipelineLayout, PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology,
    RenderPipeline, RenderPipelineDescriptor, ShaderModule, ShaderModuleDescriptor, ShaderSource,
    VertexBufferLayout, VertexState, VertexStepMode,
};

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct MaskingVertex {
    pub position: [f32; 3],
    pub color: LinSrgba<f32>,
    pub texure_coord: [f32; 2],
    pub mask_texure_coord: [f32; 2],
}

pub fn init_mask_shader(device: &Device) -> ShaderModule {
    device.create_shader_module(&ShaderModuleDescriptor {
        label: Some("Masking shader"),
        source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("mask.wgsl"))),
    })
}

pub fn init_mask_pipeline_layout(
    device: &Device,
    texture_bind_group_layout: &BindGroupLayout,
) -> PipelineLayout {
    device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Masking shader pipeline layout"),
        bind_group_layouts: &[texture_bind_group_layout, texture_bind_group_layout],
        push_constant_ranges: &[],
    })
}

pub fn init_mask_pipeline(
    device: &Device,
    pipeline_layout: &PipelineLayout,
    shader: &ShaderModule,
    fragment_targets: &[ColorTargetState],
    depth_stencil: Option<DepthStencilState>,
) -> RenderPipeline {
    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Masking pipeline"),
        layout: Some(pipeline_layout),
        vertex: VertexState {
            module: shader,
            entry_point: &"vs_main",
            buffers: &[VertexBufferLayout {
                array_stride: std::mem::size_of::<MaskingVertex>() as u64,
                step_mode: VertexStepMode::Vertex,
                attributes: &vertex_attr_array![0 => Float32x3, 1 => Float32x4, 2 => Float32x2, 3 => Float32x2],
            }],
        },
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            ..PrimitiveState::default()
        },
        depth_stencil,
        multisample: MultisampleState::default(),
        fragment: Some(FragmentState {
            module: shader,
            entry_point: &"fs_main",
            targets: fragment_targets,
        }),
    })
}
