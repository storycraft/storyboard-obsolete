/*
 * Created on Fri Nov 26 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

pub mod font;
pub mod mapping;
pub mod store;

use std::sync::Arc;

pub use font_kit;
use rustc_hash::FxHashMap;
use store::StoreGlyphTexInfo;
use storyboard::{
    component::{color::ShapeColor, layout::texture::TextureLayout, DrawBox},
    graphics::{
        buffer::{index::IndexBuffer, stream::StreamSlice},
        context::{DrawContext, RenderContext},
        pass::StoryboardRenderPass,
        renderer::{mask::draw_masked_rect, DrawState, RenderState, RenderStateQueue},
        texture::Texture2D,
        wgpu::CommandEncoder,
        PixelUnit,
    },
    math::Rect,
};

#[derive(Debug, Clone)]
pub struct TextStyle {
    pub color: ShapeColor<4>,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            color: ShapeColor::white(),
        }
    }
}

pub struct TextDrawState {
    pub style: TextStyle,
    pub glyphs: Vec<(DrawBox, StoreGlyphTexInfo)>,
    pub textures: Vec<Arc<Texture2D>>,
}

impl DrawState for TextDrawState {
    fn prepare(
        &mut self,
        ctx: &mut DrawContext,
        depth: f32,
        _: &mut CommandEncoder,
        state_queue: &mut RenderStateQueue,
    ) {
        let mut map: FxHashMap<usize, (StreamSlice, u32)> = FxHashMap::default();

        let mut group: FxHashMap<usize, Vec<(DrawBox, Rect<u32, PixelUnit>)>> =
            FxHashMap::default();
        for (draw_box, tex_info) in &self.glyphs {
            group
                .entry(tex_info.index)
                .or_default()
                .push((draw_box.clone(), tex_info.rect));
        }

        for (index, glyphs) in group.into_iter() {
            if let Some(texture) = self.textures.get(index) {
                let mut entry = ctx.stream_allocator.start_entry();

                let count = glyphs.len();

                for (draw_box, tex_rect) in glyphs {
                    entry.write(storyboard::bytemuck::cast_slice(&draw_masked_rect(
                        &draw_box,
                        depth,
                        &self.style.color,
                        &TextureLayout::STRETCHED,
                        &texture.to_tex_coords(tex_rect),
                    )));
                }

                map.insert(index, (entry.finish(), count as u32));
            }
        }

        state_queue.push(TextRenderState {
            glyphs: map,
            textures: self.textures.clone(),
        })
    }
}

pub struct TextRenderState {
    pub glyphs: FxHashMap<usize, (StreamSlice, u32)>,
    pub textures: Vec<Arc<Texture2D>>,
}

impl RenderState for TextRenderState {
    fn render<'r>(&'r self, context: &RenderContext<'r>, pass: &mut StoryboardRenderPass<'r>) {
        pass.set_pipeline(&context.render_data.mask_pipeline);

        pass.set_bind_group(0, &context.render_data.empty_texture_bind_group, &[]);
        pass.set_index_buffer(
            context.render_data.quad_index_buffer.slice(),
            IndexBuffer::FORMAT,
        );

        for (i, (slice, count)) in self.glyphs.iter() {
            if let Some(texture) = self.textures.get(*i) {
                pass.set_bind_group(1, texture.bind_group(), &[]);

                pass.set_vertex_buffer(0, context.stream_buffer.slice(slice));
                for c in 0..*count {
                    pass.draw_indexed(0..6, c as i32 * 4, 0..1);
                }
            }
        }
    }
}
