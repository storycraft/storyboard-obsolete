/*
 * Created on Sun May 08 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{borrow::Cow, sync::Arc};

use storyboard_core::{
    graphics::buffer::stream::BufferStream,
    wgpu::{BufferUsages, CommandEncoder, RenderPassColorAttachment, RenderPassDescriptor},
};

use crate::graphics::{
    backend::StoryboardBackend, compositor::ComponentCompositor, context::DrawContext,
    pass::StoryboardRenderPass,
};

#[derive(Debug)]
pub struct StoryboardRenderer<C: ComponentCompositor> {
    compositor: Arc<C>,

    prepared_components: Vec<C::Prepared>,

    vertex_stream: BufferStream<'static>,
    index_stream: BufferStream<'static>,
}

impl<C: ComponentCompositor> StoryboardRenderer<C> {
    pub fn new(compositor: Arc<C>) -> Self {
        let vertex_stream = BufferStream::new(
            Some(Cow::from("StoryboardRenderer vertex stream buffer")),
            BufferUsages::VERTEX,
        );
        let index_stream = BufferStream::new(
            Some(Cow::from("StoryboardRenderer index stream buffer")),
            BufferUsages::INDEX,
        );

        Self {
            compositor,
            prepared_components: Vec::new(),

            vertex_stream,
            index_stream,
        }
    }

    pub fn render(
        &mut self,
        backend: &StoryboardBackend,
        components: &[C::Component],
        encoder: &mut CommandEncoder,
        color_attachments: &[RenderPassColorAttachment],
    ) {
        let mut draw_context = DrawContext {
            device: backend.device(),
            queue: backend.queue(),
            vertex_stream: &mut self.vertex_stream,
            index_stream: &mut self.index_stream,
        };

        let total = components.len() as f32;
        for (i, component) in components.iter().enumerate() {
            self.prepared_components.push(self.compositor.draw(
                &mut draw_context,
                component,
                1.0_f32 - ((1.0_f32 + i as f32) / total),
            ));
        }

        {
            let render_context = draw_context.into_render_context();

            let mut pass =
                StoryboardRenderPass::new(encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("RenderTask render pass"),
                    color_attachments,
                    depth_stencil_attachment: None,
                }));

            for prepared in &self.prepared_components {
                self.compositor
                    .render(&render_context, &mut pass, &prepared);
            }
        }

        self.prepared_components.clear();
    }
}
