/*
 * Created on Fri Jun 03 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

//! Common rendering resources

use storyboard_core::{
    euclid::Size2D,
    graphics::{
        renderer::context::BackendContext,
        texture::{ SizedTexture2D, TextureView2D},
    },
    store::{StoreResources, Store},
    wgpu::{
        util::{BufferInitDescriptor, DeviceExt},
        Buffer, BufferUsages, IndexFormat, TextureFormat, TextureUsages,
    },
};

use crate::graphics::texture::{RenderTexture2D, data::TextureData};

#[derive(Debug)]
/// Resources containing quad index buffer
pub struct QuadIndexBufferResources {
    pub quad_index_buffer: Buffer,
}

impl QuadIndexBufferResources {
    pub const FORMAT: IndexFormat = IndexFormat::Uint16;
}

impl StoreResources<BackendContext<'_>> for QuadIndexBufferResources {
    fn initialize(_: &Store, ctx: &BackendContext) -> Self {
        let quad_index_buffer = ctx.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("QuadIndexBufferResources quad index buffer"),
            contents: bytemuck::cast_slice(&[0_u16, 1, 2, 0, 2, 3]),
            usage: BufferUsages::INDEX,
        });

        Self { quad_index_buffer }
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
