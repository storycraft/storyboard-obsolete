/*
 * Created on Thu Oct 07 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

pub mod compositor;

pub extern crate lyon;

use std::borrow::Cow;

use lyon::lyon_tessellation::VertexBuffers;
use storyboard_graphics::color::LinSrgba;
use storyboard_graphics::math::Rect;

use storyboard_graphics::shader::RenderShaderDescriptor;
use storyboard_graphics::unit::PixelUnit;
use storyboard_graphics::{pipeline::PipelineTargetDescriptor, shader::RenderShader};

use bytemuck::{Pod, Zeroable};
use storyboard_graphics::wgpu::{BufferAddress, Device, FragmentState, PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor, ShaderSource, VertexBufferLayout, VertexState, VertexStepMode, vertex_attr_array};

pub type Path = VertexBuffers<PathVertex, u16>;

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

pub fn init_path_shader(device: &Device) -> RenderShader {
    RenderShader::init(
        device,
        ShaderSource::Wgsl(Cow::Borrowed(include_str!("path.wgsl"))),
        &RenderShaderDescriptor {
            label: Some("Path shader"),
        },
        &PipelineLayoutDescriptor {
            label: Some("Path shader pipeline layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        },
    )
}

pub fn init_path_pipeline(
    device: &Device,
    shader: &RenderShader,
    pipeline_desc: PipelineTargetDescriptor,
) -> RenderPipeline {
    let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Path pipeline"),
        layout: Some(shader.pipeline_layout()),
        vertex: VertexState {
            module: shader.module(),
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
