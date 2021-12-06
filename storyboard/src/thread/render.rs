/*
 * Created on Thu Nov 18 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{
    any::Any,
    cell::Cell,
    iter,
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, LockResult, Mutex, MutexGuard,
    },
    thread::{self, JoinHandle},
    time::Instant,
};

use std::fmt::Debug;

use euclid::Size2D;
use ring_channel::{ring_channel, RingSender};
use wgpu::{
    BufferUsages, Color, CommandEncoderDescriptor, Device, LoadOp, Operations, PresentMode, Queue,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor, Surface,
    SurfaceConfiguration, TextureFormat, TextureUsages, TextureViewDescriptor,
};

use crate::{
    graphics::{
        backend::StoryboardBackend,
        buffer::stream::StreamBufferAllocator,
        context::DrawContext,
        pass::StoryboardRenderPass,
        renderer::{RenderData, StoryboardRenderer},
        texture::{depth::DepthStencilTexture, TextureData},
        PixelUnit,
    },
    observable::Observable,
};

pub struct RenderThread {
    handle: Option<JoinHandle<()>>,

    fps_sample: Arc<AtomicU64>,

    projector: Option<Arc<Mutex<SurfaceProjector>>>,

    current_present_mode: Cell<PresentMode>,

    sender: Option<RingSender<RenderOperation>>,
}

impl RenderThread {
    pub fn run(
        backend: &StoryboardBackend,
        surface: Surface,
        surface_format: TextureFormat,
        render_data: Arc<RenderData>,
        texture_data: Arc<TextureData>,
        configuration: RenderConfiguration,
    ) -> Self {
        let current_present_mode = Cell::new(configuration.present_mode);

        let fps_sample = Arc::new(AtomicU64::new(0));
        let render_fps_sample = fps_sample.clone();

        let (sender, mut receiver) = ring_channel(NonZeroUsize::new(1).unwrap());

        let projector = Arc::new(Mutex::new(SurfaceProjector::init(
            backend.device().clone(),
            backend.queue().clone(),
            texture_data,
            render_data,
            surface,
            surface_format,
            configuration,
        )));
        let render_projector = projector.clone();

        let handle = thread::spawn(move || {
            let projector = render_projector;
            let fps_sample = render_fps_sample;

            while let Ok(operation) = receiver.recv() {
                let instant = Instant::now();

                if let Ok(mut projector) = projector.lock() {
                    projector.process(operation);
                }

                fps_sample.store(instant.elapsed().as_micros() as u64, Ordering::Relaxed);
            }
        });

        Self {
            handle: Some(handle),

            fps_sample,

            projector: Some(projector),
            current_present_mode,

            sender: Some(sender),
        }
    }

    pub fn submit(&mut self, operation: RenderOperation) -> bool {
        if let Some(sender) = &mut self.sender {
            sender.send(operation).is_ok()
        } else {
            false
        }
    }

    fn try_get_projector(&self) -> Option<LockResult<MutexGuard<SurfaceProjector>>> {
        self.projector.as_ref().map(|projector| projector.lock())
    }

    pub fn resize_surface(&self, size: Size2D<u32, PixelUnit>) {
        if let Some(Ok(mut projector)) = self.try_get_projector() {
            projector.configuration_mut().size = size;
        }
    }

    pub fn set_present_mode(&self, present_mode: PresentMode) {
        if let Some(Ok(mut projector)) = self.try_get_projector() {
            projector.configuration_mut().present_mode = present_mode;
            self.current_present_mode.set(present_mode);
        }
    }

    pub fn present_mode(&self) -> PresentMode {
        self.current_present_mode.get()
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

    pub const fn finished(&self) -> bool {
        self.handle.is_none()
    }

    pub fn join(&mut self) -> Option<Result<Surface, Box<dyn Any + Send + 'static>>> {
        match self.handle.take()?.join() {
            Ok(_) => {
                if let Some(projector) = self.projector.take() {
                    Some(Ok(Arc::try_unwrap(projector)
                        .unwrap()
                        .into_inner()
                        .unwrap()
                        .into_inner()))
                } else {
                    None
                }
            }

            Err(err) => Some(Err(err)),
        }
    }
}

impl Debug for RenderThread {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderThread")
            .field("handle", &self.handle)
            .field("fps_sample", &self.fps_sample)
            .finish()
    }
}

impl Drop for RenderThread {
    fn drop(&mut self) {
        self.interrupt();
    }
}

#[derive(Debug)]
pub struct SurfaceProjector {
    device: Arc<Device>,
    queue: Arc<Queue>,

    texture_data: Arc<TextureData>,
    render_data: Arc<RenderData>,

    surface: Surface,
    surface_format: TextureFormat,

    stream_allocator: StreamBufferAllocator,

    surface_depth_stencil: DepthStencilTexture,

    configuration: Observable<RenderConfiguration>,
}

impl SurfaceProjector {
    pub fn init(
        device: Arc<Device>,
        queue: Arc<Queue>,

        texture_data: Arc<TextureData>,
        render_data: Arc<RenderData>,

        surface: Surface,
        surface_format: TextureFormat,

        configuration: RenderConfiguration,
    ) -> Self {
        let surface_depth_stencil = DepthStencilTexture::init(&device, configuration.size);

        Self {
            device,
            queue,

            texture_data,
            render_data,

            surface,
            surface_format,

            stream_allocator: StreamBufferAllocator::new(
                BufferUsages::INDEX | BufferUsages::VERTEX,
            ),

            surface_depth_stencil,

            configuration: Observable::new(configuration),
        }
    }

    pub fn configuration(&self) -> &RenderConfiguration {
        self.configuration.inner_ref()
    }

    pub fn configuration_mut(&mut self) -> &mut RenderConfiguration {
        self.configuration.inner_mut()
    }

    pub fn set_configuration(&mut self, configuration: RenderConfiguration) {
        self.configuration.set(configuration);
    }

    pub fn process(&mut self, mut surface_operation: RenderOperation) {
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("RenderThread main encoder"),
            });

        let mut draw_ctx = DrawContext {
            device: &self.device,
            queue: &self.queue,
            textures: &self.texture_data,
            stream_allocator: &mut self.stream_allocator,
        };

        surface_operation
            .renderer
            .prepare(&mut draw_ctx, &mut encoder);

        let render_ctx = draw_ctx.into_render_context(&self.render_data);

        if self.configuration.unmark() {
            let config = self.configuration.inner_ref();

            self.surface.configure(
                &self.device,
                &SurfaceConfiguration {
                    usage: TextureUsages::RENDER_ATTACHMENT,
                    format: self.surface_format,
                    width: config.size.width,
                    height: config.size.height,
                    present_mode: config.present_mode,
                },
            );
            self.surface_depth_stencil = DepthStencilTexture::init(&self.device, config.size);
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

                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: self.surface_depth_stencil.view(),
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

            surface_operation
                .renderer
                .render(&render_ctx, &mut StoryboardRenderPass::new(pass));

            self.queue.submit(iter::once(encoder.finish()));

            surface_texture.present();
        }
    }

    pub fn into_inner(self) -> Surface {
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
