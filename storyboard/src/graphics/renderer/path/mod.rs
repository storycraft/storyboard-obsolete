/*
 * Created on Fri Nov 19 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::borrow::Cow;

use bytemuck::{Pod, Zeroable};
use euclid::Rect;
use lyon::path::Path;
use palette::LinSrgba;
use wgpu::{
    vertex_attr_array, BufferAddress, ColorTargetState, DepthStencilState, Device, FragmentState,
    MultisampleState, PipelineLayout, PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology,
    RenderPipeline, RenderPipelineDescriptor, ShaderModule, ShaderModuleDescriptor, ShaderSource,
    VertexBufferLayout, VertexState, VertexStepMode,
};

use crate::graphics::PixelUnit;

#[derive(Debug, Clone)]
pub struct ScalablePath {
    pub path: Path,
    pub rect: Rect<f32, PixelUnit>,
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct PathVertex {
    pub position: [f32; 3],
    pub color: LinSrgba<f32>,
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct PathInstance {
    pub matrix: [f32; 16],
}

pub fn init_path_shader(device: &Device) -> ShaderModule {
    device.create_shader_module(&ShaderModuleDescriptor {
        label: Some("Path shader"),
        source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("path.wgsl"))),
    })
}

pub fn init_path_pipeline_layout(device: &Device) -> PipelineLayout {
    device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Path shader pipeline layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    })
}

pub fn init_path_pipeline(
    device: &Device,
    pipeline_layout: &PipelineLayout,
    shader: &ShaderModule,
    fragment_targets: &[ColorTargetState],
    depth_stencil: Option<DepthStencilState>,
) -> RenderPipeline {
    let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Path pipeline"),
        layout: Some(pipeline_layout),
        vertex: VertexState {
            module: shader,
            entry_point: &"vs_main",
            buffers: &[VertexBufferLayout {
                array_stride: std::mem::size_of::<PathVertex>() as BufferAddress,
                step_mode: VertexStepMode::Vertex,
                attributes: &vertex_attr_array![0 => Float32x3, 1 => Float32x4],
            },
            VertexBufferLayout {
                array_stride: std::mem::size_of::<PathInstance>() as BufferAddress,
                step_mode: VertexStepMode::Instance,
                attributes: &vertex_attr_array![2 => Float32x4, 3 => Float32x4, 4 => Float32x4, 5 => Float32x4],
            }],
        },
        fragment: Some(FragmentState {
            module: shader,
            entry_point: &"fs_main",
            targets: fragment_targets,
        }),
        depth_stencil,
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            ..PrimitiveState::default()
        },
        multisample: MultisampleState::default(),
    });

    pipeline
}
