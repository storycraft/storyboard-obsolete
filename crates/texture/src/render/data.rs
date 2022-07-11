use storyboard_core::{
    euclid::Size2D,
    store::{Store, StoreResources},
};

use storyboard_render::{
    texture::{SizedTexture2D, TextureView2D},
    wgpu::{
        AddressMode, BindGroupLayout, Device, Sampler, SamplerDescriptor, TextureFormat,
        TextureUsages, FilterMode,
    }, shared::BackendScopeContext,
};

use super::{create_texture2d_bind_group_layout, RenderTexture2D};

/// Common texture datas.
#[derive(Debug)]
pub struct TextureData {
    bind_group_layout: BindGroupLayout,
    nearest_sampler: Sampler,
    linear_sampler: Sampler,
}

impl TextureData {
    pub fn init(device: &Device) -> Self {
        let bind_group_layout = create_texture2d_bind_group_layout(device);

        let nearest_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Texture2D nearest sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,

            ..Default::default()
        });

        let linear_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Texture2D linear sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,

            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,

            ..Default::default()
        });

        Self {
            bind_group_layout,
            nearest_sampler,
            linear_sampler,
        }
    }

    pub const fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layout
    }

    pub const fn nearest_sampler(&self) -> &Sampler {
        &self.nearest_sampler
    }

    pub const fn linear_sampler(&self) -> &Sampler {
        &self.linear_sampler
    }

    pub fn create_render_texture(
        &self,
        device: &Device,
        view: TextureView2D,
        sampler: Option<&Sampler>,
    ) -> RenderTexture2D {
        RenderTexture2D::init(
            device,
            view,
            &self.bind_group_layout,
            sampler.unwrap_or(&self.nearest_sampler),
        )
    }
}

impl StoreResources<BackendScopeContext<'_>> for TextureData {
    fn initialize(_: &Store, ctx: &BackendScopeContext) -> Self {
        Self::init(ctx.device)
    }
}

#[derive(Debug)]
/// Resources containing white empty texture
pub struct EmptyTextureResources {
    pub empty_texture: RenderTexture2D,
}

impl StoreResources<BackendScopeContext<'_>> for EmptyTextureResources {
    fn initialize(store: &Store, ctx: &BackendScopeContext) -> Self {
        let textures = store.get::<TextureData, _>(ctx);

        let empty_texture = {
            let sized = SizedTexture2D::init(
                ctx.device,
                Some("EmptyTextureResources empty texture"),
                Size2D::new(1, 1),
                TextureFormat::Bgra8Unorm,
                TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            );

            sized.write(ctx.queue, None, &[0xff, 0xff, 0xff, 0xff]);

            RenderTexture2D::init(
                ctx.device,
                TextureView2D::from(sized.create_view_default(None)),
                textures.bind_group_layout(),
                textures.nearest_sampler(),
            )
        };

        Self { empty_texture }
    }
}
