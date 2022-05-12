/*
 * Created on Sun May 08 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{iter, sync::Arc};

use storyboard_core::{
    euclid::Size2D,
    unit::PixelUnit,
    wgpu::{
        Color, CommandEncoderDescriptor, LoadOp, Operations, PresentMode,
        RenderPassColorAttachment, Surface, SurfaceConfiguration, TextureFormat, TextureUsages,
        TextureViewDescriptor,
    },
};

use crate::graphics::{
    backend::StoryboardBackend, compositor::ComponentCompositor, renderer::StoryboardRenderer,
};

pub struct SurfaceRenderTask<C: ComponentCompositor> {
    backend: Arc<StoryboardBackend>,

    surface_format: TextureFormat,
    surface: Surface,
    renderer: StoryboardRenderer<C>,
}

impl<C: ComponentCompositor> SurfaceRenderTask<C> {
    pub fn new(
        backend: Arc<StoryboardBackend>,
        surface: Surface,
        surface_format: TextureFormat,
        renderer: StoryboardRenderer<C>,
    ) -> Self {
        Self {
            backend,
            surface,
            surface_format,
            renderer,
        }
    }

    pub fn reconfigure(&self, size: Size2D<u32, PixelUnit>, present_mode: PresentMode) {
        self.surface.configure(
            self.backend.device(),
            &SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format: self.surface_format,
                width: size.width,
                height: size.height,
                present_mode,
            },
        );
    }

    pub fn render(&mut self, components: &[C::Component]) {
        if let Ok(surface_texture) = self.surface.get_current_texture() {
            let mut surface_encoder =
                self.backend
                    .device()
                    .create_command_encoder(&CommandEncoderDescriptor {
                        label: Some("SurfaceRenderTask surface command encoder"),
                    });

            self.renderer.render(
                &self.backend,
                components,
                &mut surface_encoder,
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
            );

            self.backend
                .queue()
                .submit(iter::once(surface_encoder.finish()));

            surface_texture.present();
        }
    }
}
