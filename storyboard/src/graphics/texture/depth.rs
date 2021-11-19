/*
 * Created on Sun Sep 19 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use euclid::Size2D;
use wgpu::{
    Device, Extent3d, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    TextureView, TextureViewDescriptor,
};

use crate::graphics::PixelUnit;

#[derive(Debug)]
pub struct DepthStencilTexture {
    texture: Texture,
    size: Size2D<u32, PixelUnit>,
    view: TextureView,
}

impl DepthStencilTexture {
    pub fn init(device: &Device, size: Size2D<u32, PixelUnit>) -> Self {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("DepthStencilTexture texture"),
            size: Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth24PlusStencil8,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
        });

        let view = texture.create_view(&TextureViewDescriptor::default());

        Self {
            texture,
            size,
            view,
        }
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    pub fn view(&self) -> &TextureView {
        &self.view
    }

    pub fn size(&self) -> &Size2D<u32, PixelUnit> {
        &self.size
    }
}
