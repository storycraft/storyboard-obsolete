/*
 * Created on Mon Nov 15 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use euclid::Size2D;
use wgpu::CommandEncoder;

use crate::{component::{color::ShapeColor, layout::texture::TextureLayout, DrawBox}, graphics::{buffer::{index::IndexBuffer, stream::StreamSlice}, context::{DrawContext, RenderContext}, pass::StoryboardRenderPass, renderer::{DrawState, RenderState, RenderStateQueue, primitive::{draw_rect, draw_triangle}}}};

#[derive(Debug, Clone)]
pub struct PrimitiveStyle {
    pub fill_color: ShapeColor<4>,
    pub opacity: f32,
    // pub texture: Option<ComponentTexture>,
}

impl Default for PrimitiveStyle {
    fn default() -> Self {
        Self {
            fill_color: ShapeColor::default(),
            opacity: 1.0,
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
        let quad = draw_rect(
            &self.draw_box,
            depth,
            &self.style.fill_color,
            &TextureLayout::Stretch,
            &Size2D::zero(),
        );

        let vertex_slice = {
            let mut vertex_entry = ctx.stream_allocator.start_entry();

            vertex_entry.write(bytemuck::cast_slice(&quad));

            vertex_entry.finish()
        };

        state_queue.push(RectRenderState { vertex_slice });
    }
}

pub struct RectRenderState {
    pub vertex_slice: StreamSlice,
}

impl RenderState for RectRenderState {
    fn render<'r>(
        &'r mut self,
        context: &'r RenderContext<'r>,
        pass: &mut StoryboardRenderPass<'r>,
    ) {
        pass.set_pipeline(&context.render_data.primitive_pipeline);

        pass.set_vertex_buffer(0, context.stream_buffer.slice(&self.vertex_slice));
        pass.set_index_buffer(
            context.render_data.quad_index_buffer.slice(),
            IndexBuffer::FORMAT,
        );

        // TODO::
        pass.set_bind_group(0, &context.render_data.empty_texture_bind_group, &[]);

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
        let triangle = draw_triangle(
            &self.draw_box,
            depth,
            &self.style.fill_color,
            &TextureLayout::Stretch,
            &Size2D::zero(),
        );

        let vertex_slice = {
            let mut vertex_entry = ctx.stream_allocator.start_entry();

            vertex_entry.write(bytemuck::cast_slice(&triangle));

            vertex_entry.finish()
        };

        state_queue.push(RectRenderState { vertex_slice });
    }
}

pub struct TriangleRenderState {
    pub vertex_slice: StreamSlice,
}

impl RenderState for TriangleRenderState {
    fn render<'r>(
        &'r mut self,
        context: &'r RenderContext<'r>,
        pass: &mut StoryboardRenderPass<'r>,
    ) {
        pass.set_pipeline(&context.render_data.primitive_pipeline);

        pass.set_vertex_buffer(0, context.stream_buffer.slice(&self.vertex_slice));

        // TODO::
        pass.set_bind_group(0, &context.render_data.empty_texture_bind_group, &[]);

        pass.draw(0..3, 0..1);
    }
}
