/*
 * Created on Sun Jun 12 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

pub mod data;

use storyboard_core::{
    graphics::texture::TextureView2D,
    wgpu::{
        util::RenderEncoder, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
        BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Device,
        Sampler, SamplerBindingType, ShaderStages, TextureSampleType, TextureView,
        TextureViewDimension,
    },
};

#[derive(Debug)]
pub struct RenderTexture2D {
    view: TextureView2D,
    bind_group: BindGroup,
}

impl RenderTexture2D {
    pub fn init(
        device: &Device,
        view: TextureView2D,
        layout: &BindGroupLayout,
        sampler: &Sampler,
    ) -> Self {
        let bind_group = create_texture_bind_group(device, layout, view.inner().inner(), sampler);

        RenderTexture2D::new_from_bind_group(view, bind_group)
    }

    pub const fn new_from_bind_group(view: TextureView2D, bind_group: BindGroup) -> Self {
        Self { view, bind_group }
    }

    pub const fn view(&self) -> &TextureView2D {
        &self.view
    }

    pub fn bind<'a>(&'a self, index: u32, pass: &mut (impl RenderEncoder<'a> + ?Sized)) {
        pass.set_bind_group(index, &self.bind_group, &[])
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
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
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
