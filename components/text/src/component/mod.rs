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
    cache::shader::ShaderCache,
    component::{Component, Drawable, self},
    renderer::{
        context::{BackendContext, DrawContext, RenderContext},
        pass::StoryboardRenderPass,
        ComponentQueue,
    },
    wgpu::{
        vertex_attr_array, BindGroupLayout, BlendState, ColorTargetState, ColorWrites,
        CommandEncoder, DepthStencilState, Device, FragmentState, MultisampleState, PipelineLayout,
        PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology, RenderPipeline,
        RenderPipelineDescriptor, ShaderModule, ShaderModuleDescriptor, ShaderSource,
        VertexBufferLayout, VertexState, VertexStepMode,
    },
};
use storyboard_texture::render::{data::TextureData, RenderTexture2D};

#[derive(Debug)]
pub struct TextResources {
    pub pipeline: RenderPipeline,
}

impl StoreResources<BackendContext<'_>> for TextResources {
    fn initialize(_: &Store, ctx: &BackendContext) -> Self {
        let textures = ctx.get::<TextureData>();

        let shader = ctx
            .get::<ShaderCache>()
            .get_or_create("glyph_shader", || init_glyph_shader(ctx.device));
        let pipeline_layout = init_glyph_pipeline_layout(ctx.device, textures.bind_group_layout());

        let pipeline = init_glyph_pipeline(
            ctx.device,
            &pipeline_layout,
            &shader,
            &[Some(ColorTargetState {
                format: ctx.screen_format(),
                blend: Some(BlendState::ALPHA_BLENDING),
                write_mask: ColorWrites::ALL,
            })],
            Some(component::TRANSPARENT_DEPTH_STENCIL),
        );

        Self { pipeline }
    }
}

#[derive(Debug, Clone)]
pub struct TextRenderBatch {
    pub texture: Arc<RenderTexture2D>,
    pub rects: Vec<GlyphRect>,
}

#[derive(Debug, Clone)]
pub struct GlyphRect {
    pub rect: Rect<f32, LogicalPixelUnit>,
    pub texture_rect: Rect<f32, TextureUnit>,
}

#[derive(Debug)]
pub struct TextDrawable {
    pub batches: Arc<Vec<TextRenderBatch>>,
    pub transform: Transform3D<f32, LogicalPixelUnit, LogicalPixelUnit>,
    pub color: ShapeColor<4>,
}

impl Drawable for TextDrawable {
    fn prepare(
        &self,
        component_queue: &mut ComponentQueue,
        ctx: &mut DrawContext,
        _: &mut CommandEncoder,
        depth: f32,
    ) {
        for batch in self.batches.iter() {
            if let Some(component) = GlyphComponent::from_batch(batch, &self.transform, ctx, depth)
            {
                component_queue.push_transparent(component);
            }
        }
    }
}

#[derive(Debug)]
pub struct GlyphComponent {
    texture: Arc<RenderTexture2D>,
    vertices: u32,
    vertices_slice: StreamRange,
}

impl GlyphComponent {
    pub fn from_batch(
        batch: &TextRenderBatch,
        transform: &Transform3D<f32, LogicalPixelUnit, LogicalPixelUnit>,
        ctx: &mut DrawContext,
        depth: f32,
    ) -> Option<Self> {
        let mut writer = ctx.vertex_stream.next_writer();

        let mut vertices = 0;
        for rect in &batch.rects {
            if rect.texture_rect.area() <= 0.0 {
                continue;
            }

            let coords = transform.outer_transformed_rect(&rect.rect)?.into_coords();
            let tex_coords = rect.texture_rect.into_coords();

            let left_top = GlyphVertex {
                position: ctx
                    .screen_matrix
                    .transform_point2d(coords[0])?
                    .extend(depth),
                color: ShapeColor::WHITE.into(),
                texture_coord: tex_coords[0],
            };

            let left_bottom = GlyphVertex {
                position: ctx
                    .screen_matrix
                    .transform_point2d(coords[1])?
                    .extend(depth),
                color: ShapeColor::WHITE.into(),
                texture_coord: tex_coords[1],
            };

            let right_bottom = GlyphVertex {
                position: ctx
                    .screen_matrix
                    .transform_point2d(coords[2])?
                    .extend(depth),
                color: ShapeColor::WHITE.into(),
                texture_coord: tex_coords[2],
            };

            let right_top = GlyphVertex {
                position: ctx
                    .screen_matrix
                    .transform_point2d(coords[3])?
                    .extend(depth),
                color: ShapeColor::WHITE.into(),
                texture_coord: tex_coords[3],
            };

            writer.write(bytemuck::bytes_of(&[
                left_top,
                left_bottom,
                right_top,
                right_top,
                left_bottom,
                right_bottom,
            ]));

            vertices += 6;
        }

        if vertices == 0 {
            return None;
        }

        let vertices_slice = writer.finish();

        Some(Self {
            texture: batch.texture.clone(),
            vertices,
            vertices_slice,
        })
    }
}

impl Component for GlyphComponent {
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
        let text_resources = ctx.get::<TextResources>();

        pass.set_pipeline(&text_resources.pipeline);
        pass.set_bind_group(0, self.texture.bind_group(), &[]);
        pass.set_vertex_buffer(0, ctx.vertex_stream.slice(self.vertices_slice.clone()));
        pass.draw(0..self.vertices, 0..1);
    }
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct GlyphVertex {
    pub position: Point3D<f32, RenderUnit>,
    pub color: LinSrgba<f32>,
    pub texture_coord: Point2D<f32, TextureUnit>,
}

pub fn init_glyph_shader(device: &Device) -> ShaderModule {
    device.create_shader_module(ShaderModuleDescriptor {
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
    fragment_targets: &[Option<ColorTargetState>],
    depth_stencil: Option<DepthStencilState>,
) -> RenderPipeline {
    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Glyph pipeline"),
        layout: Some(pipeline_layout),
        vertex: VertexState {
            module: shader,
            entry_point: "vs_main",
            buffers: &[VertexBufferLayout {
                array_stride: std::mem::size_of::<GlyphVertex>() as u64,
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
