use std::{borrow::Cow, sync::Arc};

use bytemuck::{Pod, Zeroable};
use storyboard_core::{
    color::ShapeColor,
    euclid::{Point2D, Point3D, Rect, Transform3D, Vector2D},
    math::RectExt,
    palette::LinSrgba,
    store::{Store, StoreResources},
    unit::{LogicalPixelUnit, RenderUnit, TextureUnit},
};

use storyboard_render::{
    buffer::stream::StreamRange,
    cache::shader::ShaderCache,
    component::{Component, Drawable, self},
    renderer::pass::StoryboardRenderPass,
    renderer::{
        context::{DrawContext, RenderContext},
        ComponentQueue,
    },
    wgpu::{
        util::{BufferInitDescriptor, DeviceExt},
        vertex_attr_array, BindGroupLayout, BlendState, Buffer, BufferAddress, BufferUsages,
        ColorTargetState, ColorWrites, CommandEncoder, DepthStencilState, Device, FragmentState,
        IndexFormat, MultisampleState, PipelineLayout, PipelineLayoutDescriptor, PrimitiveState,
        PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor, ShaderModule,
        ShaderModuleDescriptor, ShaderSource, VertexBufferLayout, VertexState, VertexStepMode,
    }, shared::RenderScopeContext,
};
use storyboard_texture::{
    render::{
        data::{EmptyTextureResources, TextureData},
        RenderTexture2D,
    },
    ComponentTexture,
};

#[derive(Debug)]
pub struct Box2DResources {
    pub pipeline: RenderPipeline,
    pub box_index_buffer: Buffer,
}

impl StoreResources<RenderScopeContext<'_>> for Box2DResources {
    fn initialize(_: &Store, ctx: &RenderScopeContext) -> Self {
        let textures = ctx.backend.get::<TextureData>();

        let shader = ctx
        .backend
            .get::<ShaderCache>()
            .get_or_create("box_2d_shader", || init_box_shader(ctx.backend.device()));
        let pipeline_layout = init_box_pipeline_layout(ctx.backend.device(), textures.bind_group_layout());
        let pipeline = init_box_pipeline(
            ctx.backend.device(),
            &pipeline_layout,
            &shader,
            &[Some(ColorTargetState {
                format: ctx.texture_format,
                blend: Some(BlendState::ALPHA_BLENDING),
                write_mask: ColorWrites::ALL,
            })],
            Some(component::TRANSPARENT_DEPTH_STENCIL),
        );

        let box_index_buffer = ctx.backend.device().create_buffer_init(&BufferInitDescriptor {
            label: Some("Box2DResources quad index buffer"),
            contents: bytemuck::cast_slice(&[0_u16, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7]),
            usage: BufferUsages::INDEX,
        });

        Self {
            pipeline,
            box_index_buffer,
        }
    }
}

#[derive(Debug)]
pub struct Box2D {
    pub bounds: Rect<f32, LogicalPixelUnit>,

    pub texture: Option<ComponentTexture>,

    pub fill_color: ShapeColor<4>,
    pub border_color: ShapeColor<4>,

    pub style: Box2DStyle,

    pub transform: Transform3D<f32, LogicalPixelUnit, LogicalPixelUnit>,
}

impl Drawable for Box2D {
    fn prepare(
        &self,
        component_queue: &mut ComponentQueue,
        ctx: &mut DrawContext,
        _: &mut CommandEncoder,
        depth: f32,
    ) {
        if let Some(component) = Box2DComponent::from_box2d(self, ctx, depth) {
            component_queue.push_transparent(component);
        }
    }
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Box2DStyle {
    pub border_radius: [f32; 4],
    pub border_thickness: f32,

    pub glow_radius: f32,
    pub glow_color: LinSrgba,

    pub shadow_offset: Vector2D<f32, LogicalPixelUnit>,
    pub shadow_radius: f32,
    pub shadow_color: LinSrgba,
}

impl Default for Box2DStyle {
    fn default() -> Self {
        Self {
            border_radius: Default::default(),
            border_thickness: Default::default(),
            glow_radius: Default::default(),
            glow_color: LinSrgba::new(0.0, 0.0, 0.0, 0.0),
            shadow_offset: Default::default(),
            shadow_radius: Default::default(),
            shadow_color: LinSrgba::new(0.0, 0.0, 0.0, 0.0),
        }
    }
}

#[derive(Debug)]
pub struct Box2DComponent {
    texture: Option<Arc<RenderTexture2D>>,

