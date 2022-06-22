use storyboard_core::{
    euclid::Size2D,
    store::{Store, StoreResources},
};

use storyboard_render::{
    renderer::context::BackendContext,
    texture::{SizedTexture2D, TextureView2D},
    wgpu::{
        AddressMode, BindGroupLayout, Device, Sampler, SamplerDescriptor, TextureFormat,
        TextureUsages,
    },
};

use super::{create_texture2d_bind_group_layout, RenderTexture2D};

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
            sampler.unwrap_or(&self.sampler),
        )
    }
}

impl StoreResources<BackendContext<'_>> for TextureData {
    fn initialize(_: &Store, ctx: &BackendContext) -> Self {
        Self::init(ctx.device)
    }
}

#[derive(Debug)]
/// Resources containing white empty texture
pub struct EmptyTextureResources {
    pub empty_texture: RenderTexture2D,
}

impl StoreResources<BackendContext<'_>> for EmptyTextureResources {
    fn initialize(store: &Store, ctx: &BackendContext) -> Self {
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
                textures.default_sampler(),
            )
        };

        Self { empty_texture }
    }
}
