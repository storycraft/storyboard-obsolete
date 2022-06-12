/*
 * Created on Wed Jun 08 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{collections::HashMap, fmt::Debug, hash::Hash};

use ringbuffer::{ConstGenericRingBuffer, RingBufferExt, RingBufferWrite};
use storyboard::core::{
    euclid::{Point2D, Rect, Size2D},
    graphics::texture::{packed::PackedTexture, SizedTexture2D, TextureView2D},
    unit::PixelUnit,
    wgpu::{Device, Queue, TextureFormat, TextureUsages},
};
use ttf_parser::{Face, GlyphId};

use super::GlyphOutlineBuilder;

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
        font: &Face,
        index: u16,
        size_px: u32,
    ) -> Option<(Point2D<f32, PixelUnit>, TextureView2D)> {
        if size_px > Self::PAGE_SIZE_LIMIT {
            unimplemented!()
        }

        let key = GlyphKey::new(0, index, size_px);

        {
            let mut ring_iter = self.pages.iter();
            while let Some(atlas_map) = ring_iter.next() {
                if let Some(item) = atlas_map.get_view(&key) {
                    return Some(item);
                }
            }
        }

        let mut ring_iter = self.pages.iter_mut();
        while let Some(atlas_map) = ring_iter.next() {
            if let Some(item) = atlas_map.pack(queue, font, index, size_px) {
                return Some(item);
            }
        }

        let atlas = GlyphAtlasMap::init(device, Size2D::new(1024, 1024), TextureFormat::R8Unorm);
        self.pages.push(atlas);
        self.pages
            .back_mut()
            .unwrap()
            .pack(queue, font, index, size_px)
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
    pub index: u16,
    pub size_px: u32,
}

impl GlyphKey {
    pub const fn new(font_file_hash: usize, index: u16, size_px: u32) -> Self {
        Self {
            font_file_hash,
            index,
            size_px,
        }
    }
}

#[derive(Debug)]
pub struct GlyphAtlasMap {
    texture: PackedTexture,
    map: HashMap<GlyphKey, (Point2D<f32, PixelUnit>, Rect<u32, PixelUnit>)>,
}

impl GlyphAtlasMap {
    pub fn init(device: &Device, size: Size2D<u32, PixelUnit>, format: TextureFormat) -> Self {
        let texture = PackedTexture::new(SizedTexture2D::init(
            device,
            Some("GlyphAtlasTexture texture"),
            size,
            format,
            TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        ));

        Self {
            texture,
            map: HashMap::new(),
        }
    }

    pub fn get_view(&self, key: &GlyphKey) -> Option<(Point2D<f32, PixelUnit>, TextureView2D)> {
        let (offset, rect) = *self.map.get(key)?;
        Some((offset, self.texture.slice(None, rect)))
    }

    pub fn pack(
        &mut self,
        queue: &Queue,
        font: &Face,
        index: u16,
        size_px: u32,
    ) -> Option<(Point2D<f32, PixelUnit>, TextureView2D)> {
        let bounding_box = font.glyph_bounding_box(GlyphId(index))?;

        let mut builder = GlyphOutlineBuilder::new(font, bounding_box, size_px);
        font.outline_glyph(GlyphId(index), &mut builder);

        let rasterizer = builder.into_rasterizer();
        let mut data: Vec<u8> = vec![0; rasterizer.dimensions().0 * rasterizer.dimensions().1];
        rasterizer.for_each_pixel(|i, alpha| data[i] = (alpha * 255.0) as u8);

        let offset = Point2D::<f32, PixelUnit>::new(
            size_px as f32 * bounding_box.x_min as f32 / font.units_per_em() as f32,
            -1.0 * size_px as f32 * bounding_box.y_min as f32 / font.units_per_em() as f32,
        );

        let rect: Rect<u32, PixelUnit> = self.texture.pack(
            queue,
            Size2D::new(
                rasterizer.dimensions().0 as u32,
                rasterizer.dimensions().1 as u32,
            ),
            &data,
        )?;

        let key = GlyphKey {
            font_file_hash: 0,
            index,
            size_px,
        };

        self.map.insert(key, (offset, rect));

        self.get_view(&key)
    }
}

#[cfg(test)]
mod tests {
    use storyboard::core::wgpu::{Backends, Features, Instance};
    use ttf_parser::Face;

    use storyboard::core::graphics::backend::{BackendOptions, StoryboardBackend};

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

        let font = Face::from_slice(FONT, 0).unwrap();

        println!("font: {:?}", font);

        let mut cache = GlyphCache::new();

        println!(
            "Glyph 가: {:?}",
            cache.get_view(
                backend.device(),
                backend.queue(),
                &font,
                font.glyph_index('가').unwrap().0,
                40
            )
        );
        println!(
            "Glyph a: {:?}",
            cache.get_view(
                backend.device(),
                backend.queue(),
                &font,
                font.glyph_index('a').unwrap().0,
                40
            )
        );
        println!(
            "Glyph b: {:?}",
            cache.get_view(
                backend.device(),
                backend.queue(),
                &font,
                font.glyph_index('b').unwrap().0,
                40
            )
        );
    }
}
