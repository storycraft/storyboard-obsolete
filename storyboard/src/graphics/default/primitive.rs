/*
 * Created on Mon Nov 15 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::sync::Arc;

use wgpu::CommandEncoder;

use crate::{
    component::{
        color::ShapeColor,
        layout::texture::{ComponentTexture, TextureLayout},
        DrawBox,
    },
    graphics::{
        buffer::{index::IndexBuffer, stream::StreamSlice},
        context::{DrawContext, RenderContext},
        pass::StoryboardRenderPass,
        renderer::{
            primitive::{draw_rect, draw_triangle},
            DrawState, RenderState, RenderStateQueue,
        },
        texture::Texture2D,
    },
};

#[derive(Debug, Clone)]
pub struct PrimitiveStyle {
    pub fill_color: ShapeColor<4>,
    pub opacity: f32,
    pub texture: Option<ComponentTexture>,
}

impl Default for PrimitiveStyle {
    fn default() -> Self {
        Self {
            fill_color: ShapeColor::white(),
            opacity: 1.0,
            texture: None,
        }
    }
}

pub struct RectDrawState {
    pub style: PrimitiveStyle,
    pub draw_box: DrawBox,
}

impl DrawState for RectDrawState {
    fn prepare(
        &mut self,
        ctx: &mut DrawContext,
        depth: f32,
        _: &mut CommandEncoder,
        state_queue: &mut RenderStateQueue,
    ) {
        let texture_coords = match &self.style.texture {
            Some(component_texture) => {
                component_texture.layout.texture_coord_quad(
                    &self.draw_box.into_space(),
                    component_texture.texture.size(),
                )
            }
            None => TextureLayout::STRETCHED,
        };

        let quad = draw_rect(
            &self.draw_box,
            depth,
            &self.style.fill_color,
            &texture_coords,
        );

        let vertex_slice = {
            let mut vertex_entry = ctx.stream_allocator.start_entry();

            vertex_entry.write(bytemuck::cast_slice(&quad));

            vertex_entry.finish()
        };

        let texture = self
            .style
            .texture
            .take()
            .map(|component_texture| component_texture.texture);

        state_queue.push(RectRenderState {
            vertex_slice,
            texture,
        });
    }
}

pub struct RectRenderState {
    pub vertex_slice: StreamSlice,
    pub texture: Option<Arc<Texture2D>>,
}

impl RenderState for RectRenderState {
    fn render<'r>(&'r self, context: &RenderContext<'r>, pass: &mut StoryboardRenderPass<'r>) {
        pass.set_pipeline(&context.render_data.primitive_pipeline);

        pass.set_vertex_buffer(0, context.stream_buffer.slice(&self.vertex_slice));
        pass.set_index_buffer(
            context.render_data.quad_index_buffer.slice(),
            IndexBuffer::FORMAT,
        );

        pass.set_bind_group(
            0,
            self.texture
                .as_ref()
                .map(|texture| texture.bind_group())
                .unwrap_or(&context.render_data.empty_texture_bind_group),
            &[],
        );

        pass.draw_indexed(0..6, 0, 0..1);
    }
}

pub struct TriangleDrawState {
    pub style: PrimitiveStyle,
    pub draw_box: DrawBox,
}

impl DrawState for TriangleDrawState {
    fn prepare(
        &mut self,
        ctx: &mut DrawContext,
        depth: f32,
        _: &mut CommandEncoder,
        state_queue: &mut RenderStateQueue,
    ) {
        let texture_coords = match &self.style.texture {
            Some(component_texture) => {
                component_texture.layout.texture_coord_quad(
                    &self.draw_box.into_space(),
                    component_texture.texture.size(),
                )
            }
            None => TextureLayout::STRETCHED,
        };

        let triangle = draw_triangle(
            &self.draw_box,
            depth,
            &self.style.fill_color,
            &texture_coords,
        );

        let vertex_slice = {
            let mut vertex_entry = ctx.stream_allocator.start_entry();

            vertex_entry.write(bytemuck::cast_slice(&triangle));

            vertex_entry.finish()
        };

        let texture = self
            .style
            .texture
            .take()
            .map(|component_texture| component_texture.texture);

        state_queue.push(TriangleRenderState {
            vertex_slice,
            texture,
        });
    }
}

pub struct TriangleRenderState {
    pub vertex_slice: StreamSlice,
    pub texture: Option<Arc<Texture2D>>,
}

impl RenderState for TriangleRenderState {
    fn render<'r>(&'r self, context: &RenderContext<'r>, pass: &mut StoryboardRenderPass<'r>) {
        pass.set_pipeline(&context.render_data.primitive_pipeline);

        pass.set_vertex_buffer(0, context.stream_buffer.slice(&self.vertex_slice));

        pass.set_bind_group(
            0,
            self.texture
                .as_ref()
                .map(|texture| texture.bind_group())
                .unwrap_or(&context.render_data.empty_texture_bind_group),
            &[],
        );

        pass.draw(0..3, 0..1);
    }
}
