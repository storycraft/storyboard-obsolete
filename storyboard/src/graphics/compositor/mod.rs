/*
 * Created on Thu May 05 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

pub mod primitive;

use storyboard_core::{
    euclid::Size2D,
    graphics::texture::SizedTexture2D,
    wgpu::{
        util::RenderEncoder, BlendState, ColorTargetState, ColorWrites, TextureFormat,
        TextureUsages, TextureViewDescriptor,
    },
};

use self::primitive::{PreparedPrimitive, Primitive, PrimitiveCompositor};

use super::{
    backend::StoryboardBackend,
    context::{DrawContext, RenderContext},
    texture::TextureData,
};

/// Component compositor
///
/// Each compositor contains gpu resources and draw, render instructions need for component rendering.
pub trait ComponentCompositor {
    type Component: Send + Sync;
    type Prepared: Send + Sync;

    /// Prepare and write gpu side datas for component rendering
    fn draw(&self, ctx: &mut DrawContext, component: &Self::Component, depth: f32) -> Self::Prepared;

    /// Render component using prepared data
    fn render<'rpass>(
        &'rpass self,
        ctx: &RenderContext<'rpass>,
        pass: &mut impl RenderEncoder<'rpass>,
        prepared: &'rpass Self::Prepared,
    );
}

#[derive(Debug)]
pub struct StoryboardCompositor {
    primitive: PrimitiveCompositor,
}

impl StoryboardCompositor {
    pub fn init(backend: &StoryboardBackend, texture_data: &TextureData) -> Self {
        let empty_texture = {
            let sized = SizedTexture2D::init(
                backend.device(),
                Some("StoryboardCompositor empty texture"),
                Size2D::new(1, 1),
                TextureFormat::Bgra8Unorm,
                TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            );

            sized.write(backend.queue(), None, &[0xff, 0xff, 0xff, 0xff]);

            sized
        };

        let fragment_targets = &[ColorTargetState {
            format: texture_data.framebuffer_texture_format(),
            blend: Some(BlendState::ALPHA_BLENDING),
            write_mask: ColorWrites::ALL,
        }];

        let primitive = PrimitiveCompositor::init(
            backend.device(),
            texture_data.bind_group_layout(),
            fragment_targets,
            None,
            texture_data.init_render_texture(
                backend.device(),
                empty_texture
                    .create_view(&TextureViewDescriptor::default())
                    .into(),
                None,
            ),
        );

        Self { primitive }
    }
}

impl ComponentCompositor for StoryboardCompositor {
    type Component = Component;
    type Prepared = PreparedComponent;

    fn draw(&self, ctx: &mut DrawContext, component: &Self::Component, depth: f32) -> Self::Prepared {
        match component {
            Component::Primitive(primitive) => {
                PreparedComponent::PreparedPrimitive(self.primitive.draw(ctx, primitive, depth))
            }
        }
    }

    fn render<'rpass>(
        &'rpass self,
        ctx: &RenderContext<'rpass>,
        pass: &mut impl RenderEncoder<'rpass>,
        prepared: &'rpass Self::Prepared,
    ) {
        match prepared {
            PreparedComponent::PreparedPrimitive(primitive) => {
                self.primitive.render(ctx, pass, primitive)
            }
        }
    }
}

#[derive(Debug)]
pub enum Component {
    Primitive(Primitive),
}

impl From<Primitive> for Component {
    fn from(primitive: Primitive) -> Self {
        Self::Primitive(primitive)
    }
}

#[derive(Debug)]
pub enum PreparedComponent {
    PreparedPrimitive(PreparedPrimitive),
}

impl From<PreparedPrimitive> for PreparedComponent {
    fn from(primitive: PreparedPrimitive) -> Self {
        Self::PreparedPrimitive(primitive)
    }
}