    indices: u32,

    vertices_slice: StreamRange,
    instance_slice: StreamRange,
}

impl Box2DComponent {
    pub fn from_box2d(box2d: &Box2D, ctx: &mut DrawContext, depth: f32) -> Option<Self> {
        let border_bounds_inflation = box2d.style.border_thickness + 1.0;
        let bounds_inflation = border_bounds_inflation + box2d.style.glow_radius;
        let mut inflated_bounds = box2d.bounds.inflate(bounds_inflation, bounds_inflation);

        let shadow_bounds = box2d
            .bounds
            .inflate(
                border_bounds_inflation + box2d.style.shadow_radius,
                border_bounds_inflation + box2d.style.shadow_radius,
            )
            .translate(box2d.style.shadow_offset);

        let draw_shadow_box = !inflated_bounds.intersects(&shadow_bounds);

        let indices = if draw_shadow_box { 12 } else { 6 };

        if !draw_shadow_box {
            inflated_bounds = inflated_bounds.union(&shadow_bounds);
        }

        let texture_bounds = ComponentTexture::option_get_texture_bounds(
            box2d.texture.as_ref(),
            box2d.bounds,
            ctx.screen.get_logical_size(),
        );

        let texture_coords = texture_bounds
            .relative_in(&inflated_bounds)
            .cast_unit()
            .into_coords();

        let vertices_slice = {
            let mut writer = ctx.vertex_stream.next_writer();

            let box_coords = box2d
                .transform
                .outer_transformed_rect(&inflated_bounds)?
                .into_coords();

            writer.write(bytemuck::bytes_of(&[
                BoxVertex {
                    position: ctx
                        .screen_matrix
                        .transform_point2d(box_coords[0])?
                        .extend(depth),
                    fill_color: box2d.fill_color[0],
                    border_color: box2d.border_color[0],
                    rect_coord: box_coords[0],
                    texture_coord: texture_coords[0],
                },
                BoxVertex {
                    position: ctx
                        .screen_matrix
                        .transform_point2d(box_coords[1])?
                        .extend(depth),
                    fill_color: box2d.fill_color[1],
                    border_color: box2d.border_color[1],
                    rect_coord: box_coords[1],
                    texture_coord: texture_coords[1],
                },
                BoxVertex {
                    position: ctx
                        .screen_matrix
                        .transform_point2d(box_coords[2])?
                        .extend(depth),
                    fill_color: box2d.fill_color[2],
                    border_color: box2d.border_color[2],
                    rect_coord: box_coords[2],
                    texture_coord: texture_coords[2],
                },
                BoxVertex {
                    position: ctx
                        .screen_matrix
                        .transform_point2d(box_coords[3])?
                        .extend(depth),
                    fill_color: box2d.fill_color[3],
                    border_color: box2d.border_color[3],
                    rect_coord: box_coords[3],
                    texture_coord: texture_coords[3],
                },
            ]));

            if draw_shadow_box {
                let shadow_coords = box2d
                    .transform
                    .outer_transformed_rect(&shadow_bounds)?
                    .into_coords();

                writer.write(bytemuck::bytes_of(&[
                    BoxVertex {
                        position: ctx
                            .screen_matrix
                            .transform_point2d(shadow_coords[0])?
                            .extend(depth),
                        rect_coord: shadow_coords[0],
                        ..Default::default()
                    },
                    BoxVertex {
                        position: ctx
                            .screen_matrix
                            .transform_point2d(shadow_coords[1])?
                            .extend(depth),
                        rect_coord: shadow_coords[1],
                        ..Default::default()
                    },
                    BoxVertex {
                        position: ctx
                            .screen_matrix
                            .transform_point2d(shadow_coords[2])?
                            .extend(depth),
                        rect_coord: shadow_coords[2],
                        ..Default::default()
                    },
                    BoxVertex {
                        position: ctx
                            .screen_matrix
                            .transform_point2d(shadow_coords[3])?
                            .extend(depth),
                        rect_coord: shadow_coords[3],
                        ..Default::default()
                    },
                ]))
            }

            writer.finish()
        };

        let texture_rect = ComponentTexture::option_view_texture_rect(box2d.texture.as_ref());
        let texture_wrap = ComponentTexture::option_wrapping_mode(box2d.texture.as_ref());

        let instance_slice = ctx
            .vertex_stream
            .write_slice(bytemuck::bytes_of(&BoxInstance {
                rect: box2d.bounds,

                texture_rect,
                texture_wrap_mode_u: texture_wrap.0 as _,
                texture_wrap_mode_v: texture_wrap.1 as _,

                style: box2d.style,
            }));

        Some(Self {
            texture: box2d.texture.as_ref().map(|texture| texture.inner.clone()),
            indices,
            vertices_slice,
            instance_slice,
        })
    }
}

impl Component for Box2DComponent {
    fn render_opaque<'rpass>(
        &'rpass self,
        _: &RenderContext<'rpass>,
        _: &mut StoryboardRenderPass<'rpass>,
    ) {
        unreachable!()
    }

