/*
 * Created on Fri Oct 15 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use storyboard::wgpu::RenderPipeline;
use storyboard::{
    buffer::index::IndexBuffer,
    color::{LinSrgba, Mix},
    component::{texture::TextureLayout, DrawBox, Drawable},
    pipeline::PipelineTargetDescriptor,
    wgpu::{BindGroupLayout, Device},
};

use crate::{
    init_primitive_pipeline, init_primitive_shader, rectangle::RectDrawState, triangle::TriangleDrawState,
    PrimitiveStyle, PrimitiveVertex,
};

#[derive(Debug)]
pub struct PrimitiveCompositor {
    pipeline: RenderPipeline,
    quad_index_buffer: IndexBuffer,
}

impl PrimitiveCompositor {
    pub fn new(pipeline: RenderPipeline, quad_index_buffer: IndexBuffer) -> Self {
        Self {
            pipeline,
            quad_index_buffer,
        }
    }

    pub fn init(
        device: &Device,
        texture_bind_group_layout: &BindGroupLayout,
        pipeline_desc: PipelineTargetDescriptor,
    ) -> Self {
        let shader = init_primitive_shader(device, texture_bind_group_layout);
        let pipeline = init_primitive_pipeline(device, &shader, pipeline_desc);

        let quad_index_buffer = IndexBuffer::init(
            device,
            Some("Primitive Quad index buffer"),
            &[0, 1, 2, 3, 0, 2],
        );

        Self {
            pipeline,
            quad_index_buffer,
        }
    }

    fn mix_draw_color(&self, opacity: f32, color: &LinSrgba) -> LinSrgba {
        let mut color = color.clone();

        color.alpha *= opacity;
        color
    }

    pub fn rect(
        &self,
        style: &PrimitiveStyle,
        draw_box: &DrawBox,
    ) -> Drawable<RectDrawState> {
        let quad = draw_box.get_quad_2d(&draw_box.rect);

        let texture_coords = style
            .texture
            .as_ref()
            .map_or(TextureLayout::STRETCHED, |item| {
                item.layout
                    .texture_coord_quad(&draw_box.inner_space(), &item.texture.size())
            });

        let primitive = [
            PrimitiveVertex {
                position: quad[0].to_3d().to_array(),
                color: self
                    .mix_draw_color(style.opacity, &style.fill_color[0])
                    .into_encoding(),
                texure_coord: texture_coords[0].to_array(),
            },
            PrimitiveVertex {
                position: quad[1].to_3d().to_array(),
                color: self
                    .mix_draw_color(style.opacity, &style.fill_color[1])
                    .into_encoding(),
                texure_coord: texture_coords[1].to_array(),
            },
            PrimitiveVertex {
                position: quad[2].to_3d().to_array(),
                color: self
                    .mix_draw_color(style.opacity, &style.fill_color[2])
                    .into_encoding(),
                texure_coord: texture_coords[2].to_array(),
            },
            PrimitiveVertex {
                position: quad[3].to_3d().to_array(),
                color: self
                    .mix_draw_color(style.opacity, &style.fill_color[3])
                    .into_encoding(),
                texure_coord: texture_coords[3].to_array(),
            },
        ];

        Drawable {
            opaque: !style.fill_color.partial_transparent(),
            state: RectDrawState {
                pipeline: &self.pipeline,
                quad_index_buffer: &self.quad_index_buffer,
                texture: style.texture.as_ref().map(|item| item.texture.clone()),
                primitive,
            },
        }
    }

    pub fn triangle(&self, style: &PrimitiveStyle, draw_box: &DrawBox) -> Drawable<TriangleDrawState> {
        let inner_quad = draw_box.get_quad_2d(&draw_box.rect);

        let top_color = {
            let left_top = style.fill_color[0];
            let right_top = style.fill_color[3];

            if left_top != right_top {
                left_top.mix(&right_top, 0.5)
            } else {
                left_top
            }
        };

        let texture_coords = style
            .texture
            .as_ref()
            .map_or(TextureLayout::STRETCHED, |item| {
                item.layout
                    .texture_coord_quad(&draw_box.inner_space(), &item.texture.size())
            });

        let primitive = [
            PrimitiveVertex {
                position: inner_quad[1].to_3d().to_array(),
                color: self
                    .mix_draw_color(style.opacity, &style.fill_color[1])
                    .into_encoding(),
                texure_coord: texture_coords[1].to_array(),
            },
            PrimitiveVertex {
                position: inner_quad[0].lerp(inner_quad[3], 0.5).to_3d().to_array(),
                color: self
                    .mix_draw_color(style.opacity, &top_color)
                    .into_encoding(),
                texure_coord: texture_coords[0].lerp(texture_coords[3], 0.5).to_array(),
            },
            PrimitiveVertex {
                position: inner_quad[2].to_3d().to_array(),
                color: self
                    .mix_draw_color(style.opacity, &style.fill_color[2])
                    .into_encoding(),
                texure_coord: texture_coords[2].to_array(),
            },
        ];

        Drawable {
            opaque: !style.fill_color.partial_transparent() && style.texture.is_none(),
            state: TriangleDrawState {
                pipeline: &self.pipeline,
                texture: style.texture.as_ref().map(|item| item.texture.clone()),
                primitive,
            },
        }
    }
}
