/*
 * Created on Thu Nov 04 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

pub mod app;
pub mod compositor;
pub mod scene;
pub mod window;

pub use storyboard_box_2d as box_2d;
pub use storyboard_graphics as graphics;
pub use storyboard_path as path;
pub use storyboard_primitive as primitive;
pub use winit;

use std::{iter, sync::Arc};

use graphics::{
    backend::{BackendOptions, StoryboardBackend},
    buffer::stream::StreamBufferAllocator,
    component::DrawSpace,
    context::DrawContext,
    math::{Point2D, Rect},
    pipeline::PipelineTargetDescriptor,
    renderer::StoryboardRenderer,
    texture::{depth::DepthStencilTexture, resources::TextureResources},
    wgpu::{
        Backends, BlendState, BufferUsages, Color, ColorTargetState, ColorWrites,
        CommandEncoderDescriptor, CompareFunction, DepthBiasState, DepthStencilState, Instance,
        LoadOp, MultisampleState, Operations, PolygonMode, RenderPassColorAttachment,
        RenderPassDepthStencilAttachment, RenderPassDescriptor, StencilState, TextureFormat,
        TextureViewDescriptor,
    },
};

use scene::StoryboardScene;

use compositor::StoryboardCompositor;
use window::StoryboardWindow;
use winit::window::Window;

#[derive(Debug)]
pub struct Storyboard {
    backend: StoryboardBackend,

    window: StoryboardWindow,
    window_depth_stencil: DepthStencilTexture,

    stream_allocator: StreamBufferAllocator,

    textures: TextureResources,
    compositor: StoryboardCompositor,
}

impl Storyboard {
    pub async fn init(window: Window, options: BackendOptions) -> Self {
        let instance = Instance::new(Backends::all());

        // TODO:: remove unwrap
        let mut window = StoryboardWindow::init(&instance, window);

        // TODO:: remove unwrap
        let backend = window
            .surface()
            .create_backend(&instance, options)
            .await
            .unwrap();

        let surface_texture_format = window
            .surface_mut()
            .update_format_for(backend.adapter())
            .unwrap();

        let textures = TextureResources::init(
            Arc::clone(backend.device()),
            Arc::clone(backend.queue()),
            surface_texture_format,
        );

        let window_depth_stencil = DepthStencilTexture::init(backend.device(), window.inner_size());

        let compositor = StoryboardCompositor::init(
            backend.device(),
            textures.texture2d_bind_group_layout(),
            PipelineTargetDescriptor {
                fragments_targets: &[ColorTargetState {
                    format: surface_texture_format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::all(),
                }],
                topology: None,
                polygon_mode: PolygonMode::Fill,
                depth_stencil: Some(DepthStencilState {
                    format: TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: true,
                    depth_compare: CompareFunction::LessEqual,
                    stencil: StencilState {
                        read_mask: !0,
                        write_mask: !0,
                        ..StencilState::default()
                    },
                    bias: DepthBiasState::default(),
                }),
                multisample: MultisampleState::default(),
            },
        );

        let stream_allocator =
            StreamBufferAllocator::new(BufferUsages::INDEX | BufferUsages::VERTEX);

        Self {
            backend,

            window,
            window_depth_stencil,

            stream_allocator,

            textures,
            compositor,
        }
    }

    pub fn window(&self) -> &StoryboardWindow {
        &self.window
    }

    pub fn window_mut(&mut self) -> &mut StoryboardWindow {
        &mut self.window
    }

    pub fn textures(&self) -> &TextureResources {
        &self.textures
    }

    pub fn draw(&mut self, scene: &mut impl StoryboardScene) {
        let window_inner_size = self.window.inner_size();

        if window_inner_size.is_empty() {
            return;
        }

        if window_inner_size != *self.window_depth_stencil.size() {
            self.window.update_view(window_inner_size);
            self.window_depth_stencil =
                DepthStencilTexture::init(self.backend.device(), window_inner_size);
        }

        // TODO:: error checking
        if let Ok(current_texture) = self
            .window
            .surface_mut()
            .get_current_texture(self.backend.device())
        {
            let current_view = current_texture
                .texture
                .create_view(&TextureViewDescriptor::default());

            let mut renderer = StoryboardRenderer::with_capacity(256);

            scene.render(
                DrawSpace::new_screen(Rect {
                    origin: Point2D::zero(),
                    size: window_inner_size.cast(),
                }),
                &self.compositor,
                &mut renderer,
            );

            let mut encoder =
                self.backend
                    .device()
                    .create_command_encoder(&CommandEncoderDescriptor {
                        label: Some("Storyboard window command encoder"),
                    });

            let mut draw_ctx = DrawContext {
                device: self.backend.device(),
                queue: self.backend.queue(),
                textures: &self.textures,
                stream_allocator: &mut self.stream_allocator,
            };

            renderer.prepare(&mut draw_ctx, &mut encoder);

            let pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Storyboard window render pass"),
                color_attachments: &[RenderPassColorAttachment {
                    view: &current_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: self.window_depth_stencil.view(),
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: Some(Operations {
                        load: LoadOp::Clear(0),
                        store: true,
                    }),
                }),
            });

            renderer.render(&draw_ctx.into_render_context(), pass);

            self.backend.queue().submit(iter::once(encoder.finish()));
            current_texture.present();
        }
    }
}

#[derive(Debug)]
pub struct StoryboardTest2 {
    backend: StoryboardBackend,

    textures: TextureResources,
    compositor: StoryboardCompositor,
}

impl StoryboardTest2 {
    pub async fn init(window: Window, options: BackendOptions) -> (Self, StoryboardWindow) {
        let instance = Instance::new(Backends::all());

        // TODO:: remove unwrap
        let mut window = StoryboardWindow::init(&instance, window);

        // TODO:: remove unwrap
        let backend = window
            .surface()
            .create_backend(&instance, options)
            .await
            .unwrap();

        let surface_texture_format = window
            .surface_mut()
            .update_format_for(backend.adapter())
            .unwrap();

        let textures = TextureResources::init(
            Arc::clone(backend.device()),
            Arc::clone(backend.queue()),
            surface_texture_format,
        );

        let compositor = StoryboardCompositor::init(
            backend.device(),
            textures.texture2d_bind_group_layout(),
            PipelineTargetDescriptor {
                fragments_targets: &[ColorTargetState {
                    format: surface_texture_format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::all(),
                }],
                topology: None,
                polygon_mode: PolygonMode::Fill,
                depth_stencil: Some(DepthStencilState {
                    format: TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: true,
                    depth_compare: CompareFunction::LessEqual,
                    stencil: StencilState {
                        read_mask: !0,
                        write_mask: !0,
                        ..StencilState::default()
                    },
                    bias: DepthBiasState::default(),
                }),
                multisample: MultisampleState::default(),
            },
        );

        (
            Self {
                backend,

                textures,
                compositor,
            },
            window,
        )
    }

    pub fn create_draw_context<'a>(
        &'a self,
        stream_allocator: &'a mut StreamBufferAllocator,
    ) -> DrawContext<'a> {
        DrawContext {
            device: self.backend.device(),
            queue: self.backend.queue(),
            textures: &self.textures,
            stream_allocator,
        }
    }
}
