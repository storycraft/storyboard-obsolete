pub mod context;
pub mod pass;
pub mod surface;

use std::{borrow::Cow, fmt::Debug, sync::Arc};

use storyboard_core::{
    euclid::{Point2D, Rect, Size2D, Transform3D},
    observable::Observable,
    store::Store,
    unit::{LogicalPixelUnit, PhyiscalPixelUnit, RenderUnit},
};
use trait_stack::TraitStack;
use wgpu::{Device, Queue};

use self::{
    context::{BackendContext, DrawContext},
    pass::StoryboardRenderPass,
};

use super::{
    buffer::stream::BufferStream,
    texture::{SizedTexture2D, SizedTextureView2D},
};

use crate::{
    component::{Component, Drawable},
    wgpu::{
        BufferUsages, CommandEncoder, CommandEncoderDescriptor, CompareFunction, DepthBiasState,
        DepthStencilState, LoadOp, Operations, RenderPassColorAttachment,
        RenderPassDepthStencilAttachment, RenderPassDescriptor, StencilState, TextureFormat,
        TextureUsages,
    },
};

#[derive(Debug)]
pub struct StoryboardRenderer {
    screen: Observable<(Size2D<u32, PhyiscalPixelUnit>, f32)>,

    screen_format: TextureFormat,
    screen_matrix: Transform3D<f32, LogicalPixelUnit, RenderUnit>,

    opaque_component: TraitStack<dyn Component>,
    transparent_component: TraitStack<dyn Component>,

    depth_texture: Observable<Option<SizedTextureView2D>>,

    store: Arc<Store>,

    vertex_stream: BufferStream<'static>,
    index_stream: BufferStream<'static>,
}

impl StoryboardRenderer {
    pub const DEPTH_TEXTURE_FORMAT: TextureFormat = TextureFormat::Depth32Float;

    pub fn new(
        screen_size: Size2D<u32, PhyiscalPixelUnit>,
        screen_scale: f32,
        screen_format: TextureFormat,
    ) -> Self {
        let vertex_stream = BufferStream::new(
            Some(Cow::from("StoryboardRenderer vertex stream buffer")),
            BufferUsages::VERTEX,
        );
        let index_stream = BufferStream::new(
            Some(Cow::from("StoryboardRenderer index stream buffer")),
            BufferUsages::INDEX,
        );

        Self {
            screen: (screen_size, screen_scale).into(),

            screen_format,
            screen_matrix: Transform3D::identity(),

            opaque_component: TraitStack::new(),
            transparent_component: TraitStack::new(),

            depth_texture: None.into(),

            store: Arc::new(Store::new()),

            vertex_stream,
            index_stream,
        }
    }

    pub fn screen_size(&self) -> Size2D<u32, PhyiscalPixelUnit> {
        self.screen.0
    }

    pub fn screen_scale(&self) -> f32 {
        self.screen.1
    }

    pub fn set_screen_size(
        &mut self,
        screen_size: Size2D<u32, PhyiscalPixelUnit>,
        screen_scale: f32,
    ) {
        if self.screen.ne(&(screen_size, screen_scale)) {
            self.screen = (screen_size, screen_scale).into();
        }
    }

    pub const fn screen_format(&self) -> TextureFormat {
        self.screen_format
    }

    fn prepare_screen_matrix(&mut self) {
        if Observable::invalidate(&mut self.screen) {
            self.screen_matrix = Transform3D::ortho(
                0.0_f32,
                self.screen.0.width as f32 / self.screen.1,
                self.screen.0.height as f32 / self.screen.1,
                0.0,
                0.0,
                1.0,
            );

            Observable::mark(&mut self.depth_texture);
        }
    }

    fn prepare_depth_stencil(&mut self, device: &Device) {
        if Observable::invalidate(&mut self.depth_texture) || self.depth_texture.is_none() {
            self.depth_texture = Some(
                SizedTexture2D::init(
                    device,
                    Some("StoryboardRenderer depth texture"),
                    self.screen.0,
                    Self::DEPTH_TEXTURE_FORMAT,
                    TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
                )
                .create_view_default(None),
            )
            .into();
        }
    }

    pub fn render<'a>(
        &mut self,
        device: &Device,
        queue: &Queue,
        drawables: impl ExactSizeIterator<Item = &'a dyn Drawable>,
        color_attachment: RenderPassColorAttachment,
    ) -> Option<CommandEncoder> {
        if drawables.len() <= 0 || self.screen.0.area() <= 0 {
            return None;
        }

        self.prepare_screen_matrix();

        self.prepare_depth_stencil(device);

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("StoryboardRenderer command encoder"),
        });

        let backend_context = BackendContext {
            device,
            queue,

            screen_format: self.screen_format,

            depth_stencil: DepthStencilState {
                format: Self::DEPTH_TEXTURE_FORMAT,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            },
        };

        let mut draw_context = DrawContext {
            backend: backend_context,
            screen: Rect::new(
                Point2D::zero(),
                (self.screen.0.cast() / self.screen.1).cast_unit(),
            ),
            pixel_density: self.screen.1,
            screen_matrix: &self.screen_matrix,
            resources: &self.store,
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
                    &mut encoder,
                    1.0_f32 - ((1.0_f32 + i as f32) / total),
                );
            }
        }

        let render_opaque = self.opaque_component.len() > 0;
        let render_transparent = self.transparent_component.len() > 0;

        let mut render_context = draw_context.into_render_context();

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
                    component.render_opaque(&mut render_context, &mut pass);
                }
            }

            if render_transparent {
                for component in self.transparent_component.iter() {
                    component.render_transparent(&mut render_context, &mut pass);
                }
            }
        }

        if render_opaque {
            self.opaque_component.clear();
        }

        if render_transparent {
            self.transparent_component.clear();
        }

        Some(encoder)
    }

    pub fn clone_shared(
        &self,
        screen_size: Size2D<u32, PhyiscalPixelUnit>,
        screen_scale: f32,
    ) -> Self {
        Self {
            store: self.store.clone(),
            ..Self::new(screen_size, screen_scale, self.screen_format)
        }
    }
}

#[derive(Debug)]
pub struct ComponentQueue<'a> {
    opaque: &'a mut TraitStack<dyn Component>,
    transparent: &'a mut TraitStack<dyn Component>,
}

impl<'a> ComponentQueue<'a> {
    pub fn push_opaque(&mut self, component: impl Component + 'static) {
        self.opaque.push(component);
    }

    pub fn push_transparent(&mut self, component: impl Component + 'static) {
        self.transparent.push(component);
    }
}