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
    },
};

pub struct TextDrawState {
    pub color: ShapeColor<4>,
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
        let mut map: FxHashMap<usize, Vec<StreamSlice>> = FxHashMap::default();

        for (draw_box, tex_info) in &self.glyphs {
            if let Some(texture) = self.textures.get(tex_info.index) {
                let mut entry = ctx.stream_allocator.start_entry();

                entry.write(&storyboard::bytemuck::cast_slice(
                    &draw_masked_rect(
                        draw_box,
                        depth,
                        &self.color,
                        &TextureLayout::STRETCHED,
                        &texture.to_tex_coords(tex_info.rect),
                    )));
                
                map.entry(tex_info.index).or_default().push(entry.finish());
            }
        }

        state_queue.push(TextRenderState {
            glyphs: map,
            textures: self.textures.clone(),
        })
    }
}

pub struct TextRenderState {
    pub glyphs: FxHashMap<usize, Vec<StreamSlice>>,
    pub textures: Vec<Arc<Texture2D>>,
}

impl RenderState for TextRenderState {
    fn render<'r>(&'r self, context: &RenderContext<'r>, pass: &mut StoryboardRenderPass<'r>) {
        pass.set_pipeline(&context.render_data.mask_pipeline);

        // TODO
        pass.set_bind_group(0, &context.render_data.empty_texture_bind_group, &[]);
        pass.set_index_buffer(
            context.render_data.quad_index_buffer.slice(),
            IndexBuffer::FORMAT,
        );

        for (i, slices) in self.glyphs.iter() {
            if let Some(texture) = self.textures.get(*i) {
                pass.set_bind_group(1, texture.bind_group(), &[]);

                for slice in slices {
                    pass.set_vertex_buffer(0, context.stream_buffer.slice(slice));
                    pass.draw_indexed(0..6, 0, 0..1);
                }
            }
        }
    }
}
