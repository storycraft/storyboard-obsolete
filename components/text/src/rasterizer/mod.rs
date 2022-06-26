pub mod outline;

use storyboard_core::{
    euclid::{Size2D, Vector2D},
    unit::PhyiscalPixelUnit,
};
use ttf_parser::{Face, GlyphId};

use self::outline::GlyphOutlineBuilder;

pub struct GlyphRasterizer<'a> {
    face: &'a Face<'a>,
}

impl<'a> GlyphRasterizer<'a> {
    pub fn new(face: &'a Face<'a>) -> Self {
        Self { face }
    }

    pub fn rasterize_image(&mut self, index: u16, size_px: f32) -> Option<GlyphData> {
        // TODO::

        None
    }

    pub fn rasterize_glyph(&self, index: u16, size_px: f32) -> Option<GlyphData> {
        let mut builder = GlyphOutlineBuilder::new();

        if self
            .face
            .outline_glyph(GlyphId(index), &mut builder)
            .is_some()
            || builder.is_empty()
        {
            Some(builder.rasterize(size_px as f32 / self.face.units_per_em() as f32))
        } else {
            None
        }
    }

    pub fn rasterize(&mut self, index: u16, size_px: f32) -> Option<RasterizedGlyph> {
        self.rasterize_glyph(index, size_px)
            .map(RasterizedGlyph::Glyph)
            .or_else(|| {
                self.rasterize_image(index, size_px)
                    .map(RasterizedGlyph::Image)
            })
    }
}

#[derive(Debug)]
pub enum RasterizedGlyph {
    Glyph(GlyphData),
    Image(GlyphData),
}

#[derive(Debug, Clone, Default)]
pub struct GlyphData {
    pub offset: Vector2D<f32, PhyiscalPixelUnit>,
    pub size: Size2D<u32, PhyiscalPixelUnit>,
    pub data: Vec<u8>,
}
