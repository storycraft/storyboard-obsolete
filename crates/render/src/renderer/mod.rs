pub mod context;
pub mod pass;
pub mod surface;

use std::{borrow::Cow, fmt::Debug};

use storyboard_core::{
    euclid::{Rect, Transform3D},
    unit::{LogicalPixelUnit, PhyiscalPixelUnit, RenderUnit},
};
use trait_stack::TraitStack;
use wgpu::{
    CompareFunction, DepthBiasState, DepthStencilState, Device, StencilFaceState, StencilState,
    TextureFormat, MultisampleState,
};

use self::{context::DrawContext, pass::StoryboardRenderPass};

use super::{
    buffer::stream::BufferStream,
    texture::{SizedTexture2D, SizedTextureView2D},
};

use crate::{
    component::{Component, Drawable},
    shared::{RenderPipelineData, RenderScope},
    wgpu::{
        BufferUsages, CommandEncoder, LoadOp, Operations, RenderPassColorAttachment,
        RenderPassDepthStencilAttachment, RenderPassDescriptor, TextureUsages,
    },
    ScreenRect,
};

#[derive(Debug)]
pub struct StoryboardRenderer {
    current_screen_rect: Rect<u32, PhyiscalPixelUnit>,
    screen_matrix: Transform3D<f32, LogicalPixelUnit, RenderUnit>,

    opaque_component: TraitStack<dyn Component>,
    transparent_component: TraitStack<dyn Component>,

    depth_texture: Option<SizedTextureView2D>,

    vertex_stream: BufferStream<'static>,
    index_stream: BufferStream<'static>,
}

impl StoryboardRenderer {
    pub const DEFAULT_DEPTH_TEXTURE_FORMAT: TextureFormat = TextureFormat::Depth32Float;

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
            current_screen_rect: Rect::zero(),
            screen_matrix: Transform3D::identity(),

            opaque_component: TraitStack::new(),
            transparent_component: TraitStack::new(),

            depth_texture: None,

            vertex_stream,
            index_stream,
        }
    }

    pub const fn create_renderer_pipeline_data(
        texture_format: TextureFormat,
        multi_sample: Option<MultisampleState>
    ) -> RenderPipelineData {
        RenderPipelineData {
            texture_format,
            depth_stencil: Some(DepthStencilState {
                format: Self::DEFAULT_DEPTH_TEXTURE_FORMAT,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: StencilState {
                    front: StencilFaceState::IGNORE,
                    back: StencilFaceState::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
                bias: DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
            }),
            multi_sample,
        }
    }

    fn update_screen_matrix(&mut self, screen: ScreenRect) {
        self.screen_matrix = screen.get_logical_ortho_matrix();
    }

    fn update_depth_stencil(&mut self, device: &Device, screen: ScreenRect) {
        self.depth_texture = Some(
            SizedTexture2D::init(
                device,
                Some("StoryboardRenderer depth texture"),
                screen.rect.size,
                Self::DEFAULT_DEPTH_TEXTURE_FORMAT,
                TextureUsages::RENDER_ATTACHMENT,
            )
            .create_view_default(None),
        );
    }

    pub fn render<'a>(
        &mut self,
        scope: RenderScope,
        screen: ScreenRect,
        drawables: impl ExactSizeIterator<Item = &'a dyn Drawable>,
        color_attachment: Option<RenderPassColorAttachment>,
        encoder: &mut CommandEncoder,
    ) {
        if drawables.len() == 0 || screen.rect.area() == 0 {
            return;
        }

        if self.current_screen_rect != screen.rect {
            self.update_screen_matrix(screen);

            if self.current_screen_rect.size != screen.rect.size {
                self.update_depth_stencil(scope.backend().device(), screen);
            }

            self.current_screen_rect = screen.rect;
        }

        let mut draw_context = DrawContext {
            scope,
            screen,
            screen_matrix: self.screen_matrix,
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
