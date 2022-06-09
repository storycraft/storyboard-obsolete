/*
 * Created on Wed Jun 08 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{collections::HashMap, fmt::Debug, hash::Hash};

use fontdue::{Font, Metrics};
use rect_packer::DensePacker;
use ringbuffer::{ConstGenericRingBuffer, RingBufferExt, RingBufferWrite};
use storyboard_core::{
    euclid::{Point2D, Rect, Size2D},
    graphics::texture::{view::TextureView2D, SizedTexture2D},
    unit::PixelUnit,
    wgpu::{Device, Queue, TextureFormat, TextureUsages},
};

pub struct GlyphCache {
    pages: ConstGenericRingBuffer<GlyphAtlasMap, 8>,
}

impl GlyphCache {
    pub const PAGE_SIZE_LIMIT: u32 = 256;

    pub fn new() -> Self {
        Self {
            pages: ConstGenericRingBuffer::new(),
        }
    }

    pub fn get_view(
        &mut self,
        device: &Device,
        queue: &Queue,
        font: &Font,
        ch: char,
        size_px: u32,
    ) -> (TextureView2D, Metrics) {
        if size_px > Self::PAGE_SIZE_LIMIT {
            unimplemented!()
        }

        let key = GlyphKey::new(font.file_hash(), ch, size_px);

        {
            let mut ring_iter = self.pages.iter();
            while let Some(atlas_map) = ring_iter.next() {
                if let Some(item) = atlas_map.get_view(&key) {
                    return item;
                }
            }
        }

        let mut ring_iter = self.pages.iter_mut();
        while let Some(atlas_map) = ring_iter.next() {
            if let Some(item) = atlas_map.pack(queue, font, ch, size_px) {
                return item;
            }
        }

        let atlas = GlyphAtlasMap::init(device, Size2D::new(1024, 1024), TextureFormat::R8Unorm);
        self.pages.push(atlas);
        self.pages.back_mut().unwrap().pack(queue, font, ch, size_px).unwrap()
    }
}

impl Debug for GlyphCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlyphCache").finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlyphKey {
    pub font_file_hash: usize,
    pub ch: char,
    pub size_px: u32,
}

impl GlyphKey {
    pub const fn new(font_file_hash: usize, ch: char, size_px: u32) -> Self {
        Self {
            font_file_hash,
            ch,
            size_px,
        }
    }
}

pub struct GlyphAtlasMap {
    texture: SizedTexture2D,
    packer: DensePacker,
    map: HashMap<GlyphKey, (Rect<u32, PixelUnit>, Metrics)>,
}

impl GlyphAtlasMap {
    pub fn init(device: &Device, size: Size2D<u32, PixelUnit>, format: TextureFormat) -> Self {
        let texture = SizedTexture2D::init(
            device,
            Some("GlyphAtlasTexture texture"),
            size,
            format,
            TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        );

        let packer = DensePacker::new(size.width as _, size.height as _);

        Self {
            texture,
            packer,
            map: HashMap::new(),
        }
    }

    pub fn get_view(&self, key: &GlyphKey) -> Option<(TextureView2D, Metrics)> {
        let (rect, metrics) = *self.map.get(key)?;
        Some((self.texture.create_view_default(None).slice(rect), metrics))
    }

    pub fn pack(
        &mut self,
        queue: &Queue,
        font: &Font,
        ch: char,
        size_px: u32,
    ) -> Option<(TextureView2D, Metrics)> {
        let (metrics, data) = font.rasterize(ch, size_px as f32);

        let rect: Rect<u32, PixelUnit> = {
            let packer_rect = self
                .packer
                .pack(metrics.width as _, metrics.height as _, false)?;

            Rect::new(
                Point2D::new(packer_rect.x, packer_rect.y),
                Size2D::new(packer_rect.width, packer_rect.height),
            )
            .cast()
        };

        let key = GlyphKey {
            font_file_hash: font.file_hash(),
            ch,
            size_px,
        };

        self.texture.write(queue, Some(&rect), &data);

        self.map.insert(key, (rect, metrics));

        self.get_view(&key)
    }
}

impl Debug for GlyphAtlasMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlyphAtlasTexture").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use fontdue::{Font, FontSettings};
    use storyboard_core::wgpu::{Backends, Features, Instance};

    use crate::graphics::backend::{BackendOptions, StoryboardBackend};

    use super::GlyphCache;

    pub static FONT: &'static [u8] = include_bytes!("./test-assets/NotoSansCJKkr-Regular.otf");

    #[test]
    fn test_cache() {
        let backend = pollster::block_on(StoryboardBackend::init(
            &Instance::new(Backends::all()),
            None,
            Features::empty(),
            &BackendOptions::default(),
        ))
        .unwrap();

        println!("backend: {:?}", backend);

        let font = Font::from_bytes(FONT, FontSettings::default()).unwrap();

        println!("font: {:?}", font);

        let mut cache = GlyphCache::new();

        println!(
            "Glyph 가: {:?}",
            cache.get_view(backend.device(), backend.queue(), &font, '가', 40)
        );
        println!(
            "Glyph a: {:?}",
            cache.get_view(backend.device(), backend.queue(), &font, 'a', 40)
        );
        println!(
            "Glyph b: {:?}",
            cache.get_view(backend.device(), backend.queue(), &font, 'b', 40)
        );
    }
}
