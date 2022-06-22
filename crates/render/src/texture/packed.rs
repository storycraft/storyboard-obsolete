use rect_packer::DensePacker;
use storyboard_core::{euclid::{Size2D, Rect, Point2D}, unit::PhyiscalPixelUnit};
use std::{fmt::Debug, num::NonZeroU32};
use wgpu::{Extent3d, ImageCopyTexture, ImageDataLayout, Origin3d, Queue, TextureAspect};

use super::SizedTexture2D;

pub struct PackedTexture {
    texture: SizedTexture2D,
    packer: DensePacker,
}

impl PackedTexture {
    pub fn new(texture: SizedTexture2D) -> Self {
        let packer = DensePacker::new(
            texture.size().width.try_into().unwrap(),
            texture.size().height.try_into().unwrap(),
        );

        Self { texture, packer }
    }

    pub fn pack(
        &mut self,
        queue: &Queue,
        size: Size2D<u32, PhyiscalPixelUnit>,
        data: &[u8],
    ) -> Option<Rect<u32, PhyiscalPixelUnit>> {
        let rect = self
            .packer
            .pack(size.width as i32, size.height as i32, false)?;

        queue.write_texture(
            ImageCopyTexture {
                texture: self.texture.inner(),
                mip_level: 0,
                origin: Origin3d {
                    x: rect.x as u32,
                    y: rect.y as u32,
                    z: 0,
                },
                aspect: TextureAspect::All,
            },
            data,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(
                    size.width as u32 * self.texture.format().describe().block_size as u32,
                ),
                rows_per_image: NonZeroU32::new(size.height as u32),
            },
            Extent3d {
                width: size.width as u32,
                height: size.height as u32,
                depth_or_array_layers: 1,
            },
        );

        Some(Rect::new(
            Point2D::new(rect.x as u32, rect.y as u32),
            size.cast(),
        ))
    }

    pub fn reset(&mut self) {
        let (width, height) = self.packer.size();
        self.packer = DensePacker::new(width, height);
    }

    pub fn inner(&self) -> &SizedTexture2D {
        &self.texture
    }

    pub fn finish(self) -> SizedTexture2D {
        self.texture
    }
}

impl Debug for PackedTexture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PackedTexture")
            .field("texture", &self.texture)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PackedArea {
    pub x: u32,
    pub y: u32,

    pub width: u32,
    pub height: u32,
}
