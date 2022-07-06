use std::{borrow::Cow, sync::Arc};

use bytemuck::{Pod, Zeroable};
use storyboard_core::{
    color::ShapeColor,
    euclid::{Point2D, Point3D, Rect, Transform3D},
    math::RectExt,
    palette::LinSrgba,
    store::{Store, StoreResources},
    unit::{LogicalPixelUnit, RenderUnit, TextureUnit},
};

use storyboard_render::{
    buffer::stream::StreamRange,
    component::{Component, Drawable},
    renderer::pass::StoryboardRenderPass,
    renderer::{
        context::{BackendContext, DrawContext, RenderContext},
        ComponentQueue,
    },
    wgpu::{
        util::{BufferInitDescriptor, DeviceExt},
        vertex_attr_array, BindGroupLayout, BlendState, Buffer, BufferUsages, ColorTargetState,
        ColorWrites, CommandEncoder, DepthStencilState, Device, FragmentState, IndexFormat,
        MultisampleState, PipelineLayout, PipelineLayoutDescriptor, PrimitiveState,
        PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor, ShaderModule,
        ShaderModuleDescriptor, ShaderSource, VertexBufferLayout, VertexState, VertexStepMode,
    },
};
use storyboard_texture::render::data::EmptyTextureResources;
use storyboard_texture::render::{data::TextureData, RenderTexture2D};

#[derive(Debug)]
pub struct PrimitiveResources {
    pub opaque_pipeline: RenderPipeline,
    pub transparent_pipeline: RenderPipeline,
    pub quad_index_buffer: Buffer,
}

