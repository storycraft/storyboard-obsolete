/*
 * Created on Fri Oct 15 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use storyboard::color::LinSrgba;
use storyboard::math::SideOffsets2D;
use storyboard::pipeline::PipelineTargetDescriptor;
use storyboard::wgpu::{BindGroupLayout, Device, RenderPipeline};

use storyboard::{
    buffer::index::IndexBuffer,
    component::{texture::TextureLayout, DrawBox, Drawable},
};

use crate::{init_box_pipeline, init_box_shader, BoxDrawState, BoxInstance, BoxStyle, BoxVertex};

#[derive(Debug)]
pub struct BoxCompositor {
    pipeline: RenderPipeline,
    quad_index_buffer: IndexBuffer,
}

impl BoxCompositor {
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
        let shader = init_box_shader(device, texture_bind_group_layout);
        let pipeline = init_box_pipeline(device, &shader, pipeline_desc);

        let quad_index_buffer =
            IndexBuffer::init(device, Some("Box Quad index buffer"), &[0, 1, 2, 3, 0, 2]);

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

    pub fn box_2d(&self, style: &BoxStyle, draw_box: &DrawBox) -> Drawable<BoxDrawState> {
        let expanded_rect = {
            if style.border_thickness != 0.0 {
                draw_box
                    .rect
                    .outer_rect(SideOffsets2D::new_all_same(style.border_thickness))
            } else {
                draw_box.rect
            }
        };

        let quad = draw_box.get_quad_2d(&expanded_rect);

        let texture_coords = style
            .texture
            .as_ref()
            .map_or(TextureLayout::STRETCHED, |item| {
                let mut space = draw_box.inner_space();
                if style.border_thickness != 0.0 {
                    space.parent.size.width += style.border_thickness * 2.0;
                    space.parent.size.height += style.border_thickness * 2.0;
                }

                item.layout.texture_coord_quad(&space, &item.texture.size())
            });

        let quad = [
            BoxVertex {
                position: quad[0].to_3d().to_array(),
                fill_color: self
                    .mix_draw_color(style.opacity, &style.fill_color[0])
                    .into_encoding(),
                border_color: self
                    .mix_draw_color(style.opacity, &style.border_color[0])
                    .into_encoding(),
                tex_coord: texture_coords[0].to_array(),
                rect_coord: [-style.border_thickness, -style.border_thickness],
            },
            BoxVertex {
                position: quad[1].to_3d().to_array(),
                fill_color: self
                    .mix_draw_color(style.opacity, &style.fill_color[1])
                    .into_encoding(),
                border_color: self
                    .mix_draw_color(style.opacity, &style.border_color[1])
                    .into_encoding(),
                tex_coord: texture_coords[1].to_array(),
                rect_coord: [
                    -style.border_thickness,
                    draw_box.rect.size.height + style.border_thickness,
                ],
            },
            BoxVertex {
                position: quad[2].to_3d().to_array(),
                fill_color: self
                    .mix_draw_color(style.opacity, &style.fill_color[2])
                    .into_encoding(),
                border_color: self
                    .mix_draw_color(style.opacity, &style.border_color[2])
                    .into_encoding(),
                tex_coord: texture_coords[2].to_array(),
                rect_coord: [
                    draw_box.rect.size.width + style.border_thickness,
                    draw_box.rect.size.height + style.border_thickness,
                ],
            },
            BoxVertex {
                position: quad[3].to_3d().to_array(),
                fill_color: self
                    .mix_draw_color(style.opacity, &style.fill_color[3])
                    .into_encoding(),
                border_color: self
                    .mix_draw_color(style.opacity, &style.border_color[3])
                    .into_encoding(),
                tex_coord: texture_coords[3].to_array(),
                rect_coord: [
                    draw_box.rect.size.width + style.border_thickness,
                    -style.border_thickness,
                ],
            },
        ];

        let instance = BoxInstance {
            rect: [draw_box.rect.size.width, draw_box.rect.size.height],
            border_radius: style.border_radius,
            border_thickness: style.border_thickness,
        };

        let opaque = !style.fill_color.partial_transparent()
            && style.texture.is_none()
            && style.opacity == 1.0
            && style.border_radius == 0.0
            && style.border_thickness == 0.0;

        Drawable {
            opaque,
            state: BoxDrawState {
                pipeline: &self.pipeline,
                quad_index_buffer: &self.quad_index_buffer,
                texture: style.texture.as_ref().map(|item| item.texture.clone()),
                quad,
                instance,
            },
        }
    }
}
