/*
 * Created on Mon Nov 15 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use euclid::Size2D;
use wgpu::CommandEncoder;

use crate::{
    component::{color::ShapeColor, extent::ExtentUnit, layout::texture::TextureLayout, DrawBox},
    graphics::{
        buffer::{index::IndexBuffer, stream::StreamSlice},
        context::{DrawContext, RenderContext},
        pass::StoryboardRenderPass,
        renderer::{box2d::draw_box2d, DrawState, RenderState, RenderStateQueue},
    },
};

#[derive(Debug, Clone)]
pub struct BoxStyle {
    pub fill_color: ShapeColor<4>,
    pub border_color: ShapeColor<4>,

    pub border_thickness: f32,
    pub border_radius: ExtentUnit,

    // pub texture: Option<ComponentTexture>,
}

impl Default for BoxStyle {
    fn default() -> Self {
        Self {
            fill_color: ShapeColor::default(),
            border_color: ShapeColor::default(),

            border_thickness: 0.0,
            border_radius: ExtentUnit::default()
        }
    }
}

pub struct Box2DDrawState {
    pub style: BoxStyle,
    pub draw_box: DrawBox,
}

impl DrawState for Box2DDrawState {
    fn prepare(
        &mut self,
        ctx: &mut DrawContext,
        depth: f32,
        _: &mut CommandEncoder,
        state_queue: &mut RenderStateQueue,
    ) {
        let (quad, instance) = draw_box2d(
            &self.draw_box,
            depth,
            &self.style.fill_color,
            &self.style.border_color,
            self.style.border_radius.calc(
                self.draw_box
                    .rect
                    .size
                    .width
                    .min(self.draw_box.rect.size.height),
            ),
            self.style.border_thickness,
            &TextureLayout::Stretch,
            &Size2D::zero(),
        );

        let vertex_slice = {
            let mut vertex_entry = ctx.stream_allocator.start_entry();

            vertex_entry.write(bytemuck::cast_slice(&quad));

            vertex_entry.finish()
        };

        let instance_slice = {
            let mut instance_entry = ctx.stream_allocator.start_entry();

            instance_entry.write(bytemuck::bytes_of(&instance));

            instance_entry.finish()
        };

        state_queue.push(Box2DRenderState {
            vertex_slice,
            instance_slice,
        });
    }
}

pub struct Box2DRenderState {
    pub vertex_slice: StreamSlice,
    pub instance_slice: StreamSlice,
}

impl RenderState for Box2DRenderState {
    fn render<'r>(
        &'r mut self,
        context: &'r RenderContext<'r>,
        pass: &mut StoryboardRenderPass<'r>,
    ) {
        pass.set_pipeline(&context.render_data.box_pipeline);

        pass.set_vertex_buffer(0, context.stream_buffer.slice(&self.vertex_slice));
        pass.set_vertex_buffer(1, context.stream_buffer.slice(&self.instance_slice));
        pass.set_index_buffer(
            context.render_data.quad_index_buffer.slice(),
            IndexBuffer::FORMAT,
        );

        // TODO::
        pass.set_bind_group(0, &context.render_data.empty_texture_bind_group, &[]);

        pass.draw_indexed(0..6, 0, 0..1);
    }
}
