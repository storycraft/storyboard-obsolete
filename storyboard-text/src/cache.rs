/*
 * Created on Wed Jun 08 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{collections::HashMap, fmt::Debug, hash::Hash, iter::Peekable};

use ringbuffer::{ConstGenericRingBuffer, RingBufferExt, RingBufferWrite};
use smallvec::SmallVec;
use storyboard::core::{
    euclid::{Rect, Size2D, Vector2D},
    graphics::texture::{packed::PackedTexture, SizedTexture2D, SizedTextureView2D},
    unit::{PixelUnit, TextureUnit},
    wgpu::{Device, Queue, TextureFormat, TextureUsages},
};
use ttf_parser::GlyphId;

use crate::font::Font;

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

    #[inline]
    pub fn get_batch(
        &mut self,
        device: &Device,
        queue: &Queue,
        font: &Font,
        indices: impl Iterator<Item = u16>,
        size_px: u32,
    ) -> SmallVec<[GlyphBatch; 2]> {
        self.get_batch_inner(device, queue, font, indices.peekable(), size_px)
    }

    fn get_batch_inner(
        &mut self,
        device: &Device,
        queue: &Queue,
        font: &Font,
        mut indices: Peekable<impl Iterator<Item = u16>>,
        size_px: u32,
    ) -> SmallVec<[GlyphBatch; 2]> {
        if size_px > Self::PAGE_SIZE_LIMIT {
            unimplemented!()
        }

        let mut vec = SmallVec::new();

        let mut ring_iter = self.pages.iter_mut();
        while let Some(map) = ring_iter.next() {
            let mut rects = Vec::new();
            while let Some(index) = indices.next() {
                let key = GlyphKey {
                    font_file_hash: Font::file_hash(&font),
                    index,
                    size_px,
                };

                if let Some(item) = map.get_rect(&key) {
                    rects.push(item);
                } else if let Some(item) = map.pack(queue, font, index, size_px) {
                    rects.push(item);
                } else {
                    break;
                }
            }

            if rects.len() > 1 {
                vec.push(GlyphBatch {
                    view: map.create_view(),
                    rects,
                });
            }
        }

        if indices.peek().is_some() {
            let atlas =
                GlyphAtlasMap::init(device, Size2D::new(1024, 1024), TextureFormat::R8Unorm);
            self.pages.push(atlas);

            vec.append(&mut self.get_batch_inner(device, queue, font, indices, size_px));
        }

        vec
    }
}

impl Debug for GlyphCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlyphCache").finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub struct GlyphBatch {
    pub view: SizedTextureView2D,
    pub rects: Vec<GlyphTextureRect>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlyphKey {
    pub font_file_hash: u64,
    pub index: u16,
    pub size_px: u32,
}

impl GlyphKey {
    pub const fn new(font_file_hash: u64, index: u16, size_px: u32) -> Self {
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
    map: HashMap<GlyphKey, GlyphTextureRect>,
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

    pub fn create_view(&self) -> SizedTextureView2D {
        self.texture.inner().create_view_default(None)
    }

    pub fn get_rect(&self, key: &GlyphKey) -> Option<GlyphTextureRect> {
        Some(*self.map.get(key)?)
    }

    pub fn pack(
        &mut self,
        queue: &Queue,
        font: &Font,
        index: u16,
        size_px: u32,
    ) -> Option<GlyphTextureRect> {
        let key = GlyphKey {
            font_file_hash: Font::file_hash(font),
            index,
            size_px,
        };

        match font.glyph_bounding_box(GlyphId(index)) {
            Some(bounding_box) => {
                let mut builder = GlyphOutlineBuilder::new(font, bounding_box, size_px);
                font.outline_glyph(GlyphId(index), &mut builder);

                let rasterizer = builder.into_rasterizer();
                let mut data: Vec<u8> =
                    vec![0; rasterizer.dimensions().0 * rasterizer.dimensions().1];
                rasterizer.for_each_pixel(|i, alpha| data[i] = (alpha * 255.0) as u8);

                let offset = Vector2D::<f32, PixelUnit>::new(
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

                self.map.insert(
                    key,
                    GlyphTextureRect {
                        glyph_offset: offset,
                        rasterized_size: rect.size.cast(),
                        tex_rect: rect
                            .cast::<f32>()
                            .scale(
                                1.0 / self.texture.inner().size().width as f32,
                                1.0 / self.texture.inner().size().height as f32,
                            )
                            .cast_unit(),
                    },
                );
            }
            None => {
                return Some(GlyphTextureRect::default());
            }
        }

        self.get_rect(&key)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct GlyphTextureRect {
    pub glyph_offset: Vector2D<f32, PixelUnit>,
    pub rasterized_size: Size2D<f32, PixelUnit>,
    pub tex_rect: Rect<f32, TextureUnit>,
}

#[cfg(test)]
mod tests {
    use storyboard::core::wgpu::{Backends, Features, Instance};

    use storyboard::core::graphics::backend::{BackendOptions, StoryboardBackend};

    use crate::font::Font;

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

        let font = Font::from_slice(FONT, 0).unwrap();

        let mut cache = GlyphCache::new();

        println!(
            "Batch: {:?}",
            cache.get_batch(
                backend.device(),
                backend.queue(),
                &font,
                "test string"
                    .chars()
                    .map(|ch| font.glyph_index(ch).unwrap_or_default().0),
                40
            )
        );
    }
}
