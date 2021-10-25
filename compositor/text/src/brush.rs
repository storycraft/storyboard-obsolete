/*
 * Created on Tue Oct 19 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{collections::HashMap, sync::Arc};

use font_kit::{
    canvas::{Canvas, Format, RasterizationOptions},
    hinting::HintingOptions,
};
use pathfinder_geometry::transform2d::Transform2F;
use rect_packer::DensePacker;
use storyboard::{
    math::Rect,
    texture::{resources::TextureResources, Texture2D},
    unit::{PixelUnit, WgpuUnit},
    wgpu::{Queue, TextureFormat},
};

use crate::font::DrawFont;

pub struct GlyphBrush {
    draw_font: Arc<DrawFont>,

    size: f32,

    packer: DensePacker,
    mapping: HashMap<char, GlyphTexInfo>,

    texture: Arc<Texture2D>,
}

impl GlyphBrush {
    pub const ATLAS_SIZE: u32 = 2048;

    pub fn init(textures: &TextureResources, draw_font: Arc<DrawFont>, size: f32) -> Self {
        Self {
            draw_font,

            size,
            packer: DensePacker::new(Self::ATLAS_SIZE as i32, Self::ATLAS_SIZE as i32),
            mapping: HashMap::with_capacity(32),

            texture: Arc::new(textures.create_texture(
                TextureFormat::R8Unorm,
                (Self::ATLAS_SIZE, Self::ATLAS_SIZE).into(),
                None,
            )),
        }
    }

    pub fn draw_font(&self) -> &Arc<DrawFont> {
        &self.draw_font
    }

    pub fn size(&self) -> f32 {
        self.size
    }

    pub fn get_glyph_tex_info(&mut self, queue: &Queue, ch: char) -> Option<GlyphTexInfo> {
        if let Some(info) = self.mapping.get(&ch) {
            Some(*info)
        } else {
            let id = self.draw_font.font().glyph_for_char(ch)?;
            let hinting = HintingOptions::Full(self.size);
            let rasterization_options = RasterizationOptions::GrayscaleAa;

            let raster_rect = self
                .draw_font
                .font()
                .raster_bounds(
                    id,
                    self.size,
                    Transform2F::default(),
                    hinting,
                    rasterization_options,
                )
                .ok()?;

            let mut canvas = Canvas::new(raster_rect.size(), Format::A8);

            let texture_rect = self
                .draw_font
                .font()
                .rasterize_glyph(
                    &mut canvas,
                    id,
                    self.size,
                    Transform2F::from_translation(-raster_rect.origin().to_f32()),
                    hinting,
                    rasterization_options,
                )
                .ok()
                .and_then(|_| {
                    let pixel_rect = {
                        let packer_rect =
                            self.packer.pack(canvas.size.x(), canvas.size.y(), false)?;

                        Rect {
                            origin: (packer_rect.x, packer_rect.y).into(),
                            size: (packer_rect.width, packer_rect.height).into(),
                        }
                        .cast()
                    };

                    self.texture.write(queue, &pixel_rect, &canvas.pixels);

                    let gpu_rect =
                        (pixel_rect.cast::<f32>() / GlyphBrush::ATLAS_SIZE as f32).cast_unit();
                    Some(gpu_rect)
                })?;

            let info = GlyphTexInfo {
                raster_rect: Rect {
                    origin: (raster_rect.origin_x(), raster_rect.origin_y()).into(),
                    size: (raster_rect.width(), raster_rect.height()).into(),
                },
                texture_rect,
            };

            self.mapping.insert(ch, info);

            Some(info)
        }
    }

    pub fn texture(&self) -> &Arc<Texture2D> {
        &self.texture
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GlyphTexInfo {
    pub raster_rect: Rect<i32, PixelUnit>,
    pub texture_rect: Rect<f32, WgpuUnit>,
}
