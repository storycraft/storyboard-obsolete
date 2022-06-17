use std::{fmt::Debug, sync::Arc};

use crate::{
    euclid::Size2D,
    graphics::{backend::StoryboardBackend, component::Drawable, renderer::StoryboardRenderer},
    observable::Observable,
    trait_stack::TraitStack,
    unit::PhyiscalPixelUnit,
};

use wgpu::{
    self, Color, CommandBuffer, LoadOp, Operations, PresentMode, RenderPassColorAttachment,
    Surface, SurfaceTexture, TextureFormat, TextureUsages, TextureViewDescriptor,
};

#[derive(Debug)]
pub struct StoryboardSurfaceRenderer {
    surface: Surface,
    configuration: Observable<SurfaceConfiguration>,

    renderer: StoryboardRenderer,
}

impl StoryboardSurfaceRenderer {
    pub fn new(
        backend: Arc<StoryboardBackend>,
        surface: Surface,
        configuration: SurfaceConfiguration,
        screen_format: TextureFormat,
    ) -> Self {
        let renderer = StoryboardRenderer::new(
            backend,
            configuration.screen_size,
            configuration.screen_scale,
            screen_format,
        );

        Self {
            surface,
            configuration: configuration.into(),
            renderer,
        }
    }

    pub const fn backend(&self) -> &Arc<StoryboardBackend> {
        self.renderer.backend()
    }

    pub fn configuration(&self) -> SurfaceConfiguration {
        *self.configuration
    }

    pub fn set_configuration(&mut self, configuration: SurfaceConfiguration) {
        if self.configuration.ne(&configuration) {
            self.configuration = configuration.into();
        }
    }

    pub fn render(&mut self, drawables: &TraitStack<dyn Drawable>) -> Option<SurfaceRenderResult> {
        if self.renderer.screen_size().area() <= 0 {
            return None;
        }

        if Observable::invalidate(&mut self.configuration) {
            self.surface.configure(
                self.renderer.backend().device(),
                &wgpu::SurfaceConfiguration {
                    usage: TextureUsages::RENDER_ATTACHMENT,
                    format: self.renderer.screen_format(),
                    width: self.configuration.screen_size.width,
                    height: self.configuration.screen_size.height,
                    present_mode: self.configuration.present_mode,
                },
            );

            self.renderer.set_screen_size(
                self.configuration.screen_size,
                self.configuration.screen_scale,
            );
        }

        if let Ok(surface_texture) = self.surface.get_current_texture() {
            let renderer_encoder = self.renderer.render(
                drawables,
                RenderPassColorAttachment {
                    view: &surface_texture
                        .texture
                        .create_view(&TextureViewDescriptor::default()),
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                },
            )?;

            return Some(SurfaceRenderResult {
                surface_texture,
                command_buffer: renderer_encoder.finish(),
            });
        }

        return None;
    }

    pub fn into_inner(self) -> Surface {
        self.surface
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceConfiguration {
    pub present_mode: PresentMode,
    pub screen_size: Size2D<u32, PhyiscalPixelUnit>,
    pub screen_scale: f32,
}

#[derive(Debug)]
pub struct SurfaceRenderResult {
    pub surface_texture: SurfaceTexture,
    pub command_buffer: CommandBuffer,
}
