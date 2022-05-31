/*
 * Created on Sun May 01 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use euclid::{Point2D, Rect, Size2D};
use wgpu::{TextureView, TextureViewDescriptor};

use crate::unit::{PixelUnit, TextureUnit};

use super::SizedTexture2D;

#[derive(Debug)]
pub struct SizedTextureView2D {
    view: TextureView,
    size: Size2D<u32, PixelUnit>,
}

impl SizedTextureView2D {
    pub fn init(sized_texture: &SizedTexture2D, desc: &TextureViewDescriptor) -> Self {
        let view = sized_texture.inner().create_view(desc);
        let size = sized_texture.size();

        Self::from_view(view, size)
    }

    pub fn from_view(view: TextureView, size: Size2D<u32, PixelUnit>) -> Self {
        Self { view, size }
    }

    pub const fn inner(&self) -> &TextureView {
        &self.view
    }

    pub const fn size(&self) -> Size2D<u32, PixelUnit> {
        self.size
    }

    pub const fn render_rect(&self) -> Rect<f32, TextureUnit> {
        Rect::new(Point2D::new(0.0, 0.0), Size2D::new(1.0, 1.0))
    }

    pub fn slice(self, rect: Rect<u32, PixelUnit>) -> TextureView2D {
        TextureView2D::Partial(PartialTextureView2D::new(self, rect))
    }

    pub fn into_inner(self) -> TextureView {
        self.view
    }
}

#[derive(Debug)]
pub enum TextureView2D {
    All(SizedTextureView2D),
    Partial(PartialTextureView2D),
}

impl TextureView2D {
    pub fn inner(&self) -> &SizedTextureView2D {
        match self {
            TextureView2D::All(view) => &view,
            TextureView2D::Partial(partial) => partial.view(),
        }
    }

    /// Slice view into partial
    pub fn slice(self, rect: Rect<u32, PixelUnit>) -> TextureView2D {
        match self {
            TextureView2D::All(view) => view.slice(rect),

            TextureView2D::Partial(partial) => TextureView2D::Partial(partial.slice(rect)),
        }
    }

    pub const fn origin(&self) -> Point2D<u32, PixelUnit> {
        match self {
            TextureView2D::All(_) => Point2D::new(0, 0),
            TextureView2D::Partial(partial) => partial.rect.origin,
        }
    }

    pub const fn size(&self) -> Size2D<u32, PixelUnit> {
        match self {
            TextureView2D::All(view) => view.size(),
            TextureView2D::Partial(partial) => partial.rect.size,
        }
    }

    pub const fn rect(&self) -> Rect<u32, PixelUnit> {
        match self {
            TextureView2D::All(view) => Rect::new(Point2D::new(0, 0), view.size),
            TextureView2D::Partial(partial) => partial.rect,
        }
    }

    pub fn render_rect(&self) -> Rect<f32, TextureUnit> {
        match self {
            TextureView2D::All(view) => view.render_rect(),
            TextureView2D::Partial(partial) => partial.render_rect(),
        }
    }

    pub fn into_view_coord(&self, coord: Point2D<f32, TextureUnit>) -> Point2D<f32, TextureUnit> {
        let rect = self.render_rect();

        rect.origin + coord.to_vector().component_mul(rect.size.to_vector())
    }
}

impl From<SizedTextureView2D> for TextureView2D {
    fn from(sized: SizedTextureView2D) -> Self {
        TextureView2D::All(sized)
    }
}

impl From<PartialTextureView2D> for TextureView2D {
    fn from(partial: PartialTextureView2D) -> Self {
        TextureView2D::Partial(partial)
    }
}

#[derive(Debug)]
pub struct PartialTextureView2D {
    view: SizedTextureView2D,

    pub rect: Rect<u32, PixelUnit>,
}
impl PartialTextureView2D {
    pub const fn new(view: SizedTextureView2D, rect: Rect<u32, PixelUnit>) -> Self {
        Self { view, rect }
    }

    /// Slice partial view. The offset and size must be larger than (0, 0) or it will be clamped to zero
    pub fn slice(self, inner_rect: Rect<u32, PixelUnit>) -> PartialTextureView2D {
        let offset = inner_rect.origin.max(Point2D::zero());
        let size = inner_rect.size.max(Size2D::zero());

        Self {
            view: self.view,
            rect: Rect::new(
                offset,
                (inner_rect.size - offset.to_vector().to_size() - size).max(Size2D::zero()),
            ),
        }
    }

    pub const fn view(&self) -> &SizedTextureView2D {
        &self.view
    }

    pub fn render_rect(&self) -> Rect<f32, TextureUnit> {
        Rect::new(
            Point2D::new(
                self.rect.origin.x as f32 / self.view.size.width as f32,
                self.rect.origin.y as f32 / self.view.size.height as f32,
            ),
            Size2D::new(
                self.rect.size.width as f32 / self.view.size.width as f32,
                self.rect.size.height as f32 / self.view.size.height as f32,
            ),
        )
    }
}
