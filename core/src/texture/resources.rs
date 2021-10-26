/*
 * Created on Thu Sep 30 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::sync::Arc;

use euclid::Size2D;
use wgpu::{BindGroup, BindGroupLayout, Device, Queue, Sampler, SamplerDescriptor, TextureFormat};

use crate::{texture::framebuffer::Framebuffer, unit::PixelUnit};

use super::super::texture::{create_texture2d_bind_group_layout, Texture2D};

#[derive(Debug)]
pub struct TextureResources {
    device: Arc<Device>,
    queue: Arc<Queue>,

    empty_texture: Texture2D,
    default_sampler: Sampler,

    texture2d_bind_group_layout: BindGroupLayout,
}

impl TextureResources {
    pub fn init(device: Arc<Device>, queue: Arc<Queue>) -> Self {
        let texture2d_bind_group_layout = create_texture2d_bind_group_layout(&device);

        let default_sampler = init_default_sampler(&device);

        let empty_texture = Texture2D::init_data(
            &device,
            &queue,
            &texture2d_bind_group_layout,
            TextureFormat::Bgra8Unorm,
            (1, 1).into(),
            &default_sampler,
            &[0xff, 0xff, 0xff, 0xff],
        );

        Self {
            device,
            queue,

            empty_texture,
            default_sampler,
            texture2d_bind_group_layout,
        }
    }

    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }

    pub fn queue(&self) -> &Arc<Queue> {
        &self.queue
    }

    pub fn default_sampler(&self) -> &Sampler {
        &self.default_sampler
    }

    pub fn texture2d_bind_group_layout(&self) -> &BindGroupLayout {
        &self.texture2d_bind_group_layout
    }

    pub fn empty_texture_bind_group(&self) -> &BindGroup {
        self.empty_texture.bind_group()
    }

    pub fn create_texture(
        &self,
        format: TextureFormat,
        size: Size2D<u32, PixelUnit>,
        sampler: Option<&Sampler>,
    ) -> Texture2D {
        Texture2D::init(
            &self.device,
            &self.texture2d_bind_group_layout,
            format,
            size,
            sampler.unwrap_or(&self.default_sampler),
        )
    }

    pub fn create_texture_init(
        &self,
        format: TextureFormat,
        size: Size2D<u32, PixelUnit>,
        sampler: Option<&Sampler>,
        data: &[u8],
    ) -> Texture2D {
        Texture2D::init_data(
            &self.device,
            &self.queue,
            &self.texture2d_bind_group_layout,
            format,
            size,
            sampler.unwrap_or(&self.default_sampler),
            data,
        )
    }

    pub fn create_framebuffer(
        &self,
        format: TextureFormat,
        size: Size2D<u32, PixelUnit>,
        sampler: Option<&Sampler>,
    ) -> Framebuffer {
        Framebuffer::init(
            &self.device,
            &self.texture2d_bind_group_layout,
            format,
            size,
            sampler.unwrap_or(&self.default_sampler),
        )
    }

    pub fn texture_bind_group_or_empty<'a>(
        &'a self,
        texture: Option<&'a Texture2D>,
    ) -> &'a BindGroup {
        texture.unwrap_or(&self.empty_texture).bind_group()
    }
}

#[inline]
fn init_default_sampler(device: &Device) -> Sampler {
    device.create_sampler(&SamplerDescriptor {
        label: Some("Context default sampler"),
        ..SamplerDescriptor::default()
    })
}
