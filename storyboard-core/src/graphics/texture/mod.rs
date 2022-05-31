/*
 * Created on Sat Apr 30 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

pub mod packed;
pub mod view;

use std::num::NonZeroU32;

use euclid::{Rect, Size2D};
use wgpu::{
    Device, Extent3d, ImageCopyTexture, ImageDataLayout, Origin3d, Queue, Texture, TextureAspect,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor,
};

use crate::unit::PixelUnit;

use self::view::SizedTextureView2D;

#[derive(Debug)]
pub struct SizedTexture2D {
    texture: Texture,
    format: TextureFormat,
    size: Size2D<u32, PixelUnit>,
}

impl SizedTexture2D {
    pub fn init(
        device: &Device,
        label: Option<&str>,
        size: Size2D<u32, PixelUnit>,
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
        size: Size2D<u32, PixelUnit>,
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

    pub const fn size(&self) -> Size2D<u32, PixelUnit> {
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

    pub fn write(&self, queue: &Queue, rect: Option<&Rect<u32, PixelUnit>>, data: &[u8]) {
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

fn rect_to_origin_extent(rect: &Rect<u32, PixelUnit>) -> (Origin3d, Extent3d) {
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
