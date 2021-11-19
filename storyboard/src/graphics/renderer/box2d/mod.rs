/*
 * Created on Sun Nov 14 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::borrow::Cow;

use bytemuck::{Pod, Zeroable};
use euclid::{SideOffsets2D, Size2D};
use palette::Srgba;
use wgpu::{
    vertex_attr_array, BindGroupLayout, BufferAddress, ColorTargetState, DepthStencilState, Device,
    FragmentState, MultisampleState, PipelineLayout, PipelineLayoutDescriptor, PrimitiveState,
    PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor, ShaderModule,
    ShaderModuleDescriptor, ShaderSource, VertexBufferLayout, VertexState, VertexStepMode,
};

use crate::{
    component::{color::ShapeColor, layout::texture::TextureLayout, DrawBox},
    graphics::{buffer::index::IndexBuffer, PixelUnit},
};

#[derive(Debug)]
pub struct Box2DPipelineData {
    pub pipeline: RenderPipeline,
    pub quad_index_buffer: IndexBuffer,
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

pub fn draw_box2d(
    draw_box: &DrawBox,
    depth: f32,
    fill_color: &ShapeColor<4>,
    border_color: &ShapeColor<4>,
    border_radius: f32,
    border_thickness: f32,
    texture_layout: &TextureLayout,
    texture_size: &Size2D<u32, PixelUnit>,
) -> ([BoxVertex; 4], BoxInstance) {
    let expanded_rect = {
        if border_thickness != 0.0 {
            draw_box
                .rect
                .outer_rect(SideOffsets2D::new_all_same(border_thickness))
        } else {
            draw_box.rect
        }
    };

    let quad = draw_box.get_quad_2d(&expanded_rect);

    let texture_coords = {
        let mut space = draw_box.into_space();
        if border_thickness != 0.0 {
            space.parent.size.width += border_thickness * 2.0;
            space.parent.size.height += border_thickness * 2.0;
        }

        texture_layout.texture_coord_quad(&space, texture_size)
    };

    let quad = [
        BoxVertex {
            position: quad[0].extend(depth).to_array(),
            fill_color: fill_color[0].into_encoding(),
            border_color: border_color[0].into_encoding(),
            tex_coord: texture_coords[0].to_array(),
            rect_coord: [-border_thickness, -border_thickness],
        },
        BoxVertex {
            position: quad[1].extend(depth).to_array(),
            fill_color: fill_color[1].into_encoding(),
            border_color: border_color[1].into_encoding(),
            tex_coord: texture_coords[1].to_array(),
            rect_coord: [
                -border_thickness,
                draw_box.rect.size.height + border_thickness,
            ],
        },
        BoxVertex {
            position: quad[2].extend(depth).to_array(),
            fill_color: fill_color[2].into_encoding(),
            border_color: border_color[2].into_encoding(),
            tex_coord: texture_coords[2].to_array(),
            rect_coord: [
                draw_box.rect.size.width + border_thickness,
                draw_box.rect.size.height + border_thickness,
            ],
        },
        BoxVertex {
            position: quad[3].extend(depth).to_array(),
            fill_color: fill_color[3].into_encoding(),
            border_color: border_color[3].into_encoding(),
            tex_coord: texture_coords[3].to_array(),
            rect_coord: [
                draw_box.rect.size.width + border_thickness,
                -border_thickness,
            ],
        },
    ];

    let instance = BoxInstance {
        rect: [draw_box.rect.size.width, draw_box.rect.size.height],
        border_radius,
        border_thickness,
    };

    (quad, instance)
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
    });

    pipeline
}
