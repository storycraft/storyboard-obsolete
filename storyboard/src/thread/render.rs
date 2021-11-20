/*
 * Created on Thu Nov 18 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{
    any::Any,
    iter,
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, RwLock, RwLockReadGuard, RwLockWriteGuard,
    },
    thread::{self, JoinHandle},
    time::Instant,
};

use std::fmt::Debug;

use euclid::Size2D;
use ring_channel::{ring_channel, RingSender};
use wgpu::{
    BufferUsages, Color, CommandEncoderDescriptor, Operations, PresentMode,
    RenderPassColorAttachment, RenderPassDescriptor, Surface, SurfaceConfiguration, TextureFormat,
    TextureUsages, TextureView, TextureViewDescriptor,
};

use crate::{
    graphics::{
        backend::StoryboardBackend,
        buffer::stream::StreamBufferAllocator,
        context::DrawContext,
        pass::StoryboardRenderPass,
        renderer::{RenderData, StoryboardRenderer},
        texture::TextureData,
        PixelUnit,
    },
    observable::Observable,
};

pub struct RenderThread {
    handle: Option<JoinHandle<Surface>>,

    fps_sample: Arc<AtomicU64>,

    configuration: Arc<RwLock<Observable<RenderConfiguration>>>,

    sender: Option<RingSender<RenderQueue>>,
}

impl RenderThread {
    pub fn run(
        backend: Arc<StoryboardBackend>,
        surface: Surface,
        surface_format: TextureFormat,
        render_data: Arc<RenderData>,
        texture_data: Arc<TextureData>,
        configuration: RenderConfiguration,
    ) -> Self {
        let configuration = Arc::new(RwLock::new(Observable::new(configuration)));
        let render_configuration = configuration.clone();

        let fps_sample = Arc::new(AtomicU64::new(0));
        let render_fps_sample = fps_sample.clone();

        let (sender, mut receiver) = ring_channel(NonZeroUsize::new(1).unwrap());

        let handle = thread::spawn(move || {
            let render_fps_sample = render_fps_sample;

            let mut processor = SurfaceRenderProcessor {
                backend,
                texture_data,
                render_data,
                surface,
                surface_format,
                stream_allocator: StreamBufferAllocator::new(
                    BufferUsages::INDEX | BufferUsages::VERTEX,
                ),
                configuration: render_configuration,
            };

            while let Ok(render_queue) = receiver.recv() {
                let instant = Instant::now();

                processor.process(render_queue);

                render_fps_sample.store(instant.elapsed().as_micros() as u64, Ordering::Relaxed);
            }

            processor.inner()
        });

        Self {
            handle: Some(handle),

            fps_sample,

            configuration,

            sender: Some(sender),
        }
    }

    pub fn submit_queue(&mut self, queue: RenderQueue) -> bool {
        if let Some(sender) = &mut self.sender {
            sender.send(queue).is_ok()
        } else {
            false
        }
    }

    pub fn configuration(&self) -> RwLockReadGuard<Observable<RenderConfiguration>> {
        self.configuration.read().unwrap()
    }

    pub fn configuration_mut(&self) -> RwLockWriteGuard<Observable<RenderConfiguration>> {
        self.configuration.write().unwrap()
    }

    pub fn fps(&self) -> f64 {
        1_000_000.0 / self.fps_sample.load(Ordering::Relaxed) as f64
    }

    pub fn interrupt(&mut self) {
        self.sender.take();
    }

    pub const fn interrupted(&self) -> bool {
        self.sender.is_none()
    }

    pub fn join(&mut self) -> Option<Result<Surface, Box<dyn Any + Send + 'static>>> {
        if let Some(handle) = self.handle.take() {
            Some(handle.join())
        } else {
            None
        }
    }
}

impl Debug for RenderThread {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderThread")
            .field("handle", &self.handle)
            .field("fps_sample", &self.fps_sample)
            .field("configuration", &self.configuration)
            .finish()
    }
}

impl Drop for RenderThread {
    fn drop(&mut self) {
        self.interrupt();
        self.join();
    }
}

pub struct SurfaceRenderProcessor {
    backend: Arc<StoryboardBackend>,

    texture_data: Arc<TextureData>,
    render_data: Arc<RenderData>,

    surface: Surface,
    surface_format: TextureFormat,

    stream_allocator: StreamBufferAllocator,

    configuration: Arc<RwLock<Observable<RenderConfiguration>>>,
}

impl SurfaceRenderProcessor {
    pub fn process(&mut self, mut render_queue: RenderQueue) {
        if !render_queue.is_empty() {
            let mut encoder =
                self.backend
                    .device()
                    .create_command_encoder(&CommandEncoderDescriptor {
                        label: Some("RenderThread main encoder"),
                    });

            let mut draw_ctx = DrawContext {
                device: self.backend.device(),
                queue: self.backend.queue(),
                textures: &self.texture_data,
                stream_allocator: &mut self.stream_allocator,
            };

            for (_, operation) in &mut render_queue.tasks {
                operation.renderer.prepare(&mut draw_ctx, &mut encoder);
            }

            if let Some(surface_operation) = &mut render_queue.surface_task {
                surface_operation
                    .renderer
                    .prepare(&mut draw_ctx, &mut encoder);
            }

            let render_ctx = draw_ctx.into_render_context(&self.render_data);

            for (view, mut operation) in render_queue.tasks {
                let pass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("Storyboard RenderThread texture render pass"),
                    color_attachments: &[RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: operation.operations,
                    }],

                    // TODO
                    depth_stencil_attachment: None,
                });

                operation
                    .renderer
                    .render(&render_ctx, &mut StoryboardRenderPass::new(pass));
            }

            if let Some(mut surface_operation) = render_queue.surface_task {
                if let Ok(mut configuration) = self.configuration.write() {
                    if configuration.unmark() {
                        let config = configuration.inner_ref();
                        self.surface.configure(
                            self.backend.device(),
                            &SurfaceConfiguration {
                                usage: TextureUsages::RENDER_ATTACHMENT,
                                format: self.surface_format,
                                width: config.size.width,
                                height: config.size.height,
                                present_mode: config.present_mode,
                            },
                        );
                    }
                }

                if let Ok(surface_texture) = self.surface.get_current_texture() {
                    let view = surface_texture
                        .texture
                        .create_view(&TextureViewDescriptor::default());

                    let pass = encoder.begin_render_pass(&RenderPassDescriptor {
                        label: Some("Storyboard RenderThread surface render pass"),
                        color_attachments: &[RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: surface_operation.operations,
                        }],

                        // TODO
                        depth_stencil_attachment: None,
                    });

                    surface_operation
                        .renderer
                        .render(&render_ctx, &mut StoryboardRenderPass::new(pass));

                    self.backend.queue().submit(iter::once(encoder.finish()));

                    surface_texture.present();
                    return;
                }
            }

            self.backend.queue().submit(iter::once(encoder.finish()));
        }
    }

    pub fn inner(self) -> Surface {
        self.surface
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RenderConfiguration {
    pub size: Size2D<u32, PixelUnit>,
    pub present_mode: PresentMode,
}

pub struct RenderOperation {
    pub operations: Operations<Color>,
    pub renderer: StoryboardRenderer<'static>,
}

impl Debug for RenderOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderOperation")
            .field("operations", &self.operations)
            .finish()
    }
}

#[derive(Debug)]
pub struct RenderQueue {
    surface_task: Option<RenderOperation>,
    tasks: Vec<(TextureView, RenderOperation)>,
}

impl RenderQueue {
    pub const fn new() -> Self {
        Self {
            surface_task: None,
            tasks: Vec::new(),
        }
    }

    pub fn push_texture_task(&mut self, view: TextureView, operation: RenderOperation) {
        self.tasks.push((view, operation));
    }

    pub fn set_surface_task(&mut self, operation: RenderOperation) {
        self.surface_task = Some(operation);
    }

    pub fn is_empty(&self) -> bool {
        self.surface_task.is_none() && self.tasks.is_empty()
    }
}
