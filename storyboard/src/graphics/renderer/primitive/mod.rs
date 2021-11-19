/*
 * Created on Sun Nov 14 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::borrow::Cow;

use bytemuck::{Pod, Zeroable};
use euclid::Size2D;
use palette::{Mix, Srgba};
use wgpu::{
    vertex_attr_array, BindGroupLayout, ColorTargetState, DepthStencilState, Device, FragmentState,
    MultisampleState, PipelineLayout, PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology,
    RenderPipeline, RenderPipelineDescriptor, ShaderModule, ShaderModuleDescriptor, ShaderSource,
    VertexBufferLayout, VertexState, VertexStepMode,
};

use crate::{
    component::{color::ShapeColor, layout::texture::TextureLayout, DrawBox},
    graphics::PixelUnit,
};

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct PrimitiveVertex {
    pub position: [f32; 3],
    pub color: Srgba<f32>,
    pub texure_coord: [f32; 2],
}

pub fn draw_triangle(
    draw_box: &DrawBox,
    depth: f32,
    fill_color: &ShapeColor<4>,
    texture_layout: &TextureLayout,
    texture_size: &Size2D<u32, PixelUnit>,
) -> [PrimitiveVertex; 3] {
    let quad = draw_box.get_quad_2d(&draw_box.rect);

    let texture_coords = texture_layout.texture_coord_quad(&draw_box.into_space(), texture_size);

    let top_color = {
        let left_top = fill_color[0];
        let right_top = fill_color[3];

        if left_top != right_top {
            left_top.mix(&right_top, 0.5)
        } else {
            left_top
        }
    };

    [
        PrimitiveVertex {
            position: quad[1].extend(depth).to_array(),
            color: fill_color[1].into_encoding(),
            texure_coord: texture_coords[1].to_array(),
        },
        PrimitiveVertex {
            position: quad[0].lerp(quad[3], 0.5).extend(depth).to_array(),
            color: top_color.into_encoding(),
            texure_coord: texture_coords[0].lerp(texture_coords[3], 0.5).to_array(),
        },
        PrimitiveVertex {
            position: quad[2].extend(depth).to_array(),
            color: fill_color[2].into_encoding(),
            texure_coord: texture_coords[2].to_array(),
        },
    ]
}

pub fn draw_rect(
    draw_box: &DrawBox,
    depth: f32,
    fill_color: &ShapeColor<4>,
    texture_layout: &TextureLayout,
    texture_size: &Size2D<u32, PixelUnit>,
) -> [PrimitiveVertex; 4] {
    let quad = draw_box.get_quad_2d(&draw_box.rect);

    let texture_coords = texture_layout.texture_coord_quad(&draw_box.into_space(), texture_size);

    [
        PrimitiveVertex {
            position: quad[0].extend(depth).to_array(),
            color: fill_color[0].into_encoding(),
            texure_coord: texture_coords[0].to_array(),
        },
        PrimitiveVertex {
            position: quad[1].extend(depth).to_array(),
            color: fill_color[1].into_encoding(),
            texure_coord: texture_coords[1].to_array(),
        },
        PrimitiveVertex {
            position: quad[2].extend(depth).to_array(),
            color: fill_color[2].into_encoding(),
            texure_coord: texture_coords[2].to_array(),
        },
        PrimitiveVertex {
            position: quad[3].extend(depth).to_array(),
            color: fill_color[3].into_encoding(),
            texure_coord: texture_coords[3].to_array(),
        },
    ]
}

pub fn init_primitive_shader(device: &Device) -> ShaderModule {
    device.create_shader_module(&ShaderModuleDescriptor {
        label: Some("Primitive shader"),
        source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("primitive.wgsl"))),
    })
}

pub fn init_primitive_pipeline_layout(
    device: &Device,
    texture_bind_group_layout: &BindGroupLayout,
) -> PipelineLayout {
    device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Primitive shader pipeline layout"),
        bind_group_layouts: &[texture_bind_group_layout],
        push_constant_ranges: &[],
    })
}

pub fn init_primitive_pipeline(
    device: &Device,
    pipeline_layout: &PipelineLayout,
    shader: &ShaderModule,
    fragment_targets: &[ColorTargetState],
    depth_stencil: Option<DepthStencilState>,
) -> RenderPipeline {
    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Primitive pipeline"),
        layout: Some(pipeline_layout),
        vertex: VertexState {
            module: shader,
            entry_point: &"vs_main",
            buffers: &[VertexBufferLayout {
                array_stride: std::mem::size_of::<PrimitiveVertex>() as u64,
                step_mode: VertexStepMode::Vertex,
                attributes: &vertex_attr_array![0 => Float32x3, 1 => Float32x4, 2 => Float32x2],
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
