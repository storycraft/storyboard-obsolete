use std::{fmt::Debug, sync::Arc};

use storyboard_core::{euclid::Size2D, unit::PhyiscalPixelUnit};
use storyboard_render::{
    component::Drawable,
    renderer::{StoryboardRenderer, context::BackendContext},
    texture::{SizedTexture2D, SizedTextureView2D},
    wgpu::{
        Color, CommandEncoder, Device, LoadOp, Operations, RenderPassColorAttachment,
        TextureFormat, TextureUsages,
    },
    ScreenRect,
};
use storyboard_texture::render::{data::TextureData, RenderTexture2D};

#[derive(Debug)]
pub struct StoryboardTextureRenderer {
    current_screen_size: Size2D<u32, PhyiscalPixelUnit>,
    current_texture_format: TextureFormat,

    view: SizedTextureView2D,
    render_texture: Arc<RenderTexture2D>,

    renderer: StoryboardRenderer,
}

impl StoryboardTextureRenderer {
    pub fn init(
        device: &Device,
        textures: &TextureData,
        texture_format: TextureFormat,
        screen_size: Size2D<u32, PhyiscalPixelUnit>,
    ) -> Self {
        let renderer = StoryboardRenderer::new();

        let texture = SizedTexture2D::init(
            device,
            Some("StoryboardTextureRenderer frame texture"),
            screen_size,
            texture_format,
            TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        );

        let render_texture = Arc::new(textures.create_render_texture(
            device,
            texture.create_view_default(None).into(),
            None,
        ));
        let view = texture.create_view_default(None);

        Self {
            current_screen_size: screen_size,
            current_texture_format: texture_format,

            view,
            render_texture,

            renderer,
        }
    }

    pub const fn current_texture_format(&self) -> TextureFormat {
        self.current_texture_format
    }

    pub fn render_texture(&self) -> &Arc<RenderTexture2D> {
        &self.render_texture
    }

    pub fn render<'a>(
        &mut self,
        backend: BackendContext<'a>,
        screen: ScreenRect,
        textures: &TextureData,
        drawables: impl ExactSizeIterator<Item = &'a dyn Drawable>,
        encoder: &mut CommandEncoder,
    ) {
        if self.current_screen_size != screen.rect.size
            || !backend.renderer_data.is_valid(self.current_texture_format)
        {
            let texture = SizedTexture2D::init(
                backend.device,
                Some("StoryboardTextureRenderer frame texture"),
                screen.rect.size,
                backend.screen_format(),
                TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            );

            self.render_texture = Arc::new(textures.create_render_texture(
                backend.device,
                texture.create_view_default(None).into(),
                None,
            ));
            self.view = texture.create_view_default(None);
        }

        self.renderer.render(
            backend,
            screen,
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
