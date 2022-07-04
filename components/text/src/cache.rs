use std::{collections::HashMap, fmt::Debug, iter::Peekable};

use rect_packer::DensePacker;
use ringbuffer::{ConstGenericRingBuffer, RingBufferExt, RingBufferWrite};
use storyboard_core::{
    euclid::{Point2D, Rect, Size2D, Vector2D},
    unit::PhyiscalPixelUnit,
};
use storyboard_render::{
    texture::{SizedTexture2D, SizedTextureView2D, TextureView2D},
    wgpu::{Device, Queue, TextureFormat, TextureUsages},
};

use crate::{
    font::Font,
    rasterizer::{GlyphData, GlyphRasterizer},
};

#[derive(Debug, Default)]
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
                    } else if !rects.is_empty() {
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

            if !rects.is_empty() {
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
        glyph_indices: &mut Peekable<impl Iterator<Item = u16>>,
        size_px: u32,
    ) -> Option<GlyphBatch> {
        let mut rects = Vec::new();

        let mut page_iter = self.pages.iter_mut();
        while let Some(page) = page_iter.next() {
            while let Some(index) = glyph_indices.peek() {
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
                    } else if !rects.is_empty() {
                        return Some(GlyphBatch {
                            view: page.create_view().into(),
                            rects,
                        });
                    } else {
                        return None;
                    }
                }

                glyph_indices.next();
            }

            if !rects.is_empty() {
                return Some(GlyphBatch {
                    view: page.create_view().into(),
                    rects,
                });
            }
        }

        if glyph_indices.peek().is_some() {
            let atlas =
                GlyphAtlasMap::init(device, Size2D::new(1024, 1024), TextureFormat::R8Unorm);
            self.pages.push(atlas);

            return self.batch_glyph(device, queue, font, glyph_indices, size_px);
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

pub struct GlyphAtlasMap {
    texture: SizedTexture2D,
    packer: DensePacker,
    map: HashMap<GlyphKey, GlyphTextureRect>,
}

impl GlyphAtlasMap {
    pub fn init(
        device: &Device,
        size: Size2D<u32, PhyiscalPixelUnit>,
        format: TextureFormat,
    ) -> Self {
        let texture = SizedTexture2D::init(
            device,
            Some("GlyphAtlasTexture texture"),
            size,
            format,
            TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        );

        Self {
            texture,
            packer: DensePacker::new(size.width as i32, size.height as i32),
            map: HashMap::new(),
        }
    }

    pub fn create_view(&self) -> SizedTextureView2D {
        self.texture.create_view_default(None)
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
        let tex_rect = if !glyph.data.is_empty() {
            self.packer
                .pack(glyph.size.width as i32, glyph.size.height as i32, false)
                .map(|rect| {
                    Rect::new(
                        Point2D::new(rect.x as u32, rect.y as u32),
                        Size2D::new(rect.width as u32, rect.height as u32),
                    )
                })?
        } else {
            Rect::zero()
        };
        
        self.texture.write(queue, Some(tex_rect), &glyph.data);
        self.map.insert(
            key,
            GlyphTextureRect {
                glyph_offset: glyph.origin,
                tex_rect,
            },
        );

        self.get_rect(&key)
    }
}

impl Debug for GlyphAtlasMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlyphAtlasMap")
            .field("texture", &self.texture)
            .field("map", &self.map)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct GlyphTextureRect {
    pub glyph_offset: Vector2D<f32, PhyiscalPixelUnit>,
    pub tex_rect: Rect<u32, PhyiscalPixelUnit>,
}
