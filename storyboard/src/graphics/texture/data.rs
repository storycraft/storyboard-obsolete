/*
 * Created on Sun Jun 12 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use storyboard_core::{
    graphics::renderer::context::BackendContext,
    store::{Store, StoreResources},
    wgpu::{AddressMode, BindGroupLayout, Device, Sampler, SamplerDescriptor},
};

use super::create_texture2d_bind_group_layout;

/// Common texture datas.
#[derive(Debug)]
pub struct TextureData {
    bind_group_layout: BindGroupLayout,
    sampler: Sampler,
}

impl TextureData {
    pub fn init(device: &Device) -> Self {
        let bind_group_layout = create_texture2d_bind_group_layout(device);

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Texture2D default sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,

            ..Default::default()
        });

        Self {
            bind_group_layout,
            sampler,
        }
    }

    pub const fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layout
    }

    pub const fn default_sampler(&self) -> &Sampler {
        &self.sampler
    }
}

impl StoreResources<BackendContext<'_>> for TextureData {
    fn initialize(_: &Store, ctx: &BackendContext) -> Self {
        Self::init(ctx.device)
    }
}
