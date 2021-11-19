/*
 * Created on Mon Oct 18 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::sync::Arc;

use storyboard_graphics::{
    buffer::{index::IndexBuffer, stream::StreamSlice},
    component::{DrawSpace, DrawState, Drawable, RenderState},
    context::{DrawContext, RenderContext},
    math::Point2D,
    pass::StoryboardRenderPass,
    pipeline::PipelineTargetDescriptor,
    renderer::RenderStateQueue,
    texture::Texture2D,
    unit::PixelUnit,
    wgpu::{BindGroupLayout, CommandEncoder, Device, Queue, RenderPipeline},
};

use crate::{
    store::GlyphStore, init_text_pipeline, init_text_shader, layout::PositionedGlyph, TextStyle,
    TextVertex,
};

pub struct GlyphCompositor {
    pipeline: RenderPipeline,
    quad_index_buffer: IndexBuffer,
}

impl GlyphCompositor {
    pub const fn new(pipeline: RenderPipeline, quad_index_buffer: IndexBuffer) -> Self {
        Self {
            pipeline,
            quad_index_buffer,
        }
    }

    pub fn init(
        device: &Device,
        texture_bind_group_layout: &BindGroupLayout,
        pipeline_desc: PipelineTargetDescriptor,
    ) -> Self {
        let shader = init_text_shader(device, texture_bind_group_layout);
        let pipeline = init_text_pipeline(device, &shader, pipeline_desc);

        let quad_index_buffer =
            IndexBuffer::init(device, Some("Text Quad index buffer"), &[0, 1, 2, 3, 0, 2]);

        Self {
            pipeline,
            quad_index_buffer,
        }
    }

    pub fn text(
        &self,
        queue: &Queue,
        brush: &mut GlyphStore,
        text: &Vec<PositionedGlyph>,
        style: &TextStyle,
        space: &DrawSpace,
        point: Point2D<f32, PixelUnit>,
    ) -> Drawable<TextDrawState> {
        let texture = brush.texture().clone();

        let mut quads = Vec::with_capacity(text.len());

        let size_scale = style.size / brush.size();

        let size_multiplier = brush.draw_font().size_multiplier(style.size);

        for glyph in text {
            if let Some(tex_info) = brush.get_glyph_tex_info(queue, glyph.glyph_id as u32) {
                let glyph_box = space.inner_box(
                    tex_info
                        .raster_rect
                        .cast()
                        .scale(size_scale, size_scale)
                        .translate(
                            glyph.position.to_vector().cast::<f32>().cast_unit() * size_multiplier
                                + point.to_vector(),
                        ),
                    None,
                );

                let glyph_quad = glyph_box.get_quad_2d(&glyph_box.rect);

                let quad = [
                    TextVertex {
                        position: glyph_quad[0].to_3d().to_array(),
                        color: style.color[0].into_encoding(),
                        texure_coord: [
                            tex_info.texture_rect.origin.x,
                            tex_info.texture_rect.origin.y,
                        ],
                    },
                    TextVertex {
                        position: glyph_quad[1].to_3d().to_array(),
                        color: style.color[1].into_encoding(),
                        texure_coord: [
                            tex_info.texture_rect.origin.x,
                            tex_info.texture_rect.origin.y + tex_info.texture_rect.size.height,
                        ],
                    },
                    TextVertex {
                        position: glyph_quad[2].to_3d().to_array(),
                        color: style.color[2].into_encoding(),
                        texure_coord: [
                            tex_info.texture_rect.origin.x + tex_info.texture_rect.size.width,
                            tex_info.texture_rect.origin.y + tex_info.texture_rect.size.height,
                        ],
                    },
                    TextVertex {
                        position: glyph_quad[3].to_3d().to_array(),
                        color: style.color[3].into_encoding(),
                        texure_coord: [
                            tex_info.texture_rect.origin.x + tex_info.texture_rect.size.width,
                            tex_info.texture_rect.origin.y,
                        ],
                    },
                ];

                quads.push(quad);
            }
        }

        Drawable {
            opaque: false,
            state: TextDrawState {
                pipeline: &self.pipeline,
                quad_index_buffer: &self.quad_index_buffer,
                quads,
                texture,
            },
        }
    }
}

#[derive(Debug)]
pub struct TextDrawState<'a> {
    pub pipeline: &'a RenderPipeline,
    pub quad_index_buffer: &'a IndexBuffer,

    pub quads: Vec<[TextVertex; 4]>,
    pub texture: Arc<Texture2D>,
}

impl<'a> DrawState<'a> for TextDrawState<'a> {
    fn prepare(
        &mut self,
        context: &mut DrawContext,
        depth: f32,
        _: &mut CommandEncoder,
        state_queue: &mut RenderStateQueue<'a>,
    ) {
        if self.quads.len() < 1 {
            return;
        }

        let instances = self.quads.len() as u32;

        let vertices = {
            let mut entry = context.stream_allocator.start_entry();

            for quad in &mut self.quads {
                quad[0].position[2] = depth;
                quad[1].position[2] = depth;
                quad[2].position[2] = depth;
                quad[3].position[2] = depth;
            }
            entry.write(bytemuck::cast_slice(&self.quads));

            entry.finish()
        };

        state_queue.push(TextRenderState {
            pipeline: &self.pipeline,
            quad_index_buffer: &self.quad_index_buffer,

            vertices,
            instances,

            texture: self.texture.clone(),
        });
    }
}

#[derive(Debug)]
pub struct TextRenderState<'a> {
    pub pipeline: &'a RenderPipeline,
    pub quad_index_buffer: &'a IndexBuffer,

    pub vertices: StreamSlice,
    pub instances: u32,

    pub texture: Arc<Texture2D>,
}

impl RenderState for TextRenderState<'_> {
    fn render<'r>(&'r mut self, context: &'r RenderContext, pass: &mut StoryboardRenderPass<'r>) {
        pass.set_pipeline(self.pipeline);
        pass.set_vertex_buffer(0, context.stream_buffer.slice(&self.vertices));
        pass.set_index_buffer(self.quad_index_buffer.slice(), IndexBuffer::FORMAT);
        pass.set_bind_group(0, self.texture.bind_group(), &[]);

        for i in 0..self.instances {
            pass.draw_indexed(0..6, i as i32 * 4, 0..1);
        }
    }
}
