/*
 * Created on Sun Nov 14 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{borrow::Cow, sync::Arc};

use bytemuck::{Pod, Zeroable};
use storyboard_core::{
    component::color::ShapeColor,
    euclid::{Point2D, Point3D, Rect, Vector2D},
    graphics::buffer::stream::StreamRange,
    palette::LinSrgba,
    store::StoreResources,
    unit::{PixelUnit, RenderUnit},
    wgpu::{
        util::RenderEncoder, vertex_attr_array, BindGroupLayout, BlendState, ColorTargetState,
        ColorWrites, DepthStencilState, Device, FragmentState, IndexFormat, MultisampleState,
        PipelineLayout, PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology,
        RenderPipeline, RenderPipelineDescriptor, ShaderModule, ShaderModuleDescriptor,
        ShaderSource, VertexBufferLayout, VertexState, VertexStepMode, CommandEncoder,
    },
};

use crate::graphics::{
    context::{BackendContext, DrawContext, RenderContext},
    texture::RenderTexture2D, renderer::ComponentQueue,
};

use super::{
    common::{EmptyTextureResources, QuadIndexBufferResources},
    Component, Drawable,
};

#[derive(Debug)]
pub struct PrimitiveResources {
    pipeline: RenderPipeline,
}

impl StoreResources<BackendContext<'_>> for PrimitiveResources {
    fn initialize(ctx: &BackendContext) -> Self {
        let shader = init_primitive_shader(ctx.device);
        let pipeline_layout =
            init_primitive_pipeline_layout(ctx.device, ctx.textures.bind_group_layout());
        let pipeline = init_primitive_pipeline(
            ctx.device,
            &pipeline_layout,
            &shader,
            &[ColorTargetState {
                format: ctx.textures.framebuffer_texture_format(),
                blend: Some(BlendState::ALPHA_BLENDING),
                write_mask: ColorWrites::ALL,
            }],
            ctx.depth_stencil.map(Clone::clone),
        );

        Self { pipeline }
    }
}

#[derive(Debug, Clone)]
pub struct Triangle {
    pub bounds: Rect<f32, PixelUnit>,
    pub color: ShapeColor<3>,
    pub texture: Option<Arc<RenderTexture2D>>,
    pub texture_rect: Rect<f32, PixelUnit>,
}

impl Drawable for Triangle {
    fn prepare(&self, component_queue: &mut ComponentQueue, ctx: &mut DrawContext, _: &mut CommandEncoder, depth: f32) {
        component_queue.push(PrimitiveComponent::from_triangle(self, ctx, depth));
    }
}

#[derive(Debug, Clone)]
pub struct Rectangle {
    pub bounds: Rect<f32, PixelUnit>,
    pub color: ShapeColor<4>,
    pub texture: Option<Arc<RenderTexture2D>>,
    pub texture_rect: Rect<f32, PixelUnit>,
}

impl Drawable for Rectangle {
    fn prepare(&self, component_queue: &mut ComponentQueue, ctx: &mut DrawContext, _: &mut CommandEncoder, depth: f32) {
        component_queue.push(PrimitiveComponent::from_rectangle(self, ctx, depth));
    }
}

#[derive(Debug)]
pub struct PrimitiveComponent {
    pritmitive_type: PrimitiveType,
    texture: Option<Arc<RenderTexture2D>>,
    vertices_slice: StreamRange,
    instance_slice: StreamRange,
}

#[derive(Debug)]
pub enum PrimitiveType {
    Triangle,
    Rectangle,
}

impl PrimitiveComponent {
    pub fn from_triangle(triangle: &Triangle, ctx: &mut DrawContext, depth: f32) -> Self {
        let top_center = Point2D::new(
            triangle.bounds.origin.x + triangle.bounds.size.width / 2.0,
            triangle.bounds.origin.y,
        );
        let bottom_left = Point2D::new(
            triangle.bounds.origin.x,
            triangle.bounds.origin.y + triangle.bounds.size.height,
        );
        let bottom_right = Point2D::new(
            triangle.bounds.origin.x + triangle.bounds.size.width,
            triangle.bounds.origin.y + triangle.bounds.size.height,
        );

        // TODO
        let texture_origin = Vector2D::zero();

        let vertices_slice = ctx.vertex_stream.write_slice(bytemuck::bytes_of(&[
            PrimitiveVertex {
                position: ctx
                    .screen_matrix
                    .transform_point2d(top_center)
                    .unwrap()
                    .extend(depth),
                color: triangle.color[0],
                texture_coord: (texture_origin + top_center.to_vector()
                    - triangle.bounds.origin.to_vector())
                .to_point(),
            },
            PrimitiveVertex {
                position: ctx
                    .screen_matrix
                    .transform_point2d(bottom_left)
                    .unwrap()
                    .extend(depth),
                color: triangle.color[1],
                texture_coord: (texture_origin + bottom_left.to_vector()
                    - triangle.bounds.origin.to_vector())
                .to_point(),
            },
            PrimitiveVertex {
                position: ctx
                    .screen_matrix
                    .transform_point2d(bottom_right)
                    .unwrap()
                    .extend(depth),
                color: triangle.color[2],
                texture_coord: (texture_origin + bottom_right.to_vector()
                    - triangle.bounds.origin.to_vector())
                .to_point(),
            },
        ]));

        let instance_slice =
            ctx.vertex_stream
                .write_slice(bytemuck::bytes_of(&PrimitiveInstance {
                    texure_rect: triangle.texture_rect,
                }));

        Self {
            pritmitive_type: PrimitiveType::Triangle,
            texture: triangle.texture.clone(),
            vertices_slice,
            instance_slice,
        }
    }

    pub fn from_rectangle(rect: &Rectangle, ctx: &mut DrawContext, depth: f32) -> Self {
        let top_left = rect.bounds.origin;
        let top_right = Point2D::new(
            rect.bounds.origin.x + rect.bounds.size.width,
            rect.bounds.origin.y,
        );
        let bottom_left = Point2D::new(
            rect.bounds.origin.x,
            rect.bounds.origin.y + rect.bounds.size.height,
        );
        let bottom_right = rect.bounds.origin + rect.bounds.size;

        // TODO
        let texture_origin = Vector2D::zero();

        let vertices_slice = ctx.vertex_stream.write_slice(bytemuck::bytes_of(&[
            PrimitiveVertex {
                position: ctx
                    .screen_matrix
                    .transform_point2d(top_left)
                    .unwrap()
                    .extend(depth),
                color: rect.color[0],
                texture_coord: (texture_origin
                    + (top_left.to_vector() - rect.bounds.origin.to_vector()))
                .to_point(),
            },
            PrimitiveVertex {
                position: ctx
                    .screen_matrix
                    .transform_point2d(bottom_left)
                    .unwrap()
                    .extend(depth),
                color: rect.color[1],
                texture_coord: (texture_origin
                    + (bottom_left.to_vector() - rect.bounds.origin.to_vector()))
                .to_point(),
            },
            PrimitiveVertex {
                position: ctx
                    .screen_matrix
                    .transform_point2d(bottom_right)
                    .unwrap()
                    .extend(depth),
                color: rect.color[2],
                texture_coord: (texture_origin
                    + (bottom_right.to_vector() - rect.bounds.origin.to_vector()))
                .to_point(),
            },
            PrimitiveVertex {
                position: ctx
                    .screen_matrix
                    .transform_point2d(top_right)
                    .unwrap()
                    .extend(depth),
                color: rect.color[3],
                texture_coord: (texture_origin
                    + (top_right.to_vector() - rect.bounds.origin.to_vector()))
                .to_point(),
            },
        ]));

        let instance_slice =
            ctx.vertex_stream
                .write_slice(bytemuck::bytes_of(&PrimitiveInstance {
                    texure_rect: rect.texture_rect,
                }));

        Self {
            pritmitive_type: PrimitiveType::Rectangle,
            texture: rect.texture.clone(),
            vertices_slice,
            instance_slice,
        }
    }
}

impl Component for PrimitiveComponent {
    fn render<'rpass>(
        &'rpass self,
        ctx: &mut RenderContext<'rpass>,
        pass: &mut dyn RenderEncoder<'rpass>,
    ) {
        pass.set_pipeline(
            &ctx.resources
                .get::<PrimitiveResources>(&ctx.backend)
                .pipeline,
        );

        self.texture
            .as_deref()
            .or_else(|| {
                Some(
                    &ctx.resources
                        .get::<EmptyTextureResources>(&ctx.backend)
                        .empty_texture,
                )
            })
            .unwrap()
            .bind(0, pass);

        pass.set_vertex_buffer(0, ctx.vertex_stream.slice(self.vertices_slice.clone()));
        pass.set_vertex_buffer(1, ctx.vertex_stream.slice(self.instance_slice.clone()));

        match self.pritmitive_type {
            PrimitiveType::Triangle => {
                pass.draw(0..3, 0..1);
            }

            PrimitiveType::Rectangle => {
                pass.set_index_buffer(
                    ctx.resources
                        .get::<QuadIndexBufferResources>(&ctx.backend)
                        .quad_index_buffer
                        .slice(..),
                    IndexFormat::Uint16,
                );

                pass.draw_indexed(0..6, 0, 0..1);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct PrimitiveVertex {
    pub position: Point3D<f32, RenderUnit>,
    pub color: LinSrgba<f32>,
    pub texture_coord: Point2D<f32, PixelUnit>,
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct PrimitiveInstance {
    pub texure_rect: Rect<f32, PixelUnit>,
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
            buffers: &[
                VertexBufferLayout {
                    array_stride: std::mem::size_of::<PrimitiveVertex>() as u64,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &vertex_attr_array![0 => Float32x3, 1 => Float32x4, 2 => Float32x2],
                },
                VertexBufferLayout {
                    array_stride: std::mem::size_of::<PrimitiveInstance>() as u64,
                    step_mode: VertexStepMode::Instance,
                    attributes: &vertex_attr_array![3 => Float32x4],
                },
            ],
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
