/*
 * Created on Sat May 14 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{borrow::Cow, sync::Arc};

use bytemuck::{Pod, Zeroable};
use storyboard_core::{
    component::color::ShapeColor,
    euclid::{Point2D, Point3D, Rect},
    graphics::buffer::stream::StreamRange,
    palette::LinSrgba,
    store::StoreResources,
    unit::{PixelUnit, RenderUnit},
    wgpu::{
        util::RenderEncoder, vertex_attr_array, BindGroupLayout, BufferAddress, ColorTargetState,
        DepthStencilState, Device, FragmentState, IndexFormat, MultisampleState, PipelineLayout,
        PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology, RenderPipeline,
        RenderPipelineDescriptor, ShaderModule, ShaderModuleDescriptor, ShaderSource,
        VertexBufferLayout, VertexState, VertexStepMode, BlendState, ColorWrites,
    },
};

use crate::graphics::{
    context::{BackendContext, DrawContext, RenderContext},
    renderer::ComponentQueue,
    texture::RenderTexture2D,
};

use super::{
    common::{EmptyTextureResources, QuadIndexBufferResources},
    Component, Drawable,
};

#[derive(Debug)]
pub struct Box2DResources {
    pipeline: RenderPipeline,
}

impl StoreResources<BackendContext<'_>> for Box2DResources {
    fn initialize(ctx: &BackendContext<'_>) -> Self {
        let shader = init_box_shader(ctx.device);
        let pipeline_layout =
            init_box_pipeline_layout(ctx.device, ctx.textures.bind_group_layout());
        let pipeline = init_box_pipeline(
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

#[derive(Debug)]
pub struct Box2D {
    pub bounds: Rect<f32, PixelUnit>,

    pub texture: Option<Arc<RenderTexture2D>>,
    pub texture_rect: Rect<f32, PixelUnit>,

    pub fill_color: ShapeColor<4>,
    pub border_color: ShapeColor<4>,

    pub style: Box2DStyle,
}

impl Drawable for Box2D {
    fn prepare(&self, component_queue: &mut ComponentQueue, ctx: &mut DrawContext, depth: f32) {
        component_queue.push(Box2DComponent::from_box2d(self, ctx, depth));
    }
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Box2DStyle {
    pub border_radius: [f32; 4],
    pub border_thickness: f32,

    pub glow_radius: f32,
    pub glow_color: LinSrgba<f32>,

    pub shadow_offset: Point2D<f32, PixelUnit>,
    pub shadow_radius: f32,
    pub shadow_color: LinSrgba<f32>,
}

#[derive(Debug)]
pub struct Box2DComponent {
    texture: Option<Arc<RenderTexture2D>>,

    vertices_slice: StreamRange,
    instance_slice: StreamRange,
}

impl Box2DComponent {
    pub fn from_box2d(box2d: &Box2D, ctx: &mut DrawContext, depth: f32) -> Self {
        let top_left = box2d.bounds.origin;

        let top_right = Point2D::new(
            box2d.bounds.origin.x + box2d.bounds.size.width,
            box2d.bounds.origin.y,
        );
        let bottom_left = Point2D::new(
            box2d.bounds.origin.x,
            box2d.bounds.origin.y + box2d.bounds.size.height,
        );
        let bottom_right = box2d.bounds.origin + box2d.bounds.size;

        let vertices_slice = ctx.vertex_stream.write_slice(bytemuck::bytes_of(&[
            BoxVertex {
                position: ctx
                    .screen_matrix
                    .transform_point2d(top_left)
                    .unwrap()
                    .extend(depth),
                fill_color: box2d.fill_color[0],
                border_color: box2d.border_color[0],
                rect_coord: top_left,
            },
            BoxVertex {
                position: ctx
                    .screen_matrix
                    .transform_point2d(bottom_left)
                    .unwrap()
                    .extend(depth),
                fill_color: box2d.fill_color[1],
                border_color: box2d.border_color[1],
                rect_coord: bottom_left,
            },
            BoxVertex {
                position: ctx
                    .screen_matrix
                    .transform_point2d(bottom_right)
                    .unwrap()
                    .extend(depth),
                fill_color: box2d.fill_color[2],
                border_color: box2d.border_color[2],
                rect_coord: bottom_right,
            },
            BoxVertex {
                position: ctx
                    .screen_matrix
                    .transform_point2d(top_right)
                    .unwrap()
                    .extend(depth),
                fill_color: box2d.fill_color[3],
                border_color: box2d.border_color[3],
                rect_coord: top_right,
            },
        ]));

        let instance_slice = ctx
            .vertex_stream
            .write_slice(bytemuck::bytes_of(&BoxInstance {
                rect: box2d.bounds,

                // TODO:: Fix texture_rect
                texture_rect: box2d.texture_rect,

                style: box2d.style,
            }));

        Self {
            texture: box2d.texture.clone(),
            vertices_slice,
            instance_slice,
        }
    }
}

impl Component for Box2DComponent {
    fn render<'rpass>(
        &'rpass self,
        ctx: &mut RenderContext<'rpass>,
        pass: &mut dyn RenderEncoder<'rpass>,
    ) {
        pass.set_pipeline(&ctx.resources.get::<Box2DResources>(&ctx.backend).pipeline);

        pass.set_vertex_buffer(0, ctx.vertex_stream.slice(self.vertices_slice.clone()));
        pass.set_vertex_buffer(1, ctx.vertex_stream.slice(self.instance_slice.clone()));

        pass.set_index_buffer(
            ctx.resources
                .get::<QuadIndexBufferResources>(&ctx.backend)
                .quad_index_buffer
                .slice(..),
            IndexFormat::Uint16,
        );

        self.texture
            .as_deref()
            .or_else(|| {
                Some(
                    &ctx.resources
                        .get::<EmptyTextureResources>(&ctx.backend)
                        .empty_texture,
                )
            }).unwrap()
            .bind(0, pass);

        pass.draw_indexed(0..6, 0, 0..1);
    }
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct BoxVertex {
    pub position: Point3D<f32, RenderUnit>,

    pub fill_color: LinSrgba<f32>,
    pub border_color: LinSrgba<f32>,

    pub rect_coord: Point2D<f32, PixelUnit>,
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct BoxInstance {
    pub rect: Rect<f32, PixelUnit>,
    pub texture_rect: Rect<f32, PixelUnit>,

    pub style: Box2DStyle,
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
            buffers: &[
                VertexBufferLayout {
                    array_stride: std::mem::size_of::<BoxVertex>() as BufferAddress,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &vertex_attr_array![
                        0 => Float32x3,
                        1 => Float32x4,
                        2 => Float32x4,
                        3 => Float32x2
                    ],
                },
                VertexBufferLayout {
                    array_stride: std::mem::size_of::<BoxInstance>() as BufferAddress,
                    step_mode: VertexStepMode::Instance,
                    attributes: &vertex_attr_array![
                        4 => Float32x4,
                        5 => Float32x4,
                        6 => Float32x4,
                        7 => Float32,
                        8 => Float32,
                        9 => Float32x4,
                        10 => Float32x2,
                        11 => Float32,
                        12 => Float32x4
                    ],
                },
            ],
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
