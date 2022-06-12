/*
 * Created on Wed Jun 08 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{borrow::Cow, sync::Arc};

use bytemuck::{Pod, Zeroable};
use storyboard::{core::{
    component::color::ShapeColor,
    euclid::{Point2D, Point3D, Rect},
    graphics::{
        buffer::stream::StreamRange,
        component::{Component, Drawable},
        renderer::{
            context::{BackendContext, DrawContext, RenderContext},
            ComponentQueue,
        },
    },
    palette::LinSrgba,
    store::{Store, StoreResources},
    unit::{PixelUnit, RenderUnit, TextureUnit},
    wgpu::{
        util::RenderEncoder, vertex_attr_array, BindGroupLayout, BlendState, ColorTargetState,
        ColorWrites, CommandEncoder, DepthStencilState, Device, FragmentState, MultisampleState,
        PipelineLayout, PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology,
        RenderPipeline, RenderPipelineDescriptor, ShaderModule, ShaderModuleDescriptor,
        ShaderSource, VertexBufferLayout, VertexState, VertexStepMode,
    },
}, graphics::component::common::QuadIndexBufferResources};

use storyboard::{
    graphics::texture::{data::TextureData, RenderTexture2D},
    math::RectExt,
};

#[derive(Debug)]
pub struct TextResources {
    pub pipeline: RenderPipeline,
}

impl StoreResources<BackendContext<'_>> for TextResources {
    fn initialize(store: &Store<BackendContext>, ctx: &BackendContext) -> Self {
        let textures = store.get::<TextureData>(ctx);

        let shader = init_glyph_shader(ctx.device);
        let pipeline_layout = init_glyph_pipeline_layout(ctx.device, textures.bind_group_layout());

        let pipeline = init_glyph_pipeline(
            ctx.device,
            &pipeline_layout,
            &shader,
            &[ColorTargetState {
                format: ctx.screen_format,
                blend: Some(BlendState::ALPHA_BLENDING),
                write_mask: ColorWrites::ALL,
            }],
            Some(DepthStencilState {
                depth_write_enabled: false,
                ..ctx.depth_stencil.clone()
            }),
        );

        Self { pipeline }
    }
}

#[derive(Debug)]
pub struct Glyph {
    pub position: Point2D<f32, PixelUnit>,
    pub color: ShapeColor<4>,
    pub texture: Arc<RenderTexture2D>,
}

impl Drawable for Glyph {
    fn prepare(
        &self,
        component_queue: &mut ComponentQueue,
        ctx: &mut DrawContext,
        _: &mut CommandEncoder,
        depth: f32,
    ) {
        component_queue.push_transparent(GlyphComponent::from_glyph(self, ctx, depth));
    }
}

#[derive(Debug)]
pub struct GlyphComponent {
    texture: Arc<RenderTexture2D>,
    vertices_slice: StreamRange,
    instance_slice: StreamRange,
}

impl GlyphComponent {
    pub fn from_glyph(glyph: &Glyph, ctx: &mut DrawContext, depth: f32) -> Self {
        let coords = Rect::new(glyph.position, glyph.texture.view().size().cast()).into_coords();

        let vertices_slice = ctx.vertex_stream.write_slice(bytemuck::bytes_of(&[
            GlyphVertex {
                position: ctx
                    .screen_matrix
                    .transform_point2d(coords[0])
                    .unwrap()
                    .extend(depth),
                color: glyph.color[0],
                texture_coord: Point2D::new(0.0, 0.0),
            },
            GlyphVertex {
                position: ctx
                    .screen_matrix
                    .transform_point2d(coords[1])
                    .unwrap()
                    .extend(depth),
                color: glyph.color[1],
                texture_coord: Point2D::new(0.0, 1.0),
            },
            GlyphVertex {
                position: ctx
                    .screen_matrix
                    .transform_point2d(coords[2])
                    .unwrap()
                    .extend(depth),
                color: glyph.color[2],
                texture_coord: Point2D::new(1.0, 1.0),
            },
            GlyphVertex {
                position: ctx
                    .screen_matrix
                    .transform_point2d(coords[3])
                    .unwrap()
                    .extend(depth),
                color: glyph.color[3],
                texture_coord: Point2D::new(1.0, 0.0),
            },
        ]));

        let instance_slice = ctx
            .vertex_stream
            .write_slice(bytemuck::bytes_of(&GlyphInstance {
                texture_rect: glyph.texture.view().texture_rect(),
            }));

        Self {
            texture: glyph.texture.clone(),
            vertices_slice,
            instance_slice,
        }
    }
}

impl Component for GlyphComponent {
    fn render_opaque<'rpass>(
        &'rpass self,
        _: &RenderContext<'rpass>,
        _: &mut dyn RenderEncoder<'rpass>,
    ) {
        unreachable!()
    }

    fn render_transparent<'rpass>(
        &'rpass self,
        ctx: &RenderContext<'rpass>,
        pass: &mut dyn RenderEncoder<'rpass>,
    ) {
        let text_resources = ctx.get::<TextResources>();

        pass.set_pipeline(&text_resources.pipeline);

        self.texture.bind(0, pass);

        pass.set_vertex_buffer(0, ctx.vertex_stream.slice(self.vertices_slice.clone()));
        pass.set_vertex_buffer(1, ctx.vertex_stream.slice(self.instance_slice.clone()));

        pass.set_index_buffer(
            ctx.resources
                .get::<QuadIndexBufferResources>(&ctx.backend)
                .quad_index_buffer
                .slice(..),
            QuadIndexBufferResources::FORMAT,
        );

        pass.draw_indexed(0..6, 0, 0..1);
    }
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct GlyphVertex {
    pub position: Point3D<f32, RenderUnit>,
    pub color: LinSrgba<f32>,
    pub texture_coord: Point2D<f32, TextureUnit>,
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct GlyphInstance {
    pub texture_rect: Rect<f32, TextureUnit>,
}

pub fn init_glyph_shader(device: &Device) -> ShaderModule {
    device.create_shader_module(&ShaderModuleDescriptor {
        label: Some("Glyph shader"),
        source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("text.wgsl"))),
    })
}

pub fn init_glyph_pipeline_layout(
    device: &Device,
    texture_bind_group_layout: &BindGroupLayout,
) -> PipelineLayout {
    device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Glyph shader pipeline layout"),
        bind_group_layouts: &[texture_bind_group_layout],
        push_constant_ranges: &[],
    })
}

pub fn init_glyph_pipeline(
    device: &Device,
    pipeline_layout: &PipelineLayout,
    shader: &ShaderModule,
    fragment_targets: &[ColorTargetState],
    depth_stencil: Option<DepthStencilState>,
) -> RenderPipeline {
    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Glyph pipeline"),
        layout: Some(pipeline_layout),
        vertex: VertexState {
            module: shader,
            entry_point: &"vs_main",
            buffers: &[
                VertexBufferLayout {
                    array_stride: std::mem::size_of::<GlyphVertex>() as u64,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &vertex_attr_array![0 => Float32x3, 1 => Float32x4, 2 => Float32x2],
                },
                VertexBufferLayout {
                    array_stride: std::mem::size_of::<GlyphInstance>() as u64,
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
