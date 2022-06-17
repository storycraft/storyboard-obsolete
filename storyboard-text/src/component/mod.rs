pub mod text;

use std::{borrow::Cow, sync::Arc};

use bytemuck::{Pod, Zeroable};
use storyboard::core::{
    component::color::ShapeColor,
    euclid::{Point2D, Point3D, Rect},
    graphics::{
        buffer::stream::StreamRange,
        component::{Component, Drawable},
        renderer::{
            context::{BackendContext, DrawContext, RenderContext},
            pass::StoryboardRenderPass,
            ComponentQueue,
        },
    },
    palette::LinSrgba,
    store::{Store, StoreResources},
    unit::{LogicalPixelUnit, RenderUnit, TextureUnit},
    wgpu::{
        vertex_attr_array, BindGroupLayout, BlendState, ColorTargetState, ColorWrites,
        CommandEncoder, DepthStencilState, Device, FragmentState, MultisampleState, PipelineLayout,
        PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology, RenderPipeline,
        RenderPipelineDescriptor, ShaderModule, ShaderModuleDescriptor, ShaderSource,
        VertexBufferLayout, VertexState, VertexStepMode,
    },
};

use storyboard::{
    graphics::texture::{data::TextureData, RenderTexture2D},
    math::RectExt,
};

#[derive(Debug)]
pub struct TextResources {
    pub pipeline: RenderPipeline,
}

impl StoreResources<BackendContext<'_>> for TextResources {
    fn initialize(store: &Store, ctx: &BackendContext) -> Self {
        let textures = store.get::<TextureData, _>(ctx);

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
            component_queue.push_transparent(GlyphComponent::from_batch(batch, ctx, depth));
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
    pub fn from_batch(batch: &TextRenderBatch, ctx: &mut DrawContext, depth: f32) -> Self {
        let mut writer = ctx.vertex_stream.next_writer();
        for rect in &batch.rects {
            let coords = rect.rect.into_coords();
            let tex_coords = rect.texture_rect.into_coords();

            let left_top = GlyphVertex {
                position: ctx
                    .screen_matrix
                    .transform_point2d(coords[0])
                    .unwrap()
                    .extend(depth),
                color: ShapeColor::WHITE.into(),
                texture_coord: tex_coords[0],
            };

            let left_bottom = GlyphVertex {
                position: ctx
                    .screen_matrix
                    .transform_point2d(coords[1])
                    .unwrap()
                    .extend(depth),
                color: ShapeColor::WHITE.into(),
                texture_coord: tex_coords[1],
            };

            let right_bottom = GlyphVertex {
                position: ctx
                    .screen_matrix
                    .transform_point2d(coords[2])
                    .unwrap()
                    .extend(depth),
                color: ShapeColor::WHITE.into(),
                texture_coord: tex_coords[2],
            };

            let right_top = GlyphVertex {
                position: ctx
                    .screen_matrix
                    .transform_point2d(coords[3])
                    .unwrap()
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
        }

        let vertices_slice = writer.finish();
        let vertices = 6 * batch.rects.len() as u32;

        Self {
            texture: batch.texture.clone(),
            vertices,
            vertices_slice,
        }
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
            entry_point: &"fs_main",
            targets: fragment_targets,
        }),
        multiview: None,
    })
}
