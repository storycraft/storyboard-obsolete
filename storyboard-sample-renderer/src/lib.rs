/*
 * Created on Sat Apr 30 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

pub mod render;

use std::borrow::Cow;

use bytemuck::{Pod, Zeroable};
use palette::LinSrgba;
use storyboard_core::wgpu::{
    vertex_attr_array, BindGroupLayout, BufferAddress, ColorTargetState, DepthStencilState, Device,
    FragmentState, MultisampleState, PipelineLayout, PipelineLayoutDescriptor, PrimitiveState,
    PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor, ShaderModule,
    ShaderModuleDescriptor, ShaderSource, VertexBufferLayout, VertexState, VertexStepMode,
};


#[derive(Debug, Clone)]
pub struct Box2D {
    pub fill_color: [LinSrgba<f32>; 4],
    pub border_color: [LinSrgba<f32>; 4],

    pub border_thickness: f32,
    pub border_radius: f32,

    pub texture: Option<()>,
}

impl Default for Box2D {
    fn default() -> Self {
        Self {
            fill_color: [LinSrgba::new(1.0, 1.0, 1.0, 1.0); 4],
            border_color: [LinSrgba::new(1.0, 1.0, 1.0, 1.0); 4],

            border_thickness: 0.0,
            border_radius: 0.0,

            texture: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct BoxVertex {
    pub position: [f32; 3],
    pub fill_color: LinSrgba<f32>,
    pub border_color: LinSrgba<f32>,
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

pub fn init_box_shader(device: &Device) -> ShaderModule {
    device.create_shader_module(&ShaderModuleDescriptor {
        label: Some("Box2D shader"),
        source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("box.wgsl"))),
    })
}

pub fn init_box_pipeline_layout(
    device: &Device,
    texture_bind_group_layout: &BindGroupLayout,
) -> PipelineLayout {
    device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Box2D shader pipeline layout"),
        bind_group_layouts: &[texture_bind_group_layout],
        push_constant_ranges: &[],
    })
}

pub fn init_box_pipeline(
    device: &Device,
    pipeline_layout: &PipelineLayout,
    shader: &ShaderModule,
    fragment_targets: &[ColorTargetState],
    depth_stencil: Option<DepthStencilState>,
) -> RenderPipeline {
    let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Box2D pipeline"),
        layout: Some(pipeline_layout),
        vertex: VertexState {
            module: shader,
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
        multiview: None,
    });

    pipeline
}
