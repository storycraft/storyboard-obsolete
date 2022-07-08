use std::{iter, sync::Arc};

use parking_lot::Mutex;
use renderer::StoryboardTextureRenderer;
use storyboard_core::{
    color::ShapeColor,
    euclid::{Rect, Transform3D},
    unit::LogicalPixelUnit,
};
use storyboard_primitive::{PrimitiveComponent, Rectangle};
use storyboard_render::{
    component::Drawable,
    renderer::{context::DrawContext, ComponentQueue},
    wgpu::CommandEncoder, ScreenRect,
};
use storyboard_texture::render::data::TextureData;

pub mod renderer;

pub trait Bufferable: Drawable {
    fn bounds(&self) -> Rect<u32, LogicalPixelUnit>;
}

#[derive(Debug)]
pub struct BufferedDrawable<T> {
    pub drawable: T,
    pub cached_data: Arc<CachedBufferData>,
}

impl<T: Bufferable> Drawable for BufferedDrawable<T> {
    fn prepare(
        &self,
        component_queue: &mut ComponentQueue,
        ctx: &mut DrawContext,
        encoder: &mut CommandEncoder,
        depth: f32,
    ) {
        let target_bounds = self.drawable.bounds();
        if target_bounds.area() == 0 {
            return;
        }

        let mut phyiscal_size = target_bounds.size;
        phyiscal_size.width = (phyiscal_size.width as f32 * ctx.pixel_density).ceil() as _;
        phyiscal_size.height = (phyiscal_size.height as f32 * ctx.pixel_density).ceil() as _;

        let rect = Rect::new(target_bounds.origin, phyiscal_size).cast_unit();

        let textures = ctx.get::<TextureData>();

        let mut inner_renderer = self.cached_data.inner_renderer.lock();
        if inner_renderer.is_none() {
            *inner_renderer = Some(StoryboardTextureRenderer::init(
                ctx.backend.device,
                textures,
                ctx.backend.renderer_data.screen_format(),
                rect.size,
            ));
        }

        let inner_renderer = inner_renderer.as_mut().unwrap();

        inner_renderer.render(
            ctx.backend.device,
            ctx.backend.queue,
            ScreenRect::new(rect, ctx.pixel_density),
            textures,
            iter::once(&self.drawable as _),
            ctx.backend.renderer_data,
            encoder,
        );

        if let Some(component) = PrimitiveComponent::from_rectangle(
            &Rectangle {
                bounds: target_bounds.cast(),
                color: ShapeColor::WHITE,
                texture: Some(inner_renderer.render_texture().clone()),
                transform: Transform3D::identity(),
            },
            ctx,
            depth,
        ) {
            component_queue.push_transparent(component);
        }
    }
}

#[derive(Debug)]
pub struct CachedBufferData {
    inner_renderer: Mutex<Option<StoryboardTextureRenderer>>,
}

impl CachedBufferData {
    pub fn new() -> Self {
        Self {
            inner_renderer: Mutex::new(None),
        }
    }
}
