/*
 * Created on Fri Nov 26 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::borrow::Cow;

use allsorts::{
    error::ParseError, font::MatchingPresentation, gpos::Info, gsub::Features,
    tables::FontTableProvider,
};
use font_kit::{
    canvas::{Canvas, RasterizationOptions},
    error::GlyphLoadingError,
    font::Font,
    hinting::HintingOptions,
    metrics::Metrics,
};
use pathfinder_geometry::{transform2d::Transform2F, vector::Vector2F};
use storyboard::{
    graphics::PixelUnit,
    math::{Point2D, Rect, Size2D},
};

pub struct DrawFont {
    shaper: allsorts::font::Font<FontKitFont>,
    metrics: Metrics,
}

impl DrawFont {
    pub fn new(font: Font) -> Self {
        let metrics = font.metrics();

        let shaper = allsorts::font::Font::new(FontKitFont::new(font))
            .unwrap()
            .unwrap();

        Self { shaper, metrics }
    }

    pub fn metrics(&self) -> &Metrics {
        &self.metrics
    }

    pub fn postscript_name(&self) -> String {
        let font = &self.shaper.font_table_provider.font;

        font.postscript_name().unwrap_or(font.full_name())
    }

    pub fn char_to_glyph(&self, ch: char) -> Option<u16> {
        self.shaper
            .font_table_provider
            .font
            .glyph_for_char(ch)
            .map(|id| id as u16)
    }

    pub fn line_height(&self) -> f32 {
        self.metrics.ascent + self.metrics.descent.abs() + self.metrics.line_gap
    }

    pub fn size_multiplier(&self, size: f32) -> f32 {
        size / self.metrics.units_per_em as f32
    }

    pub fn advance_x(&mut self, glyph: u16) -> Option<u16> {
        self.shaper.horizontal_advance(glyph)
    }

    pub fn advance_y(&mut self, glyph: u16) -> Option<u16> {
        self.shaper.vertical_advance(glyph)
    }

    pub fn raster_bounds(
        &self,
        id: u16,
        size: f32,
    ) -> Result<Rect<i32, PixelUnit>, GlyphLoadingError> {
        self.shaper.font_table_provider.raster_bounds(id, size)
    }

    pub fn rasterize(
        &self,
        canvas: &mut Canvas,
        offset: Point2D<i32, PixelUnit>,
        id: u16,
        size: f32,
    ) -> Result<(), GlyphLoadingError> {
        self.shaper
            .font_table_provider
            .rasterize(canvas, offset, id, size)
    }

    pub fn shape<'a>(
        &mut self,
        text: &str,
        script_tag: u32,
        matching: MatchingPresentation,
        lang_tag: Option<u32>,
        features: &Features,
        kerning: bool,
    ) -> Option<Vec<Info>> {
        let glyphs = self.shaper.map_glyphs(text, script_tag, matching);

        let infos = self
            .shaper
            .shape(glyphs, script_tag, lang_tag, features, kerning)
            .ok()?;

        Some(infos)
    }
}

#[derive(Debug)]
struct FontKitFont {
    font: Font,
}

impl FontKitFont {
    pub fn new(font: Font) -> Self {
        Self { font }
    }

    pub fn raster_bounds(
        &self,
        id: u16,
        size: f32,
    ) -> Result<Rect<i32, PixelUnit>, GlyphLoadingError> {
        let bounds = self.font.raster_bounds(
            id as u32,
            size,
            Transform2F::default(),
            HintingOptions::Full(size),
            RasterizationOptions::GrayscaleAa,
        )?;

        Ok(Rect::new(
            Point2D::new(bounds.origin_x(), bounds.origin_y()),
            Size2D::new(bounds.width(), bounds.height()),
        ))
    }

    pub fn rasterize(
        &self,
        canvas: &mut Canvas,
        offset: Point2D<i32, PixelUnit>,
        id: u16,
        size: f32,
    ) -> Result<(), GlyphLoadingError> {
        self.font.rasterize_glyph(
            canvas,
            id as u32,
            size,
            Transform2F::from_translation(Vector2F::new(offset.x as f32, offset.y as f32)),
            HintingOptions::Full(size),
            RasterizationOptions::GrayscaleAa,
        )?;

        Ok(())
    }
}

impl FontTableProvider for FontKitFont {
    fn table_data<'a>(&'a self, tag: u32) -> Result<Option<Cow<'a, [u8]>>, ParseError> {
        let data: Option<Cow<'a, [u8]>> = if let Some(data) = self.font.load_font_table(tag) {
            Some(Cow::Owned(Vec::from(data)))
        } else {
            None
        };

        Ok(data)
    }

    fn has_table<'a>(&'a self, tag: u32) -> bool {
        self.font.load_font_table(tag).is_some()
    }
}
