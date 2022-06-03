/*
 * Created on Sun May 08 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{fmt::Debug, iter, sync::Arc};

use storyboard_core::{
    euclid::{Size2D, Transform3D},
    observable::Observable,
    unit::{PixelUnit, RenderUnit},
    wgpu::{
        Color, CommandEncoderDescriptor, LoadOp, Operations, PresentMode,
        RenderPassColorAttachment, Surface, SurfaceConfiguration, TextureUsages,
        TextureViewDescriptor,
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

    surface_size: Observable<Size2D<u32, PixelUnit>>,
    screen_matrix: Transform3D<f32, PixelUnit, RenderUnit>,

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
            surface_size: Size2D::zero().into(),
            screen_matrix: Transform3D::identity(),
            renderer,
        }
    }

    pub fn reconfigure(&mut self, size: Size2D<u32, PixelUnit>, present_mode: PresentMode) {
        self.surface_size = size.into();

        if self.surface_size.area() > 0 {
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
        if self.surface_size.area() <= 0 {
            return;
        }

        if let Ok(surface_texture) = self.surface.get_current_texture() {
            let mut surface_encoder =
                self.backend
                    .device()
                    .create_command_encoder(&CommandEncoderDescriptor {
                        label: Some("SurfaceRenderTask surface command encoder"),
                    });

            if Observable::unmark(&mut self.surface_size) {
                self.screen_matrix = Transform3D::ortho(
                    0.0_f32,
                    self.surface_size.width as f32,
                    self.surface_size.height as f32,
                    0.0,
                    0.0,
                    1.0,
                );
            }

            self.renderer.render(
                &self.backend,
                &self.textures,
                &self.screen_matrix,
                &[RenderPassColorAttachment {
                    view: &surface_texture
                        .texture
                        .create_view(&TextureViewDescriptor::default()),
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                }],
                &mut surface_encoder,
            );

            self.backend
                .queue()
                .submit(iter::once(surface_encoder.finish()));

            surface_texture.present();
        }
    }
}
