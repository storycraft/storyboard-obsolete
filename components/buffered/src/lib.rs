use std::{iter, sync::Arc};

use parking_lot::Mutex;
use renderer::StoryboardTextureRenderer;
use storyboard_core::{
    color::ShapeColor,
    euclid::{Point2D, Rect, Transform3D},
    unit::LogicalPixelUnit,
};
use storyboard_primitive::{PrimitiveComponent, Rectangle};
use storyboard_render::{
    component::Drawable,
    renderer::{context::DrawContext, ComponentQueue},
    wgpu::CommandEncoder,
    ScreenRect,
};
use storyboard_texture::render::data::TextureData;

pub mod renderer;

pub trait Bufferable: Drawable {
    fn bounds(&self) -> Option<Rect<f32, LogicalPixelUnit>>;
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
        let (logical_rect, physical_screen) = if let Some(rect) = self.drawable.bounds() {
            if rect.area() <= 0.0 {
                return;
            }

            let phyiscal_rect = ScreenRect::new(
                Rect::new(rect.origin, rect.size * ctx.screen.scale_factor)
                    .round_out()
                    .cast()
                    .cast_unit(),
                ctx.screen.scale_factor,
            );

            (rect.round_out(), phyiscal_rect)
        } else {
            (ctx.screen.get_logical_rect(), ctx.screen)
        };
        
        let textures = ctx.scope.backend().get::<TextureData>();

        let mut inner_renderer = self.cached_data.inner_renderer.lock();
        if inner_renderer.is_none() {
            *inner_renderer = Some(StoryboardTextureRenderer::init(
                ctx.scope.backend().device(),
                textures,
                ctx.scope.pipeline().texture_format,
                physical_screen.rect.size,
            ));
        }

        let inner_renderer = inner_renderer.as_mut().unwrap();

        inner_renderer.render(
            ctx.scope,
            physical_screen,
            textures,
            iter::once(&self.drawable as _),
            encoder,
        );

        if let Some(component) = PrimitiveComponent::from_rectangle(
            &Rectangle {
                bounds: logical_rect,
                color: ShapeColor::WHITE,
                texture: Some(inner_renderer.render_texture().clone()),
                texture_coord: [
                    Point2D::new(0.0, 0.0),
                    Point2D::new(0.0, 1.0),
                    Point2D::new(1.0, 1.0),
                    Point2D::new(1.0, 0.0),
                ],
                transform: Transform3D::identity(),
            },
            ctx,
            depth,
        ) {
            component_queue.push_transparent(component);
        }
    }
}

#[derive(Debug, Default)]
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
