/*
 * Created on Tue Oct 19 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::sync::Arc;

use font_kit::{
    canvas::{Canvas, Format, RasterizationOptions},
    hinting::HintingOptions,
};
use pathfinder_geometry::transform2d::Transform2F;
use rect_packer::DensePacker;
use rustc_hash::FxHashMap;
use storyboard_graphics::{
    math::Rect,
    texture::{resources::TextureResources, Texture2D},
    unit::{PixelUnit, WgpuUnit},
    wgpu::{Queue, TextureFormat},
};

use crate::font::DrawFont;

pub struct GlyphStore {
    draw_font: Arc<DrawFont>,

    size: f32,

    packer: DensePacker,
    mapping: FxHashMap<u32, GlyphTexInfo>,

    texture: Arc<Texture2D>,
}

impl GlyphStore {
    pub const ATLAS_SIZE: u32 = 2048;
    pub const TEXTURE_COUNT: usize = 4;

    pub fn init(textures: &TextureResources, draw_font: Arc<DrawFont>, size: f32) -> Self {
        Self {
            draw_font,

            size,
            packer: DensePacker::new(Self::ATLAS_SIZE as i32, Self::ATLAS_SIZE as i32),
            mapping: FxHashMap::default(),

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

    fn raster_glyph(&mut self, id: u32) -> Option<GlyphRasterData> {
        let hinting = HintingOptions::Vertical(self.size);
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

        let mut bitmap = Canvas::new(raster_rect.size(), Format::A8);

        self.draw_font
            .font()
            .rasterize_glyph(
                &mut bitmap,
                id,
                self.size,
                Transform2F::from_translation(-raster_rect.origin().to_f32()),
                hinting,
                rasterization_options,
            )
            .ok()?;

        Some(GlyphRasterData {
            raster_rect: Rect {
                origin: (raster_rect.origin_x(), raster_rect.origin_y()).into(),
                size: (raster_rect.width(), raster_rect.height()).into(),
            },
            bitmap,
        })
    }

    pub fn get_glyph_tex_info(&mut self, queue: &Queue, glyph_id: u32) -> Option<GlyphTexInfo> {
        if let Some(info) = self.mapping.get(&glyph_id) {
            Some(*info)
        } else {
            let glyph_data = self.raster_glyph(glyph_id)?;

            let pixel_rect = {
                let packer_rect = self.packer.pack(
                    glyph_data.bitmap.size.x(),
                    glyph_data.bitmap.size.y(),
                    false,
                )?;

                Rect {
                    origin: (packer_rect.x, packer_rect.y).into(),
                    size: (packer_rect.width, packer_rect.height).into(),
                }
                .cast()
            };

            self.texture
                .write(queue, &pixel_rect, &glyph_data.bitmap.pixels);

            let texture_rect =
                (pixel_rect.cast::<f32>() / GlyphStore::ATLAS_SIZE as f32).cast_unit();

            let info = GlyphTexInfo {
                raster_rect: glyph_data.raster_rect,
                texture_rect,
            };

            self.mapping.insert(glyph_id, info);

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

#[derive(Debug)]
pub struct GlyphRasterData {
    pub raster_rect: Rect<i32, PixelUnit>,
    pub bitmap: Canvas,
}
