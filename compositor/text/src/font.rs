/*
 * Created on Tue Oct 19 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{borrow::Cow, ops::Deref, rc::Rc};

use allsorts::{
    error::ParseError,
    tables::FontTableProvider,
};
use font_kit::{font::Font, metrics::Metrics};

pub struct FontUnit;

pub struct DrawFont {
    font: InnerFont,
    shaper: allsorts::font::Font<InnerFont>,
    metrics: Metrics,
}

impl DrawFont {
    pub fn new(font: Font) -> Self {
        let font = InnerFont(Rc::new(font));
        let shaper = allsorts::font::Font::new(font.clone()).unwrap().unwrap();

        let metrics = font.metrics();

        Self {
            font,
            shaper,
            metrics,
        }
    }

    pub fn font(&self) -> &Font {
        &self.font
    }

    pub fn shaper(&self) -> &allsorts::font::Font<InnerFont> {
        &self.shaper
    }

    pub fn shaper_mut(&mut self) -> &mut allsorts::font::Font<InnerFont> {
        &mut self.shaper
    }

    pub fn metrics(&self) -> &Metrics {
        &self.metrics
    }

    #[inline]
    pub fn line_height(&self) -> f32 {
        self.metrics.ascent + self.metrics.descent.abs() + self.metrics.line_gap
    }

    #[inline]
    pub fn size_multiplier(&self, size: f32) -> f32 {
        size / self.metrics.units_per_em as f32
    }

    pub fn advance_x(&mut self, glyph: u16) -> Option<u16> {
        self.shaper.horizontal_advance(glyph)
    }

    pub fn advance_y(&mut self, glyph: u16) -> Option<u16> {
        self.shaper.vertical_advance(glyph)
    }
}


#[derive(Debug, Clone)]
pub struct InnerFont(Rc<Font>);

impl Deref for InnerFont {
    type Target = Font;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FontTableProvider for InnerFont {
    fn table_data<'a>(&'a self, tag: u32) -> Result<Option<Cow<'a, [u8]>>, ParseError> {
        if let Some(table) = self.0.load_font_table(tag) {
            Ok(Some(Cow::Owned(Vec::from(table))))
        } else {
            Err(ParseError::MissingValue)
        }
    }

    fn has_table<'a>(&'a self, tag: u32) -> bool {
        self.0.load_font_table(tag).is_some()
    }
}
