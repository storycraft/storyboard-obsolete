/*
 * Created on Wed May 04 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use storyboard_core::{
    graphics::texture::view::TextureView2D,
    wgpu::{
        util::RenderEncoder, AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry,
        BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource,
        BindingType, Device, Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages,
        TextureFormat, TextureSampleType, TextureView, TextureViewDimension,
    },
};

#[derive(Debug)]
pub struct Texture2D {
    view: TextureView2D,
    bind_group: BindGroup,
}

impl Texture2D {
    pub const fn new_from_bind_group(view: TextureView2D, bind_group: BindGroup) -> Self {
        Self { view, bind_group }
    }

    pub const fn view(&self) -> &TextureView2D {
        &self.view
    }

    pub fn bind<'a>(&'a self, index: u32, pass: &mut impl RenderEncoder<'a>) {
        pass.set_bind_group(index, &self.bind_group, &[])
    }
}

/// Common storyboard app texture datas.
#[derive(Debug)]
pub struct TextureData {
    bind_group_layout: BindGroupLayout,
    sampler: Sampler,

    framebuffer_texture_format: TextureFormat,
}

impl TextureData {
    pub fn init(device: &Device, framebuffer_texture_format: TextureFormat) -> Self {
        let bind_group_layout = create_texture2d_bind_group_layout(device);

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Texture2D default sampler"),
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,

            ..Default::default()
        });

        Self {
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

    /// Construct renderable [Texture2D] from [TextureView2D] and optionally [Sampler]
    pub fn init_render_texture(
        &self,
        device: &Device,
        view: TextureView2D,
        sampler: Option<&Sampler>,
    ) -> Texture2D {
        let bind_group = create_texture_bind_group(
            device,
            &self.bind_group_layout,
            view.inner().inner(),
            sampler.unwrap_or(&self.sampler),
        );

        Texture2D::new_from_bind_group(view, bind_group)
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