    fn render_transparent<'rpass>(
        &'rpass self,
        ctx: &RenderContext<'rpass>,
        pass: &mut StoryboardRenderPass<'rpass>,
    ) {
        let box_resources = ctx.scope.get::<Box2DResources>();

        pass.set_pipeline(&box_resources.pipeline);

        pass.set_vertex_buffer(0, ctx.vertex_stream.slice(self.vertices_slice.clone()));
        pass.set_vertex_buffer(1, ctx.vertex_stream.slice(self.instance_slice.clone()));

        pass.set_index_buffer(
            box_resources.box_index_buffer.slice(..),
            IndexFormat::Uint16,
        );

        pass.set_bind_group(
            0,
            self.texture
                .as_deref()
                .or_else(|| Some(&ctx.scope.backend().get::<EmptyTextureResources>().empty_texture))
                .unwrap()
                .bind_group(),
            &[],
        );

        pass.draw_indexed(0..self.indices, 0, 0..1);
    }
}

#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct BoxVertex {
    pub position: Point3D<f32, RenderUnit>,

    pub fill_color: LinSrgba<f32>,
    pub border_color: LinSrgba<f32>,

    pub rect_coord: Point2D<f32, LogicalPixelUnit>,
    pub texture_coord: Point2D<f32, TextureUnit>,
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct BoxInstance {
    pub rect: Rect<f32, LogicalPixelUnit>,
    pub texture_rect: Rect<f32, TextureUnit>,
    pub texture_wrap_mode_u: u32,
    pub texture_wrap_mode_v: u32,

    pub style: Box2DStyle,
}

pub fn init_box_shader(device: &Device) -> ShaderModule {
    device.create_shader_module(ShaderModuleDescriptor {
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
    fragment_targets: &[Option<ColorTargetState>],
    depth_stencil: Option<DepthStencilState>,
) -> RenderPipeline {
    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Box2D pipeline"),
        layout: Some(pipeline_layout),
        vertex: VertexState {
            module: shader,
            entry_point: "vs_main",
            buffers: &[
                VertexBufferLayout {
                    array_stride: std::mem::size_of::<BoxVertex>() as BufferAddress,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &vertex_attr_array![
                        0 => Float32x3,
                        1 => Float32x4,
                        2 => Float32x4,
                        3 => Float32x2,
                        4 => Float32x2
                    ],
                },
                VertexBufferLayout {
                    array_stride: std::mem::size_of::<BoxInstance>() as BufferAddress,
                    step_mode: VertexStepMode::Instance,
                    attributes: &vertex_attr_array![
                        5 => Float32x4,
                        6 => Float32x4,
                        7 => Uint32x2,
                        8 => Float32x4,
                        9 => Float32,
                        10 => Float32,
                        11 => Float32x4,
                        12 => Float32x2,
                        13 => Float32,
                        14 => Float32x4
                    ],
                },
            ],
        },
        fragment: Some(FragmentState {
            module: shader,
            entry_point: "fs_main",
            targets: fragment_targets,
        }),
        depth_stencil,
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            ..PrimitiveState::default()
        },
        multisample: MultisampleState::default(),
        multiview: None,
    })
}
