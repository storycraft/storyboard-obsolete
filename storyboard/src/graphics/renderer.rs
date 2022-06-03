/*
 * Created on Sun May 08 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{borrow::Cow, fmt::Debug, ops::Deref, sync::Arc};

use dynstack::{dyn_push, DynStack};
use storyboard_core::{
    euclid::Transform3D,
    graphics::buffer::stream::BufferStream,
    store::Store,
    unit::{PixelUnit, RenderUnit},
    wgpu::{BufferUsages, CommandEncoder, RenderPassColorAttachment, RenderPassDescriptor},
};

use crate::graphics::{
    backend::StoryboardBackend, context::DrawContext, pass::StoryboardRenderPass,
};

use super::{
    component::{Component, Drawable},
    context::BackendContext,
    texture::TextureData,
};

pub struct StoryboardRenderer<'a> {
    drawables: DynStack<dyn Drawable>,
    components: DynStack<dyn Component>,

    resources: Arc<Store<BackendContext<'a>>>,

    vertex_stream: BufferStream<'static>,
    index_stream: BufferStream<'static>,
}

impl<'a> StoryboardRenderer<'a> {
    pub fn new(resources: Arc<Store<BackendContext<'a>>>) -> Self {
        let vertex_stream = BufferStream::new(
            Some(Cow::from("StoryboardRenderer vertex stream buffer")),
            BufferUsages::VERTEX,
        );
        let index_stream = BufferStream::new(
            Some(Cow::from("StoryboardRenderer index stream buffer")),
            BufferUsages::INDEX,
        );

        Self {
            drawables: DynStack::new(),
            components: DynStack::new(),

            resources,

            vertex_stream,
            index_stream,
        }
    }

    pub fn push(&mut self, drawable: impl Drawable + 'static) {
        dyn_push!(self.drawables, drawable);
    }

    pub fn render(
        &mut self,
        backend: &StoryboardBackend,
        textures: &TextureData,
        screen_matrix: &Transform3D<f32, PixelUnit, RenderUnit>,
        color_attachments: &[RenderPassColorAttachment],
        encoder: &mut CommandEncoder,
    ) {
        let backend_context = BackendContext {
            device: backend.device(),
            queue: backend.queue(),
            textures,

            // TODO:: Depth stencil
            depth_stencil: None,
        };

        let mut draw_context = DrawContext {
            backend: backend_context,
            screen_matrix,
            resources: self.resources.deref(),
            vertex_stream: &mut self.vertex_stream,
            index_stream: &mut self.index_stream,
        };

        {
            let mut components_queue = ComponentQueue(&mut self.components);

            let total = self.drawables.len() as f32;
            for (i, drawable) in self.drawables.iter().enumerate() {
                drawable.prepare(
                    &mut components_queue,
                    &mut draw_context,
                    1.0_f32 - ((1.0_f32 + i as f32) / total),
                );
            }

            // TODO:: Do clear instead of allocating clearing
            self.drawables = DynStack::new();
        }

        {
            let mut render_context = draw_context.into_render_context();

            let mut pass =
                StoryboardRenderPass::new(encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("StoryboardRenderer render pass"),
                    color_attachments,
                    depth_stencil_attachment: None,
                }));

            for component in &self.components {
                component.render(&mut render_context, &mut pass);
            }
        }
        // TODO:: Do clear instead of allocating
        self.components = DynStack::new();
    }
}

impl Debug for StoryboardRenderer<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoryboardRenderer")
            .field("resources", &self.resources)
            .field("vertex_stream", &self.vertex_stream)
            .field("index_stream", &self.index_stream)
            .finish()
    }
}

pub struct ComponentQueue<'a>(&'a mut DynStack<dyn Component>);

impl<'a> ComponentQueue<'a> {
    pub fn push(&mut self, component: impl Component + 'static) {
        dyn_push!(self.0, component);
    }
}
