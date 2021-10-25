/*
 * Created on Fri Oct 22 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use storyboard::{
    math::{Point2D, Size2D},
    unit::PixelUnit,
};

use crate::font::DrawFont;

#[derive(Debug, Clone)]
pub struct TextLayout<'a> {
    draw_font: &'a DrawFont,

    line_height: f32,

    max_width: f32,
    offset: Point2D<f32, PixelUnit>,
}

impl<'a> TextLayout<'a> {
    pub fn new(draw_font: &'a DrawFont) -> Self {
        let metrics = draw_font.metrics();

        let line_height = metrics.ascent - metrics.descent + metrics.line_gap;

        Self {
            draw_font,

            line_height,

            max_width: 0.0,
            offset: Point2D::new(0.0, metrics.ascent),
        }
    }

    pub fn measure(&mut self) -> Size2D<f32, PixelUnit> {
        self.max_width = self.max_width.max(self.offset.x);

        let metrics = self.draw_font.metrics();

        Size2D::new(self.max_width, self.offset.y - metrics.descent + metrics.line_gap)
    }
    
    pub fn next_item(&mut self, ch: char) -> TextGlyphItem {
        let offset = self.offset;

        let glyph_id = self.draw_font.font().glyph_for_char(ch);

        let item_size = if let Some(glyph_id) = glyph_id {
            Size2D::new(
                self.draw_font.font().advance(glyph_id).unwrap().x(),
                self.line_height,
            )
        } else {
            Size2D::zero()
        };

        if item_size.width != 0.0 {
            self.offset.x += item_size.width;
        }

        if ch == '\n' {
            self.max_width = self.max_width.max(self.offset.x);
            self.offset.x = 0.0;
            self.offset.y += self.line_height;
        }

        TextGlyphItem {
            ch,
            glyph_id,
            offset,
            item_size,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TextGlyphItem {
    pub ch: char,
    pub glyph_id: Option<u32>,
    pub offset: Point2D<f32, PixelUnit>,
    pub item_size: Size2D<f32, PixelUnit>,
}

#[cfg(test)]
#[test]
pub fn test_layout() {
    use font_kit::source::SystemSource;

    let font = SystemSource::new()
        .select_by_postscript_name("ArialMT")
        .unwrap()
        .load()
        .unwrap();

    let draw_font = DrawFont::new(font);
    let mut layout = TextLayout::new(&draw_font);

    for ch in "Hello world!\nHello world!".chars() {
        println!("item: {:?}", layout.next_item(ch));
    }

    println!("measure: {:?}", layout.measure());
}
