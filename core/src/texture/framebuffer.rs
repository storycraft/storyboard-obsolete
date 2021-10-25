/*
 * Created on Fri Oct 01 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use euclid::{Rect, Size2D};
use wgpu::{
    BindGroup, BindGroupLayout, CommandEncoder, Device, Extent3d, ImageCopyBuffer,
    ImageCopyTexture, Sampler, Texture, TextureAspect, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
};

use crate::unit::PixelUnit;

use super::{create_bind_group, rect_to_origin_extent};

#[derive(Debug)]
pub struct Framebuffer {
    texture: Texture,
    view: TextureView,
    format: TextureFormat,
    bind_group: BindGroup,
    size: Size2D<u32, PixelUnit>,
}

impl Framebuffer {
    pub fn init(
        device: &Device,
        layout: &BindGroupLayout,
        format: TextureFormat,
        size: Size2D<u32, PixelUnit>,
        sampler: &Sampler,
    ) -> Self {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("Framebuffer texture"),
            size: Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::COPY_SRC
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::RENDER_ATTACHMENT,
        });

        let view = texture.create_view(&TextureViewDescriptor::default());

        let bind_group = create_bind_group(device, layout, &view, sampler);

        Self {
            texture,
            view,
            format,
            size,
            bind_group,
        }
    }

    pub fn format(&self) -> TextureFormat {
        self.format
    }

    pub fn size(&self) -> Size2D<u32, PixelUnit> {
        self.size
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    pub fn view(&self) -> &TextureView {
        &self.view
    }

    pub fn copy_to_buffer(
        &self,
        encoder: &mut CommandEncoder,
        buffer: ImageCopyBuffer,
        rect: &Rect<u32, PixelUnit>,
    ) {
        let (origin, extent) = rect_to_origin_extent(rect);

        encoder.copy_texture_to_buffer(
            ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin,
                aspect: TextureAspect::All,
            },
            buffer,
            extent,
        )
    }

    pub fn copy_to_texture(
        &self,
        encoder: &mut CommandEncoder,
        texture: ImageCopyTexture,
        rect: &Rect<u32, PixelUnit>,
    ) {
        let (origin, extent) = rect_to_origin_extent(rect);

        encoder.copy_texture_to_texture(
            ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin,
                aspect: TextureAspect::All,
            },
            texture,
            extent,
        )
    }
}
