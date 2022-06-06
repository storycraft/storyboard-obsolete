/*
 * Created on Sun May 08 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{fmt::Debug, iter, sync::Arc};

use storyboard_core::{
    euclid::Size2D,
    unit::PixelUnit,
    wgpu::{
        Color, LoadOp, Operations, PresentMode, RenderPassColorAttachment, Surface,
        SurfaceConfiguration, TextureUsages, TextureViewDescriptor,
    },
};

use crate::graphics::{
    backend::StoryboardBackend, component::Drawable, renderer::StoryboardRenderer,
    texture::TextureData,
};

#[derive(Debug)]
pub struct SurfaceRenderTask<'a> {
    backend: Arc<StoryboardBackend>,
    textures: Arc<TextureData>,

    surface: Surface,

    renderer: StoryboardRenderer<'a>,
}

impl<'a> SurfaceRenderTask<'a> {
    pub fn new(
        backend: Arc<StoryboardBackend>,
        textures: Arc<TextureData>,
        surface: Surface,
        renderer: StoryboardRenderer<'a>,
    ) -> Self {
        Self {
            backend,
            textures,
            surface,
            renderer,
        }
    }

    pub fn reconfigure(&mut self, size: Size2D<u32, PixelUnit>, present_mode: PresentMode) {
        self.renderer.screen_size = size.into();

        if self.renderer.screen_size.area() > 0 {
            self.surface.configure(
                self.backend.device(),
                &SurfaceConfiguration {
                    usage: TextureUsages::RENDER_ATTACHMENT,
                    format: self.textures.framebuffer_texture_format(),
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
        if self.renderer.screen_size.area() <= 0 {
            return;
        }

        if let Ok(surface_texture) = self.surface.get_current_texture() {
            let renderer_encoder = self.renderer.render(
                &self.backend,
                &self.textures,
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
