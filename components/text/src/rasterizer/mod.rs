pub mod outline;

use storyboard_core::{
    euclid::{Point2D, Rect, Size2D, Vector2D},
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

    pub fn rasterize_image(&mut self, _index: u16, _size_px: f32) -> Option<GlyphData> {
        // TODO::

        None
    }

    pub fn rasterize_glyph(&self, index: u16, size_px: f32) -> Option<GlyphData> {
        let bounding_box = {
            let bounding_box = self.face.glyph_bounding_box(GlyphId(index))?;

            Rect::new(
                Point2D::new(bounding_box.x_min, bounding_box.y_min),
                Size2D::new(bounding_box.width(), bounding_box.height()),
            )
            .cast()
        };

        let mut builder = GlyphOutlineBuilder::new(
            bounding_box,
            size_px as f32 / self.face.units_per_em() as f32,
        );

        self.face.outline_glyph(GlyphId(index), &mut builder)?;

        Some(builder.get_glyph_data())
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
    pub origin: Vector2D<f32, PhyiscalPixelUnit>,
    pub size: Size2D<u32, PhyiscalPixelUnit>,
    pub data: Vec<u8>,
}
