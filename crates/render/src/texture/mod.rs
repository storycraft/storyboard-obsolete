pub mod packed;

use std::num::NonZeroU32;

use storyboard_core::{
    euclid::{Point2D, Rect, Size2D},
    unit::{PhyiscalPixelUnit, TextureUnit},
};
use wgpu::{
    Device, Extent3d, ImageCopyTexture, ImageDataLayout, Origin3d, Queue, Texture, TextureAspect,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor,
};

#[derive(Debug)]
pub struct SizedTexture2D {
    texture: Texture,
    format: TextureFormat,
    size: Size2D<u32, PhyiscalPixelUnit>,
}

impl SizedTexture2D {
    pub fn init(
        device: &Device,
        label: Option<&str>,
        size: Size2D<u32, PhyiscalPixelUnit>,
        format: TextureFormat,
        usage: TextureUsages,
    ) -> Self {
        let texture = device.create_texture(&TextureDescriptor {
            label,
            size: Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage,
        });

        Self::from_texture(texture, format, size)
    }

    pub fn from_texture(
        texture: Texture,
        format: TextureFormat,
        size: Size2D<u32, PhyiscalPixelUnit>,
    ) -> Self {
        Self {
            texture,
            format,
            size,
        }
    }

    pub const fn inner(&self) -> &Texture {
        &self.texture
    }

    pub const fn format(&self) -> TextureFormat {
        self.format
    }

    pub const fn size(&self) -> Size2D<u32, PhyiscalPixelUnit> {
        self.size
    }

    pub fn create_view(&self, desc: &TextureViewDescriptor) -> SizedTextureView2D {
        SizedTextureView2D::init(self, desc)
    }

    pub fn create_view_default(&self, label: Option<&str>) -> SizedTextureView2D {
        self.create_view(&TextureViewDescriptor {
            label,
            ..Default::default()
        })
    }

    pub fn write(&self, queue: &Queue, rect: Option<&Rect<u32, PhyiscalPixelUnit>>, data: &[u8]) {
        let (origin, extent) = match rect {
            Some(rect) => rect_to_origin_extent(&rect),

            None => (
                Origin3d::ZERO,
                Extent3d {
                    width: self.size.width,
                    height: self.size.height,
                    depth_or_array_layers: 1,
                },
            ),
        };

        let format_info = self.format.describe();

        queue.write_texture(
            ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin,
                aspect: TextureAspect::All,
            },
            data,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(
                    // TODO:: Compressed texture size handling
                    extent.width * format_info.block_size as u32,
                ),
                rows_per_image: NonZeroU32::new(extent.height),
            },
            extent,
        );
    }

    pub fn into_inner(self) -> Texture {
        self.texture
    }
}

#[derive(Debug)]
pub struct SizedTextureView2D {
    view: TextureView,
    size: Size2D<u32, PhyiscalPixelUnit>,
}

impl SizedTextureView2D {
    pub fn init(sized_texture: &SizedTexture2D, desc: &TextureViewDescriptor) -> Self {
        let view = sized_texture.inner().create_view(desc);
        let size = sized_texture.size();

        Self::from_view(view, size)
    }

    pub const fn from_view(view: TextureView, size: Size2D<u32, PhyiscalPixelUnit>) -> Self {
        Self { view, size }
    }

    pub const fn inner(&self) -> &TextureView {
        &self.view
    }

    pub const fn size(&self) -> Size2D<u32, PhyiscalPixelUnit> {
        self.size
    }

    pub const fn texture_rect(&self) -> Rect<f32, TextureUnit> {
        Rect::new(Point2D::new(0.0, 0.0), Size2D::new(1.0, 1.0))
    }

    pub fn slice(self, rect: Rect<u32, PhyiscalPixelUnit>) -> TextureView2D {
        TextureView2D::Partial(PartialTextureView2D::new(self, rect))
    }

    pub fn to_texture_rect(&self, rect: Rect<u32, PhyiscalPixelUnit>) -> Rect<f32, TextureUnit> {
        rect.cast()
            .cast_unit()
            .scale(1.0 / self.size.width as f32, 1.0 / self.size.height as f32)
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
    pub fn slice(self, rect: Rect<u32, PhyiscalPixelUnit>) -> TextureView2D {
        match self {
            TextureView2D::All(view) => view.slice(rect),

            TextureView2D::Partial(partial) => TextureView2D::Partial(partial.slice(rect)),
        }
    }

    /// Slice view into partial
    pub fn to_texture_rect(&self, rect: Rect<u32, PhyiscalPixelUnit>) -> Rect<f32, TextureUnit> {
        match self {
            TextureView2D::All(view) => view.to_texture_rect(rect),

            TextureView2D::Partial(partial) => partial.to_texture_rect(rect),
        }
    }

    pub const fn origin(&self) -> Point2D<u32, PhyiscalPixelUnit> {
        match self {
            TextureView2D::All(_) => Point2D::new(0, 0),
            TextureView2D::Partial(partial) => partial.rect.origin,
        }
    }

    pub const fn size(&self) -> Size2D<u32, PhyiscalPixelUnit> {
        match self {
            TextureView2D::All(view) => view.size(),
            TextureView2D::Partial(partial) => partial.rect.size,
        }
    }

    pub const fn rect(&self) -> Rect<u32, PhyiscalPixelUnit> {
        match self {
            TextureView2D::All(view) => Rect::new(Point2D::new(0, 0), view.size),
            TextureView2D::Partial(partial) => partial.rect,
        }
    }

    pub fn texture_rect(&self) -> Rect<f32, TextureUnit> {
        match self {
            TextureView2D::All(view) => view.texture_rect(),
            TextureView2D::Partial(partial) => partial.texture_rect(),
        }
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

    pub rect: Rect<u32, PhyiscalPixelUnit>,
}
impl PartialTextureView2D {
    pub const fn new(view: SizedTextureView2D, rect: Rect<u32, PhyiscalPixelUnit>) -> Self {
        Self { view, rect }
    }

    /// Slice partial view. The offset and size must be larger than (0, 0) or it will be clamped to zero
    pub fn slice(self, inner_rect: Rect<u32, PhyiscalPixelUnit>) -> PartialTextureView2D {
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

    pub fn texture_rect(&self) -> Rect<f32, TextureUnit> {
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

    pub fn to_texture_rect(&self, rect: Rect<u32, PhyiscalPixelUnit>) -> Rect<f32, TextureUnit> {
        rect.translate(self.rect.origin.to_vector())
            .cast()
            .cast_unit()
            .scale(
                1.0 / self.view.size.width as f32,
                1.0 / self.view.size.height as f32,
            )
    }
}

fn rect_to_origin_extent(rect: &Rect<u32, PhyiscalPixelUnit>) -> (Origin3d, Extent3d) {
    (
        Origin3d {
            x: rect.origin.x,
            y: rect.origin.y,
            z: 0,
        },
        Extent3d {
            width: rect.size.width,
            height: rect.size.height,
            depth_or_array_layers: 1,
        },
    )
}
