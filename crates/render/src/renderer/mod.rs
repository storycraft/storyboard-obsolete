pub mod context;
pub mod pass;
pub mod surface;

use std::{borrow::Cow, fmt::Debug};

use storyboard_core::store::Store;
use trait_stack::TraitStack;
use wgpu::Device;

use self::{
    context::{BackendContext, DrawContext},
    pass::StoryboardRenderPass,
};

use super::{
    buffer::stream::BufferStream,
    texture::{SizedTexture2D, SizedTextureView2D},
};

use crate::{
    component::{self, Component, Drawable},
    wgpu::{
        BufferUsages, CommandEncoder, LoadOp, Operations, RenderPassColorAttachment,
        RenderPassDepthStencilAttachment, RenderPassDescriptor, TextureFormat, TextureUsages,
    },
    ScreenRect,
};

#[derive(Debug)]
pub struct StoryboardRenderer {
    opaque_component: TraitStack<dyn Component>,
    transparent_component: TraitStack<dyn Component>,

    depth_texture: Option<SizedTextureView2D>,

    vertex_stream: BufferStream<'static>,
    index_stream: BufferStream<'static>,
}

impl StoryboardRenderer {
    pub fn new() -> Self {
        let vertex_stream = BufferStream::new(
            Some(Cow::from("StoryboardRenderer vertex stream buffer")),
            BufferUsages::VERTEX,
        );
        let index_stream = BufferStream::new(
            Some(Cow::from("StoryboardRenderer index stream buffer")),
            BufferUsages::INDEX,
        );

        Self {
            opaque_component: TraitStack::new(),
            transparent_component: TraitStack::new(),

            depth_texture: None,

            vertex_stream,
            index_stream,
        }
    }

    fn update_depth_stencil(&mut self, device: &Device, screen: ScreenRect) {
        self.depth_texture = Some(
            SizedTexture2D::init(
                device,
                Some("StoryboardRenderer depth texture"),
                screen.rect.size,
                component::DEPTH_TEXTURE_FORMAT,
                TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
            )
            .create_view_default(None),
        );
    }

    pub fn render<'a>(
        &mut self,
        backend: BackendContext,
        screen: ScreenRect,
        drawables: impl ExactSizeIterator<Item = &'a dyn Drawable>,
        color_attachment: Option<RenderPassColorAttachment>,
        encoder: &mut CommandEncoder,
    ) {
        if drawables.len() == 0 || screen.rect.area() == 0 {
            return;
        }

        match &self.depth_texture {
            Some(depth_texture) if depth_texture.size() == screen.rect.size => {}

            _ => self.update_depth_stencil(backend.device, screen),
        };

        let mut draw_context = DrawContext {
            backend,
            screen,
            screen_matrix: screen.get_logical_ortho_matrix(),
            vertex_stream: &mut self.vertex_stream,
            index_stream: &mut self.index_stream,
        };

        {
            let mut components_queue = ComponentQueue {
                opaque: &mut self.opaque_component,
                transparent: &mut self.transparent_component,
            };

            let total = drawables.len() as f32;
            for (i, drawable) in drawables.enumerate() {
                drawable.prepare(
                    &mut components_queue,
                    &mut draw_context,
                    encoder,
                    1.0_f32 - ((1.0_f32 + i as f32) / total),
                );
            }
        }

        let render_opaque = !self.opaque_component.is_empty();
        let render_transparent = !self.transparent_component.is_empty();

        let render_context = draw_context.into_render_context();

        let depth_attachment = RenderPassDepthStencilAttachment {
            view: self.depth_texture.as_ref().unwrap().inner(),
            depth_ops: Some(Operations {
                load: LoadOp::Clear(1.0),
                store: true,
            }),
            stencil_ops: None,
        };

        if render_opaque || render_transparent {
            let mut pass =
                StoryboardRenderPass::new(encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("StoryboardRenderer render pass"),
                    color_attachments: &[color_attachment],
                    depth_stencil_attachment: Some(depth_attachment),
                }));

            if render_opaque {
                for component in self.opaque_component.iter().rev() {
                    component.render_opaque(&render_context, &mut pass);
                }
            }

            if render_transparent {
                for component in self.transparent_component.iter() {
                    component.render_transparent(&render_context, &mut pass);
                }
            }
        }

        if render_opaque {
            self.opaque_component.clear();
        }

        if render_transparent {
            self.transparent_component.clear();
        }
    }
}

impl Default for StoryboardRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct ComponentQueue<'a> {
    opaque: &'a mut TraitStack<dyn Component>,
    transparent: &'a mut TraitStack<dyn Component>,
}

impl<'a> ComponentQueue<'a> {
    pub fn new(
        opaque: &'a mut TraitStack<dyn Component>,
        transparent: &'a mut TraitStack<dyn Component>,
    ) -> Self {
        Self {
            opaque,
            transparent,
        }
    }

    pub fn push_opaque(&mut self, component: impl Component + 'static) {
        self.opaque.push(component);
    }

    pub fn push_transparent(&mut self, component: impl Component + 'static) {
        self.transparent.push(component);
    }
}

#[derive(Debug)]
/// Shared renderer data container
pub struct RendererData {
    screen_format: TextureFormat,
    store: Store,
}

impl RendererData {
    pub fn new(screen_format: TextureFormat) -> Self {
        Self {
            screen_format,
            store: Store::new(),
        }
    }

    pub fn is_valid(&self, format: TextureFormat) -> bool {
        self.screen_format == format
    }

    pub const fn screen_format(&self) -> TextureFormat {
        self.screen_format
    }

    pub const fn store(&self) -> &Store {
        &self.store
    }
}
