/*
 * Created on Fri Nov 12 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

pub mod backend;
pub mod buffer;
pub mod context;
pub mod default;
pub mod pass;
pub mod renderer;
pub mod surface;
pub mod texture;

pub use wgpu;
pub use lyon;

use std::sync::Arc;

use wgpu::{
    BufferUsages, Color, CommandEncoder, Device, LoadOp, Operations, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, TextureView,
};

use self::{
    buffer::stream::StreamBufferAllocator,
    context::DrawContext,
    pass::StoryboardRenderPass,
    renderer::{RenderData, StoryboardRenderer},
    texture::TextureData,
};

#[derive(Debug, Clone, Copy)]
pub struct PixelUnit;

#[derive(Debug, Clone, Copy)]
pub struct WgpuUnit;

pub struct StoryboardProjector {
    render_data: Arc<RenderData>,
    texture_data: Arc<TextureData>,
    stream_allocator: StreamBufferAllocator,

    clear_color: Color
}

impl StoryboardProjector {
    pub fn new(render_data: Arc<RenderData>, texture_data: Arc<TextureData>) -> Self {
        Self {
            render_data,
            texture_data,
            stream_allocator: StreamBufferAllocator::new(
                BufferUsages::INDEX | BufferUsages::VERTEX,
            ),
            clear_color: Color::BLACK
        }
    }

    pub const fn clear_color(&self) -> &Color {
        &self.clear_color
    }

    pub fn set_clear_color(&mut self, clear_color: Color) {
        self.clear_color = clear_color;
    }

    pub fn project(
        &mut self,
        mut renderer: StoryboardRenderer,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        surface: &TextureView,
    ) {
        let mut draw_ctx = DrawContext {
            device,
            queue,
            textures: &self.texture_data,
            stream_allocator: &mut self.stream_allocator,
        };

        renderer.prepare(&mut draw_ctx, encoder);

        let pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Storyboard projector render pass"),
            color_attachments: &[RenderPassColorAttachment {
                view: &surface,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(self.clear_color),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        renderer.render(
            &draw_ctx.into_render_context(&self.render_data),
            &mut StoryboardRenderPass::new(pass),
        );
    }
}
