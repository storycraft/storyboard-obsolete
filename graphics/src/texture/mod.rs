/*
 * Created on Tue Sep 14 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

pub mod depth;
pub mod framebuffer;
pub mod resources;

use std::num::NonZeroU32;

use euclid::{Point2D, Rect, Size2D};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Device, Extent3d, ImageCopyTexture,
    ImageDataLayout, Origin3d, Queue, Sampler, ShaderStages, Texture, TextureAspect,
    TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages,
    TextureView, TextureViewDescriptor, TextureViewDimension,
};

use crate::unit::PixelUnit;

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
            usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
        });

        let bind_group = create_bind_group(
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

    pub fn init_data(
        device: &Device,
        queue: &Queue,
        layout: &BindGroupLayout,
        format: TextureFormat,
        size: Size2D<u32, PixelUnit>,
        sampler: &Sampler,
        data: &[u8],
    ) -> Self {
        let texture = Self::init(device, layout, format, size, sampler);

        texture.write(
            queue,
            &Rect {
                origin: Point2D::zero(),
                size,
            },
            data,
        );

        texture
    }

    pub fn write(&self, queue: &Queue, rect: &Rect<u32, PixelUnit>, data: &[u8]) {
        let (origin, extent) = rect_to_origin_extent(&rect);

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
                bytes_per_row: NonZeroU32::new(rect.size.width * self.format.describe().block_size as u32),
                rows_per_image: None,
            },
            extent,
        );
    }

    pub fn as_image_copy(&self, origin: Origin3d) -> ImageCopyTexture {
        ImageCopyTexture {
            texture: &self.texture,
            mip_level: 0,
            origin,
            aspect: TextureAspect::All,
        }
    }

    pub fn set_sampler(&mut self, device: &Device, layout: &BindGroupLayout, sampler: &Sampler) {
        let bind_group = create_bind_group(
            device,
            layout,
            &self.texture.create_view(&TextureViewDescriptor::default()),
            sampler,
        );

        self.bind_group = bind_group;
    }

    pub fn size(&self) -> &Size2D<u32, PixelUnit> {
        &self.size
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}

fn create_bind_group(
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
