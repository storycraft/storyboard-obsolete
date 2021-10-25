/*
 * Created on Tue Oct 19 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use font_kit::{font::Font, metrics::Metrics};

pub struct FontUnit;

#[derive(Debug)]
pub struct DrawFont {
    font: Font,
    metrics: Metrics,
}

impl DrawFont {
    pub fn new(font: Font) -> Self {
        let metrics = font.metrics();

        Self { font, metrics }
    }

    pub fn font(&self) -> &Font {
        &self.font
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
}