impl StoreResources<BackendContext<'_>> for PrimitiveResources {
    fn initialize(store: &Store, ctx: &BackendContext) -> Self {
        let textures = store.get::<TextureData, _>(ctx);

        let shader = init_primitive_shader(ctx.device);
        let pipeline_layout =
            init_primitive_pipeline_layout(ctx.device, textures.bind_group_layout());

        let opaque_pipeline = init_primitive_pipeline(
            ctx.device,
            &pipeline_layout,
            &shader,
            &[Some(ColorTargetState {
                format: ctx.screen_format,
                blend: None,
                write_mask: ColorWrites::COLOR,
            })],
            Some(ctx.depth_stencil.clone()),
        );

        let transparent_pipeline = init_primitive_pipeline(
            ctx.device,
            &pipeline_layout,
            &shader,
            &[Some(ColorTargetState {
                format: ctx.screen_format,
                blend: Some(BlendState::ALPHA_BLENDING),
                write_mask: ColorWrites::ALL,
            })],
            Some(DepthStencilState {
                depth_write_enabled: false,
                ..ctx.depth_stencil.clone()
            }),
        );

        let quad_index_buffer = ctx.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Primitive quad index buffer"),
            contents: bytemuck::cast_slice(&[0_u16, 1, 2, 0, 2, 3]),
            usage: BufferUsages::INDEX,
        });

        Self {
            opaque_pipeline,
            transparent_pipeline,
            quad_index_buffer,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Triangle {
    pub bounds: Rect<f32, LogicalPixelUnit>,
    pub color: ShapeColor<3>,
    pub texture: Option<Arc<RenderTexture2D>>,
    pub transform: Transform3D<f32, LogicalPixelUnit, LogicalPixelUnit>,
}

impl Drawable for Triangle {
    fn prepare(
        &self,
        component_queue: &mut ComponentQueue,
        ctx: &mut DrawContext,
        _: &mut CommandEncoder,
        depth: f32,
    ) {
        if self.texture.is_none() && self.color.opaque() {
            component_queue.push_opaque(PrimitiveComponent::from_triangle(self, ctx, depth));
        } else {
            component_queue.push_transparent(PrimitiveComponent::from_triangle(self, ctx, depth));
        }
    }
}

#[derive(Debug, Clone)]
pub struct Rectangle {
    pub bounds: Rect<f32, LogicalPixelUnit>,
    pub color: ShapeColor<4>,
    pub texture: Option<Arc<RenderTexture2D>>,
    pub transform: Transform3D<f32, LogicalPixelUnit, LogicalPixelUnit>,
}

impl Drawable for Rectangle {
    fn prepare(
        &self,
        component_queue: &mut ComponentQueue,
        ctx: &mut DrawContext,
        _: &mut CommandEncoder,
        depth: f32,
    ) {
        if self.texture.is_none() && self.color.opaque() {
            component_queue.push_opaque(PrimitiveComponent::from_rectangle(self, ctx, depth));
        } else {
            component_queue.push_transparent(PrimitiveComponent::from_rectangle(self, ctx, depth));
        }
    }
}

#[derive(Debug)]
pub struct PrimitiveComponent {
    primitive_type: PrimitiveType,
    texture: Option<Arc<RenderTexture2D>>,
    vertices_slice: StreamRange,
}

#[derive(Debug)]
pub enum PrimitiveType {
    Triangle,
    Rectangle,
}

impl PrimitiveComponent {
    pub fn from_triangle(triangle: &Triangle, ctx: &mut DrawContext, depth: f32) -> Self {
        let coords = triangle
            .transform
            .outer_transformed_rect(&triangle.bounds)
            .unwrap_or(triangle.bounds)
            .into_coords();

        let vertices_slice = ctx.vertex_stream.write_slice(bytemuck::bytes_of(&[
            PrimitiveVertex {
                position: ctx
                    .screen_matrix
                    .transform_point2d((coords[0] + coords[3].to_vector()) / 2.0)
                    .unwrap()
                    .extend(depth),
                color: triangle.color[0],
                texture_coord: Point2D::new(0.5, 0.0),
            },
            PrimitiveVertex {
                position: ctx
                    .screen_matrix
                    .transform_point2d(coords[1])
                    .unwrap()
                    .extend(depth),
                color: triangle.color[1],
                texture_coord: Point2D::new(0.0, 1.0),
            },
            PrimitiveVertex {
                position: ctx
                    .screen_matrix
                    .transform_point2d(coords[2])
                    .unwrap()
                    .extend(depth),
                color: triangle.color[2],
                texture_coord: Point2D::new(1.0, 1.0),
            },
        ]));

        Self {
            primitive_type: PrimitiveType::Triangle,
            texture: triangle.texture.clone(),
            vertices_slice,
        }
    }

    pub fn from_rectangle(rect: &Rectangle, ctx: &mut DrawContext, depth: f32) -> Self {
        let coords = rect
            .transform
            .outer_transformed_rect(&rect.bounds)
            .unwrap_or(rect.bounds)
            .into_coords();

        let vertices_slice = ctx.vertex_stream.write_slice(bytemuck::bytes_of(&[
            PrimitiveVertex {
                position: ctx
                    .screen_matrix
                    .transform_point2d(coords[0])
                    .unwrap()
                    .extend(depth),
                color: rect.color[0],
                texture_coord: Point2D::new(0.0, 0.0),
            },
            PrimitiveVertex {
                position: ctx
                    .screen_matrix
                    .transform_point2d(coords[1])
                    .unwrap()
                    .extend(depth),
                color: rect.color[1],
                texture_coord: Point2D::new(0.0, 1.0),
            },
            PrimitiveVertex {
                position: ctx
                    .screen_matrix
                    .transform_point2d(coords[2])
                    .unwrap()
                    .extend(depth),
                color: rect.color[2],
                texture_coord: Point2D::new(1.0, 1.0),
            },
            PrimitiveVertex {
                position: ctx
                    .screen_matrix
                    .transform_point2d(coords[3])
                    .unwrap()
                    .extend(depth),
                color: rect.color[3],
                texture_coord: Point2D::new(1.0, 0.0),
            },
        ]));

        Self {
            primitive_type: PrimitiveType::Rectangle,
            texture: rect.texture.clone(),
            vertices_slice,
        }
    }
}

impl Component for PrimitiveComponent {
    fn render_opaque<'rpass>(
        &'rpass self,
        ctx: &RenderContext<'rpass>,
        pass: &mut StoryboardRenderPass<'rpass>,
    ) {
        let resources = ctx.get::<PrimitiveResources>();

        pass.set_pipeline(&resources.opaque_pipeline);

        pass.set_bind_group(
            0,
            self.texture
                .as_deref()
                .or_else(|| Some(&ctx.get::<EmptyTextureResources>().empty_texture))
                .unwrap()
                .bind_group(),
            &[],
        );

        pass.set_vertex_buffer(0, ctx.vertex_stream.slice(self.vertices_slice.clone()));

        match self.primitive_type {
            PrimitiveType::Triangle => {
                pass.draw(0..3, 0..1);
            }

            PrimitiveType::Rectangle => {
                pass.set_index_buffer(resources.quad_index_buffer.slice(..), IndexFormat::Uint16);

                pass.draw_indexed(0..6, 0, 0..1);
            }
        }
    }

    fn render_transparent<'rpass>(
        &'rpass self,
        ctx: &RenderContext<'rpass>,
        pass: &mut StoryboardRenderPass<'rpass>,
    ) {
        let resources = ctx.get::<PrimitiveResources>();

        pass.set_pipeline(&resources.transparent_pipeline);

        pass.set_bind_group(
            0,
            self.texture
                .as_deref()
                .or_else(|| Some(&ctx.get::<EmptyTextureResources>().empty_texture))
                .unwrap()
                .bind_group(),
            &[],
        );

        pass.set_vertex_buffer(0, ctx.vertex_stream.slice(self.vertices_slice.clone()));

        match self.primitive_type {
            PrimitiveType::Triangle => {
                pass.draw(0..3, 0..1);
            }

            PrimitiveType::Rectangle => {
                pass.set_index_buffer(resources.quad_index_buffer.slice(..), IndexFormat::Uint16);

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
    pub texture_coord: Point2D<f32, TextureUnit>,
}

pub fn init_primitive_shader(device: &Device) -> ShaderModule {
    device.create_shader_module(ShaderModuleDescriptor {
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
    fragment_targets: &[Option<ColorTargetState>],
    depth_stencil: Option<DepthStencilState>,
) -> RenderPipeline {
    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Primitive pipeline"),
        layout: Some(pipeline_layout),
        vertex: VertexState {
            module: shader,
            entry_point: "vs_main",
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
            entry_point: "fs_main",
            targets: fragment_targets,
        }),
        multiview: None,
    })
}
