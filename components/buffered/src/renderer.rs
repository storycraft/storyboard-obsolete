use std::{fmt::Debug, sync::Arc};

use storyboard_core::{
    euclid::{Rect, Size2D},
    observable::Observable,
    unit::PhyiscalPixelUnit,
};
use storyboard_render::{
    component::Drawable,
    renderer::StoryboardRenderer,
    texture::{SizedTexture2D, SizedTextureView2D},
    wgpu::{
        Color, CommandEncoder, Device, LoadOp, Operations, Queue, RenderPassColorAttachment,
        TextureFormat, TextureUsages,
    },
};
use storyboard_texture::render::{data::TextureData, RenderTexture2D};

#[derive(Debug)]
pub struct StoryboardTextureRenderer {
    size: Observable<Size2D<u32, PhyiscalPixelUnit>>,

    view: SizedTextureView2D,
    render_texture: Arc<RenderTexture2D>,

    renderer: StoryboardRenderer,
}

impl StoryboardTextureRenderer {
    pub fn init(
        device: &Device,
        textures: &TextureData,
        rect: Rect<u32, PhyiscalPixelUnit>,
        screen_scale: f32,
        format: TextureFormat,
    ) -> Self {
        let renderer = StoryboardRenderer::new(rect, screen_scale, format);

        let texture = SizedTexture2D::init(
            device,
            Some("StoryboardTextureRenderer frame texture"),
            rect.size,
            format,
            TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        );

        let render_texture = Arc::new(textures.create_render_texture(
            device,
            texture.create_view_default(None).into(),
            None,
        ));
        let view = texture.create_view_default(None);

        Self {
            size: Observable::new_unchanged(rect.size),

            view,
            render_texture,

            renderer,
        }
    }

    pub fn screen_rect(&self) -> Rect<u32, PhyiscalPixelUnit> {
        self.renderer.screen_rect()
    }

    pub fn screen_scale(&self) -> f32 {
        self.renderer.screen_scale()
    }

    pub fn set_screen_rect(&mut self, rect: Rect<u32, PhyiscalPixelUnit>, screen_scale: f32) {
        if *self.size != rect.size {
            self.size = rect.size.into();
        }

        self.renderer.set_screen(rect, screen_scale);
    }

    pub fn render_texture(&self) -> &Arc<RenderTexture2D> {
        &self.render_texture
    }

    pub fn render<'a>(
        &mut self,
        device: &Device,
        queue: &Queue,
        textures: &TextureData,
        drawables: impl ExactSizeIterator<Item = &'a dyn Drawable>,
        encoder: &mut CommandEncoder,
    ) {
        if Observable::invalidate(&mut self.size) {
            let texture = SizedTexture2D::init(
                device,
                Some("StoryboardTextureRenderer frame texture"),
                *self.size,
                self.renderer.screen_format(),
                TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            );

            self.render_texture = Arc::new(textures.create_render_texture(
                device,
                texture.create_view_default(None).into(),
                None,
            ));
            self.view = texture.create_view_default(None);
        }

        self.renderer.render(
            device,
            queue,
            drawables,
            Some(RenderPassColorAttachment {
                view: self.view.inner(),
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::TRANSPARENT),
                    store: true,
                },
            }),
            encoder,
        );
    }
}