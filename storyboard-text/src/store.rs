/*
 * Created on Fri Nov 26 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{fmt::Debug, sync::Arc};

use font_kit::canvas::Format;
use storyboard::{
    graphics::{
        texture::{Texture2D, TextureData},
        wgpu::TextureFormat,
        PixelUnit,
    },
    math::{Rect, Size2D},
    observable::Observable,
};

use crate::{
    font::DrawFont,
    mapping::{GlyphKey, GlyphMappingData},
};

/// Store glyph texture atlas.
#[derive(Debug)]
pub struct GlyphStore {
    texture_data: Arc<TextureData>,
    textures: Vec<Arc<Texture2D>>,
    mappings: Vec<GlyphStoreMappingData>,
}

impl GlyphStore {
    /// Maximum glyph height to store in atlas
    pub const MAX_CACHE_HEIGHT: u32 = 512;
    /// Atlas size
    pub const ATLAS_SIZE: u32 = 1024;
    /// Maximum glyph size
    pub const MAX_GLYPH_HEIGHT: u32 = 1024;

    pub fn new(texture_data: Arc<TextureData>) -> Self {
        Self {
            texture_data,
            textures: Vec::new(),
            mappings: Vec::new(),
        }
    }

    pub fn textures(&self) -> &Vec<Arc<Texture2D>> {
        &self.textures
    }

    pub fn get_glyph(
        &mut self,
        font: &DrawFont,
        key: GlyphKey,
    ) -> Option<StoreGlyphTexInfo> {
        if key.size > Self::MAX_GLYPH_HEIGHT {
            return None;
        }

        for (index, entry) in self.mappings.iter_mut().enumerate() {
            match entry.mapping.inner_ref().get(font, &key) {
                Some(rect) => {
                    entry.used |= true;
                    return Some(StoreGlyphTexInfo { index, rect });
                }

                None => {
                    if let Some(rect) = entry.mapping.inner_mut().cache_rasterized(font, key) {
                        entry.used |= true;
                        return Some(StoreGlyphTexInfo { index, rect });
                    }
                }
            }
        }

        let mut glyph_map = if key.size <= Self::MAX_CACHE_HEIGHT {
            GlyphMappingData::new_atlas(
                Format::A8,
                Size2D::new(Self::MAX_CACHE_HEIGHT, Self::MAX_CACHE_HEIGHT),
            )
        } else {
            GlyphMappingData::new_single(Format::A8, Size2D::new(key.size, key.size))
        };

        let rect = glyph_map.cache_rasterized(font, key)?;

        self.textures.push(Arc::new(
            self.texture_data.create_texture(TextureFormat::R8Unorm, glyph_map.get_size(), None),
        ));

        self.mappings.push(GlyphStoreMappingData {
            used: true,
            mapping: Observable::new(glyph_map),
        });

        Some(StoreGlyphTexInfo {
            index: self.mappings.len() - 1,
            rect,
        })
    }

    pub fn prepare(&mut self) {
        for (i, entry) in self.mappings.iter_mut().enumerate() {
            if entry.mapping.unmark() {
                let mapping = entry.mapping.inner_ref();

                if let Some(texture) = self.textures.get_mut(i) {
                    self.texture_data.write_texture(&texture, None, &mapping.canvas().pixels);
                }
            }
        }
    }

    pub fn finish(&mut self) {
        let mut i: usize = 0;
        self.textures.retain(|_| {
            let res = self.mappings[i].used;
            i += 1;

            res
        });
        self.mappings.retain(|entry| entry.used);

        self.mappings.iter_mut().for_each(|entry| {
            entry.used = false;
        });
    }

    pub fn clear(&mut self) {
        self.mappings.clear();
    }
}

#[derive(Debug)]
pub struct GlyphStoreMappingData {
    used: bool,
    mapping: Observable<GlyphMappingData>,
}

#[derive(Debug, Clone, Copy)]
pub struct StoreGlyphTexInfo {
    pub index: usize,
    pub rect: Rect<u32, PixelUnit>,
}
