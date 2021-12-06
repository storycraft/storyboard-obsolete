/*
 * Created on Sat Nov 13 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

pub mod depth;

use std::{num::NonZeroU32, sync::Arc};

use euclid::{Point2D, Rect, Size2D};
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Device,
    Extent3d, ImageCopyTexture, ImageDataLayout, Origin3d, Queue, Sampler, SamplerDescriptor,
    ShaderStages, Texture, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat,
    TextureSampleType, TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension,
};

use crate::component::layout::texture::QuadTextureCoord;

use super::{PixelUnit, TextureUnit};

#[derive(Debug)]
pub struct TextureData {
    device: Arc<Device>,
    queue: Arc<Queue>,

    bind_group_layout: BindGroupLayout,
    sampler: Sampler,

    framebuffer_texture_format: TextureFormat,
}

impl TextureData {
    pub fn init(
        device: Arc<Device>,
        queue: Arc<Queue>,
        framebuffer_texture_format: TextureFormat,
    ) -> Self {
        let bind_group_layout = create_texture2d_bind_group_layout(&device);

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Texture2D default sampler"),
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,

            ..Default::default()
        });

        Self {
            device,
            queue,

            bind_group_layout,
            sampler,

            framebuffer_texture_format,
        }
    }

    pub const fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layout
    }

    pub const fn default_sampler(&self) -> &Sampler {
        &self.sampler
    }

    pub const fn framebuffer_texture_format(&self) -> TextureFormat {
        self.framebuffer_texture_format
    }

    pub fn create_texture(
        &self,
        format: TextureFormat,
        size: Size2D<u32, PixelUnit>,
        sampler: Option<&Sampler>,
    ) -> Texture2D {
        Texture2D::init(
            &self.device,
            &self.bind_group_layout,
            format,
            size,
            sampler.unwrap_or(&self.sampler),
        )
    }

    pub fn write_texture(
        &self,
        texture: &Texture2D,
        rect: Option<&Rect<u32, PixelUnit>>,
        data: &[u8],
    ) {
        texture.write(&self.queue, rect, data);
    }

    pub fn create_texture_data(
        &self,
        format: TextureFormat,
        size: Size2D<u32, PixelUnit>,
        sampler: Option<&Sampler>,
        data: &[u8],
    ) -> Texture2D {
        let texture = self.create_texture(format, size, sampler);

        texture.write(&self.queue, None, data);

        texture
    }
}

#[derive(Debug)]
pub struct Texture2D {
    texture: Texture,
    format: TextureFormat,
    bind_group: BindGroup,
    size: Size2D<u32, PixelUnit>,
}

impl Texture2D {
    pub fn init(
        device: &Device,
        layout: &BindGroupLayout,
        format: TextureFormat,
        size: Size2D<u32, PixelUnit>,
        sampler: &Sampler,
    ) -> Self {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("Texture2D texture"),
            size: Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::COPY_DST
                | TextureUsages::COPY_SRC
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::RENDER_ATTACHMENT,
        });

        let bind_group = create_texture_bind_group(
            device,
            layout,
            &texture.create_view(&TextureViewDescriptor::default()),
            sampler,
        );

        Self {
            texture,
            format,
            size,
            bind_group,
        }
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
                    extent.width * self.format.describe().block_size as u32,
                ),
                rows_per_image: None,
            },
            extent,
        );
    }

    pub fn create_view(&self) -> TextureView {
        self.texture.create_view(&TextureViewDescriptor::default())
    }

    pub fn to_tex_unit(&self, point: Point2D<u32, PixelUnit>) -> Point2D<f32, TextureUnit> {
        Point2D::new(point.x as f32 / self.size.width as f32, point.y as f32 / self.size.height as f32)
    }

    pub fn to_tex_rect(&self, rect: Rect<u32, PixelUnit>) -> Rect<f32, TextureUnit> {
        rect.cast().cast_unit().scale(1.0 / self.size.width as f32, 1.0 / self.size.height as f32)
    }

    pub fn to_tex_coords(&self, rect: Rect<u32, PixelUnit>) -> QuadTextureCoord {
        let rect = self.to_tex_rect(rect);

        let (start, end) = (rect.origin, (rect.origin + rect.size));

        [
            start,
            Point2D::new(start.x, end.y),
            end,
            Point2D::new(end.x, start.y)
        ]
    }

    pub const fn as_image_copy(&self, origin: Origin3d) -> ImageCopyTexture {
        ImageCopyTexture {
            texture: &self.texture,
            mip_level: 0,
            origin,
            aspect: TextureAspect::All,
        }
    }

    pub const fn size(&self) -> &Size2D<u32, PixelUnit> {
        &self.size
    }

    pub const fn format(&self) -> TextureFormat {
        self.format
    }

    pub const fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}

pub const TEXTURE_2D_BIND_GROUP_LAYOUT_DESCRIPTOR: BindGroupLayoutDescriptor =
    BindGroupLayoutDescriptor {
        label: Some("Texture2D bind group layout"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    multisampled: false,
                    view_dimension: TextureViewDimension::D2,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler {
                    filtering: true,
                    comparison: false,
                },
                count: None,
            },
        ],
    };

#[inline]
pub fn create_texture2d_bind_group_layout(device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&TEXTURE_2D_BIND_GROUP_LAYOUT_DESCRIPTOR)
}

pub fn create_texture_bind_group(
    device: &Device,
    layout: &BindGroupLayout,
    texture_view: &TextureView,
    sampler: &Sampler,
) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor {
        label: Some("Texture2D bind group"),
        layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(texture_view),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Sampler(sampler),
            },
        ],
    })
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
