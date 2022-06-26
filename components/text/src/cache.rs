use std::{collections::HashMap, iter::Peekable};

use ringbuffer::{ConstGenericRingBuffer, RingBufferExt, RingBufferWrite};
use storyboard_core::{
    euclid::{Rect, Size2D, Vector2D},
    unit::PhyiscalPixelUnit,
};
use storyboard_render::{
    texture::{packed::PackedTexture, SizedTexture2D, SizedTextureView2D, TextureView2D},
    wgpu::{Device, Queue, TextureFormat, TextureUsages},
};

use crate::{rasterizer::{GlyphData, GlyphRasterizer}, font::Font};

#[derive(Debug)]
pub struct GlyphCache {
    pages: ConstGenericRingBuffer<GlyphAtlasMap, { Self::PAGES }>,
    colored_pages: ConstGenericRingBuffer<GlyphAtlasMap, { Self::PAGES }>,
}

impl GlyphCache {
    pub const PAGES: usize = 8;
    pub const PAGE_SIZE_LIMIT: u32 = 256;

    pub fn new() -> Self {
        Self {
            pages: ConstGenericRingBuffer::new(),
            colored_pages: ConstGenericRingBuffer::new(),
        }
    }

    pub fn batch(
        &mut self,
        device: &Device,
        queue: &Queue,
        font: &Font,
        indices: &mut Peekable<impl Iterator<Item = u16>>,
        size_px: u32,
    ) -> Option<GlyphBatch> {
        self.batch_glyph(device, queue, font, indices, size_px)
            .or_else(|| self.batch_image(device, queue, font, indices, size_px))
    }

    pub fn batch_image(
        &mut self,
        device: &Device,
        queue: &Queue,
        font: &Font,
        indices: &mut Peekable<impl Iterator<Item = u16>>,
        size_px: u32,
    ) -> Option<GlyphBatch> {
        let mut rects = Vec::new();

        let mut page_iter = self.colored_pages.iter_mut();
        while let Some(page) = page_iter.next() {
            while let Some(index) = indices.peek() {
                let key = GlyphKey {
                    font_hash: Font::font_hash(font),
                    index: *index,
                    size_px,
                };

                if let Some(item) = page.get_rect(&key) {
                    rects.push(item);
                } else {
                    let mut rasterizer = GlyphRasterizer::new(font);

                    if let Some(glyph) = rasterizer.rasterize_image(*index, size_px as f32) {
                        if let Some(rect) = page.pack(queue, key, &glyph) {
                            rects.push(rect);
                        } else {
                            break;
                        }
                    } else if rects.len() > 0 {
                        return Some(GlyphBatch {
                            view: page.create_view().into(),
                            rects,
                        });
                    } else {
                        return None;
                    }
                }

                indices.next();
            }

            if rects.len() > 0 {
                return Some(GlyphBatch {
                    view: page.create_view().into(),
                    rects,
                });
            }
        }

        if indices.peek().is_some() {
            let atlas =
                GlyphAtlasMap::init(device, Size2D::new(1024, 1024), TextureFormat::Rgba8Unorm);
            self.colored_pages.push(atlas);

            return self.batch_image(device, queue, font, indices, size_px);
        }

        None
    }

    pub fn batch_glyph(
        &mut self,
        device: &Device,
        queue: &Queue,
        font: &Font,
        indices: &mut Peekable<impl Iterator<Item = u16>>,
        size_px: u32,
    ) -> Option<GlyphBatch> {
        let mut rects = Vec::new();

        let mut page_iter = self.pages.iter_mut();
        while let Some(page) = page_iter.next() {
            while let Some(index) = indices.peek() {
                let key = GlyphKey {
                    font_hash: Font::font_hash(font),
                    index: *index,
                    size_px,
                };

                if let Some(item) = page.get_rect(&key) {
                    rects.push(item);
                } else {
                    let rasterizer = GlyphRasterizer::new(font);

                    if let Some(glyph) = rasterizer.rasterize_glyph(*index, size_px as f32) {
                        if let Some(rect) = page.pack(queue, key, &glyph) {
                            rects.push(rect);
                        } else {
                            break;
                        }
                    } else if rects.len() > 0 {
                        return Some(GlyphBatch {
                            view: page.create_view().into(),
                            rects,
                        });
                    } else {
                        return None;
                    }
                }

                indices.next();
            }

            if rects.len() > 0 {
                return Some(GlyphBatch {
                    view: page.create_view().into(),
                    rects,
                });
            }
        }

        if indices.peek().is_some() {
            let atlas =
                GlyphAtlasMap::init(device, Size2D::new(1024, 1024), TextureFormat::R8Unorm);
            self.pages.push(atlas);

            return self.batch_glyph(device, queue, font, indices, size_px);
        }

        None
    }
}

#[derive(Debug)]
pub struct GlyphBatch {
    pub view: TextureView2D,
    pub rects: Vec<GlyphTextureRect>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlyphKey {
    pub font_hash: u64,
    pub index: u16,
    pub size_px: u32,
}

impl GlyphKey {
    pub const fn new(font_hash: u64, index: u16, size_px: u32) -> Self {
        Self {
            font_hash,
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
    pub fn init(
        device: &Device,
        size: Size2D<u32, PhyiscalPixelUnit>,
        format: TextureFormat,
    ) -> Self {
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
        key: GlyphKey,
        glyph: &GlyphData,
    ) -> Option<GlyphTextureRect> {
        let tex_rect = if glyph.data.len() > 0 {
            self.texture.pack(queue, glyph.size, &glyph.data)?
        } else {
            Rect::zero()
        };

        self.map.insert(
            key,
            GlyphTextureRect {
                glyph_offset: glyph.offset,
                tex_rect,
            },
        );

        self.get_rect(&key)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct GlyphTextureRect {
    pub glyph_offset: Vector2D<f32, PhyiscalPixelUnit>,
    pub tex_rect: Rect<u32, PhyiscalPixelUnit>,
}
