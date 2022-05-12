/*
 * Created on Sun Nov 14 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{borrow::Cow, sync::Arc};

use bytemuck::{Pod, Zeroable};
use storyboard_core::{
    component::color::ShapeColor,
    euclid::{Point2D, Point3D},
    graphics::buffer::stream::StreamRange,
    palette::LinSrgba,
    unit::{RenderUnit, TextureUnit},
    wgpu::{
        util::{BufferInitDescriptor, DeviceExt, RenderEncoder},
        vertex_attr_array, BindGroupLayout, Buffer, BufferUsages, ColorTargetState,
        DepthStencilState, Device, FragmentState, IndexFormat, MultisampleState, PipelineLayout,
        PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology, RenderPipeline,
        RenderPipelineDescriptor, ShaderModule, ShaderModuleDescriptor, ShaderSource,
        VertexBufferLayout, VertexState, VertexStepMode,
    },
};

use crate::graphics::{
    context::{DrawContext, RenderContext},
    texture::Texture2D,
};

use super::ComponentCompositor;

#[derive(Debug)]
pub struct PrimitiveCompositor {
    pipeline: RenderPipeline,

    quad_index_buffer: Buffer,

    default_texture: Texture2D,
}

impl PrimitiveCompositor {
    pub fn init(
        device: &Device,
        texture_2d_bind_group_layout: &BindGroupLayout,
        fragment_targets: &[ColorTargetState],
        depth_stencil: Option<DepthStencilState>,
        default_texture: Texture2D,
    ) -> Self {
        let shader = init_primitive_shader(device);
        let pipeline_layout = init_primitive_pipeline_layout(device, texture_2d_bind_group_layout);
        let pipeline = init_primitive_pipeline(
            device,
            &pipeline_layout,
            &shader,
            fragment_targets,
            depth_stencil,
        );

        let quad_index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("PrimitiveCompositor quad index buffer"),
            contents: bytemuck::cast_slice(&[0_u16, 1, 2, 0, 2, 3]),
            usage: BufferUsages::INDEX | BufferUsages::MAP_WRITE,
        });

        Self {
            pipeline,
            quad_index_buffer,
            default_texture,
        }
    }
}

impl ComponentCompositor for PrimitiveCompositor {
    type Component = Primitive;
    type Prepared = PreparedPrimitive;

    fn draw(
        &self,
        ctx: &mut DrawContext,
        component: &Self::Component,
        depth: f32,
    ) -> Self::Prepared {
        match component {
            Primitive::Triangle(triangle) => {
                let texture = triangle.texture.as_deref().unwrap_or(&self.default_texture);

                let vertices_slice = {
                    let mut writer = ctx.vertex_stream.next_writer();

                    writer.write(bytemuck::bytes_of(&[
                        PrimitiveVertex {
                            position: triangle.points[0].extend(depth),
                            color: triangle.color[0],
                            texure_coord: texture
                                .view()
                                .into_view_coord(triangle.texture_coords[0]),
                        },
                        PrimitiveVertex {
                            position: triangle.points[1].extend(depth),
                            color: triangle.color[1],
                            texure_coord: texture
                                .view()
                                .into_view_coord(triangle.texture_coords[1]),
                        },
                        PrimitiveVertex {
                            position: triangle.points[2].extend(depth),
                            color: triangle.color[2],
                            texure_coord: texture
                                .view()
                                .into_view_coord(triangle.texture_coords[2]),
                        },
                    ]));

                    writer.finish()
                };

                PreparedPrimitive::Triangle(Prepared {
                    texture: triangle.texture.clone(),
                    vertices_slice,
                })
            }

            Primitive::Quad(quad) => {
                let texture = quad.texture.as_deref().unwrap_or(&self.default_texture);

                let vertices_slice = {
                    let mut writer = ctx.vertex_stream.next_writer();

                    writer.write(bytemuck::bytes_of(&[
                        PrimitiveVertex {
                            position: quad.points[0].extend(depth),
                            color: quad.color[0],
                            texure_coord: texture
                                .view()
                                .into_view_coord(quad.texture_coords[0]),
                        },
                        PrimitiveVertex {
                            position: quad.points[1].extend(depth),
                            color: quad.color[1],
                            texure_coord: texture
                                .view()
                                .into_view_coord(quad.texture_coords[1]),
                        },
                        PrimitiveVertex {
                            position: quad.points[2].extend(depth),
                            color: quad.color[2],
                            texure_coord: texture
                                .view()
                                .into_view_coord(quad.texture_coords[2]),
                        },
                        PrimitiveVertex {
                            position: quad.points[3].extend(depth),
                            color: quad.color[3],
                            texure_coord: texture
                                .view()
                                .into_view_coord(quad.texture_coords[3]),
                        },
                    ]));

                    writer.finish()
                };

                PreparedPrimitive::Quad(Prepared {
                    texture: quad.texture.clone(),
                    vertices_slice,
                })
            }
        }
    }

    fn render<'rpass>(
        &'rpass self,
        ctx: &RenderContext<'rpass>,
        pass: &mut impl RenderEncoder<'rpass>,
        prepared: &'rpass Self::Prepared,
    ) {
        pass.set_pipeline(&self.pipeline);

        match prepared {
            PreparedPrimitive::Triangle(triangle) => {
                triangle
                    .texture
                    .as_deref()
                    .unwrap_or(&self.default_texture)
                    .bind(0, pass);

                pass.set_vertex_buffer(0, ctx.vertex_stream.slice(triangle.vertices_slice.clone()));

                pass.draw(0..3, 0..1);
            }

            PreparedPrimitive::Quad(quad) => {
                quad.texture
                    .as_deref()
                    .unwrap_or(&self.default_texture)
                    .bind(0, pass);

                pass.set_vertex_buffer(0, ctx.vertex_stream.slice(quad.vertices_slice.clone()));
                pass.set_index_buffer(self.quad_index_buffer.slice(..), IndexFormat::Uint16);

                pass.draw_indexed(0..6, 0, 0..1);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Triangle {
    pub points: [Point2D<f32, RenderUnit>; 3],
    pub color: ShapeColor<3>,
    pub texture: Option<Arc<Texture2D>>,
    pub texture_coords: [Point2D<f32, TextureUnit>; 3],
}

#[derive(Debug, Clone)]
pub struct Quad {
    pub points: [Point2D<f32, RenderUnit>; 4],
    pub color: ShapeColor<4>,
    pub texture: Option<Arc<Texture2D>>,
    pub texture_coords: [Point2D<f32, TextureUnit>; 4],
}

#[derive(Debug)]
pub enum Primitive {
    Triangle(Triangle),
    Quad(Quad),
}

impl From<Triangle> for Primitive {
    fn from(triangle: Triangle) -> Self {
        Self::Triangle(triangle)
    }
}

impl From<Quad> for Primitive {
    fn from(quad: Quad) -> Self {
        Self::Quad(quad)
    }
}

#[derive(Debug)]
pub struct Prepared {
    texture: Option<Arc<Texture2D>>,
    vertices_slice: StreamRange,
}

#[derive(Debug)]
pub enum PreparedPrimitive {
    Triangle(Prepared),
    Quad(Prepared),
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct PrimitiveVertex {
    pub position: Point3D<f32, RenderUnit>,
    pub color: LinSrgba<f32>,
    pub texure_coord: Point2D<f32, TextureUnit>,
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
        multiview: None,
    })
}
