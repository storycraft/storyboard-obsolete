/*
 * Created on Sun May 08 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{fmt::Debug, iter, sync::Arc};

use storyboard_core::{
    euclid::Size2D,
    graphics::{backend::StoryboardBackend, component::Drawable, renderer::StoryboardRenderer},
    unit::PixelUnit,
    wgpu::{
        Color, LoadOp, Operations, PresentMode, RenderPassColorAttachment, Surface,
        SurfaceConfiguration, TextureUsages, TextureViewDescriptor,
    },
};

#[derive(Debug)]
pub struct SurfaceRenderTask {
    backend: Arc<StoryboardBackend>,

    surface: Surface,

    renderer: StoryboardRenderer,
}

impl SurfaceRenderTask {
    pub fn new(
        backend: Arc<StoryboardBackend>,
        surface: Surface,
        renderer: StoryboardRenderer,
    ) -> Self {
        Self {
            backend,
            surface,
            renderer,
        }
    }

    pub fn reconfigure(&mut self, size: Size2D<u32, PixelUnit>, present_mode: PresentMode) {
        self.renderer.set_screen_size(size);

        if size.area() > 0 {
            self.surface.configure(
                self.backend.device(),
                &SurfaceConfiguration {
                    usage: TextureUsages::RENDER_ATTACHMENT,
                    format: self.renderer.screen_format(),
                    width: size.width,
                    height: size.height,
                    present_mode,
                },
            );
        }
    }

    pub fn push(&mut self, drawable: impl Drawable + 'static) {
        self.renderer.push(drawable)
    }

    pub fn render(&mut self) {
        if self.renderer.screen_size().area() <= 0 {
            return;
        }

        if let Ok(surface_texture) = self.surface.get_current_texture() {
            let renderer_encoder = self.renderer.render(
                &self.backend,
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
            );

            if let Some(renderer_encoder) = renderer_encoder {
                self.backend
                    .queue()
                    .submit(iter::once(renderer_encoder.finish()));

                surface_texture.present();
            }
        }
    }
}
