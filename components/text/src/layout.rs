use std::{iter::Zip, slice::Iter};

use rustybuzz::{GlyphBuffer, GlyphPosition, UnicodeBuffer};
use storyboard_core::{euclid::Vector2D, unit::PhyiscalPixelUnit};
use ttf_parser::Face;

#[derive(Debug)]
pub struct TextLayout {
    ascender: i16,
    decender: i16,
    units_per_em: u16,

    buffer: GlyphBuffer,
}

impl TextLayout {
    pub fn new_layout(face: &Face, buffer: UnicodeBuffer) -> Self {
        let buffer = rustybuzz::shape(&rustybuzz::Face::from_face(face.clone()).unwrap(), &[], buffer);

        let units_per_em = face.units_per_em();

        Self {
            ascender: face.ascender(),
            decender: face.descender(),
            units_per_em,

            buffer,
        }
    }

    pub fn indices<'a>(&'a self) -> impl Iterator<Item = u16> + 'a {
        self.buffer.glyph_infos().iter().map(|info| info.glyph_id as u16)
    }

    pub fn iter(&self, size_px: f32) -> TextLayoutIter {
        TextLayoutIter {
            ascender: self.ascender,
            decender: self.decender,

            scale: size_px / self.units_per_em as f32,
            current_position: Vector2D::zero(),
            iter: self
                .buffer
                .glyph_infos()
                .iter()
                .zip(self.buffer.glyph_positions().iter()),
        }
    }

    pub fn into_inner(self) -> UnicodeBuffer {
        self.buffer.clear()
    }
}

#[derive(Debug)]
pub struct TextLayoutIter<'a> {
    ascender: i16,
    decender: i16,

    scale: f32,
    current_position: Vector2D<f32, PhyiscalPixelUnit>,
    iter: Zip<Iter<'a, rustybuzz::GlyphInfo>, Iter<'a, GlyphPosition>>,
}

impl<'a> TextLayoutIter<'a> {
    pub fn ascender(&self) -> f32 {
        self.ascender as f32 * self.scale
    }

    pub fn decender(&self) -> f32 {
        self.decender as f32 * self.scale
    }
}

impl<'a> Iterator for TextLayoutIter<'a> {
    type Item = GlyphInfo;

    fn next(&mut self) -> Option<Self::Item> {
        let (info, pos) = self.iter.next()?;

        let glyph_info = GlyphInfo {
            glyph_id: info.glyph_id as u16,
            position: self.current_position
                + Vector2D::new(
                    pos.x_offset as f32 * self.scale,
                    pos.y_offset as f32 * self.scale,
                ),
        };

        let advance = Vector2D::new(
            pos.x_advance as f32 * self.scale,
            pos.y_advance as f32 * self.scale,
        );
        self.current_position += advance;

        return Some(glyph_info);
    }
}

#[derive(Debug)]
pub struct GlyphInfo {
    pub glyph_id: u16,
    pub position: Vector2D<f32, PhyiscalPixelUnit>,
}
