use std::fmt::Debug;

use storyboard_core::{euclid::Size2D, observable::Observable, unit::PhyiscalPixelUnit};
use wgpu::{
    self, Color, CommandEncoder, Device, LoadOp, Operations, Queue, RenderPassColorAttachment,
    TextureFormat, TextureUsages,
};

use crate::{
    component::Drawable,
    texture::{SizedTexture2D, SizedTextureView2D},
};

use super::StoryboardRenderer;

#[derive(Debug)]
pub struct StoryboardTextureRenderer {
    size: Observable<Size2D<u32, PhyiscalPixelUnit>>,

    view: Option<SizedTextureView2D>,

    renderer: StoryboardRenderer,
}

impl StoryboardTextureRenderer {
    pub fn new(
        size: Size2D<u32, PhyiscalPixelUnit>,
        screen_scale: f32,
        format: TextureFormat,
    ) -> Self {
        let renderer = StoryboardRenderer::new(size, screen_scale, format);

        Self {
            size: Observable::new_unchanged(size),
            view: None,
            renderer,
        }
    }

    pub fn set_size(&mut self, size: Size2D<u32, PhyiscalPixelUnit>, screen_scale: f32) {
        if *self.size != size {
            self.size = size.into();
        }

        self.renderer.set_screen_size(size, screen_scale);
    }

    pub fn render<'a>(
        &mut self,
        device: &Device,
        queue: &Queue,
        drawables: impl ExactSizeIterator<Item = &'a dyn Drawable>,
        encoder: &mut CommandEncoder,
    ) -> &SizedTextureView2D {
        if Observable::invalidate(&mut self.size) || self.view.is_none() {
            self.view = Some(
                SizedTexture2D::init(
                    device,
                    Some("StoryboardTextureRenderer frame texture"),
                    *self.size,
                    self.renderer.screen_format(),
                    TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                )
                .create_view_default(None),
            );
        }

        let view = self.view.as_ref().unwrap();

        self.renderer.render(
            device,
            queue,
            drawables,
            Some(RenderPassColorAttachment {
                view: view.inner(),
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::TRANSPARENT),
                    store: true,
                },
            }),
            encoder,
        );

        view
    }
}
