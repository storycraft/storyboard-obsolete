use std::{
    fmt::Debug,
    iter::{Enumerate, Peekable, Zip},
    ops::Range,
    slice::Iter,
    str::CharIndices,
};

use rustybuzz::{GlyphBuffer, GlyphPosition, UnicodeBuffer};
use storyboard_core::{euclid::Vector2D, unit::PhyiscalPixelUnit};
use ttf_parser::Face;

#[derive(Debug)]
pub struct TextLayout<'a> {
    face: &'a Face<'a>,

    ascender: i16,
    descender: i16,

    space_size: u16,

    units_per_em: u16,

    text: &'a str,
}

impl<'a> TextLayout<'a> {
    pub fn new(face: &'a Face, text: &'a str) -> Self {
        let units_per_em = face.units_per_em();

        let ascender = face.ascender();
        let descender = face.descender();

        let space_size = face
            .glyph_index(' ')
            .map(|id| face.glyph_hor_advance(id))
            .flatten()
            .unwrap_or_default();

        Self {
            face,

            ascender,
            descender,

            space_size,

            units_per_em,

            text,
        }
    }

    pub fn iter(&self, tab_size: u32, size_px: f32) -> TextLayoutIter<'a> {
        TextLayoutIter {
            face: rustybuzz::Face::from_face(self.face.clone()).unwrap(),

            current_position: Vector2D::zero(),

            text: self.text,
            text_iter: self.text.char_indices().enumerate().peekable(),

            ascender: self.ascender,
            descender: self.descender,

            tab_width: self.space_size as u32 * tab_size,

            scale: size_px / self.units_per_em as f32,

            current: None,
        }
    }
}

pub struct TextLayoutIter<'a> {
    face: rustybuzz::Face<'a>,

    current_position: Vector2D<f32, PhyiscalPixelUnit>,

    text: &'a str,
    text_iter: Peekable<Enumerate<CharIndices<'a>>>,

    ascender: i16,
    descender: i16,

    tab_width: u32,

    scale: f32,

    current: Option<GlyphBuffer>,
}

impl<'a> TextLayoutIter<'a> {
    pub fn ascender(&self) -> f32 {
        self.ascender as f32 * self.scale
    }

    pub fn descender(&self) -> f32 {
        self.descender as f32 * self.scale
    }

    pub fn tab_width(&self) -> f32 {
        self.tab_width as f32 * self.scale
    }

    fn get_placement_for(&self, ch: char) -> Option<TextPlacement> {
        match ch {
            '\n' => Some(TextPlacement::Set(Vector2D::new(
                0.0,
                self.current_position.y
                    + (self.ascender as f32 - self.descender as f32) * self.scale,
            ))),

            '\r' => Some(TextPlacement::Set(Vector2D::new(
                0.0,
                self.current_position.y,
            ))),

            '\t' => Some(TextPlacement::Offset(Vector2D::new(
                self.tab_width as f32 * self.scale,
                0.0,
            ))),

            '\x0C' => Some(TextPlacement::Offset(Vector2D::new(
                0.0,
                (self.ascender as f32 - self.descender as f32) * self.scale,
            ))),

            _ => None,
        }
    }

    fn update_next_text_slice(&mut self) -> Option<TextSlice> {
        let (cluster_offset, (start_offset, start_ch)) = *self.text_iter.peek()?;

        let mut next_placement = TextPlacement::default();

        let mut end_offset = start_offset + start_ch.len_utf8();

        while let Some((_, (start_pos, ch))) = self.text_iter.next() {
            end_offset = start_pos + ch.len_utf8();

            if let Some(placement) = self.get_placement_for(ch) {
                next_placement = placement;
                break;
            }
        }

        Some(TextSlice {
            range: start_offset..end_offset,
            cluster_offset,
            next_placement,
        })
    }

    fn get_next_position(
        &self,
        buffer: &GlyphBuffer,
        next_placement: TextPlacement,
    ) -> Vector2D<f32, PhyiscalPixelUnit> {
        let mut next_position = self.current_position;
        for pos in buffer.glyph_positions() {
            next_position += Vector2D::new(
                pos.x_advance as f32 * self.scale,
                pos.y_advance as f32 * self.scale,
            );
        }

        next_placement.get_next_placement(next_position)
    }

    fn shape_text(&mut self, text: &str) -> GlyphBuffer {
        let mut shape_buffer = self
            .current
            .take()
            .map(|buffer| buffer.clear())
            .unwrap_or_default();

        shape_buffer.push_str(text);
        shape_buffer.guess_segment_properties();

        rustybuzz::shape(&self.face, &[], shape_buffer)
    }

    pub fn next<'iter>(
        &'iter mut self,
    ) -> Option<(impl Iterator<Item = u16> + 'iter, LineLayoutIter<'iter>)> {
        let slice = self.update_next_text_slice()?;

        let shape_buffer = self.shape_text(&self.text[slice.range]);
        self.current = Some(shape_buffer);
        let current = self.current.as_ref().unwrap();

        let indices_iter = current
            .glyph_infos()
            .iter()
            .map(|info| info.glyph_id as u16);

        let line_iter = LineLayoutIter {
            scale: self.scale,
            cluster_offset: slice.cluster_offset as u32,
            current_position: self.current_position,
            iter: current
                .glyph_infos()
                .iter()
                .zip(current.glyph_positions().iter()),
        };

        self.current_position = self.get_next_position(current, slice.next_placement);

        Some((indices_iter, line_iter))
    }
}

impl Debug for TextLayoutIter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextLayoutIter")
            .field("current_position", &self.current_position)
            .field("text", &self.text)
            .field("ascender", &self.ascender)
            .field("descender", &self.descender)
            .field("tab_width", &self.tab_width)
            .field("scale", &self.scale)
            .field("current", &self.current)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TextPlacement {
    Set(Vector2D<f32, PhyiscalPixelUnit>),
    Offset(Vector2D<f32, PhyiscalPixelUnit>),
}

impl Default for TextPlacement {
    fn default() -> Self {
        Self::Offset(Vector2D::zero())
    }
}

impl TextPlacement {
    pub fn get_next_placement(
        &self,
        current: Vector2D<f32, PhyiscalPixelUnit>,
    ) -> Vector2D<f32, PhyiscalPixelUnit> {
        match self {
            Self::Set(pos) => *pos,
            Self::Offset(offset) => current + offset,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextSlice {
    pub range: Range<usize>,
    pub cluster_offset: usize,
    pub next_placement: TextPlacement,
}

#[derive(Debug)]
pub struct LineLayout {
    units_per_em: u16,

    buffer: GlyphBuffer,
}

impl LineLayout {
    pub fn new_layout(face: &Face, buffer: UnicodeBuffer) -> Self {
        let buffer = rustybuzz::shape(
            &rustybuzz::Face::from_face(face.clone()).unwrap(),
            &[],
            buffer,
        );

        let units_per_em = face.units_per_em();

        Self {
            units_per_em,

            buffer,
        }
    }

    pub fn indices<'a>(&'a self) -> impl Iterator<Item = u16> + 'a {
        self.buffer
            .glyph_infos()
            .iter()
            .map(|info| info.glyph_id as u16)
    }

    pub fn iter(&self, size_px: f32) -> LineLayoutIter {
        LineLayoutIter {
            scale: size_px / self.units_per_em as f32,
            cluster_offset: 0,
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
pub struct LineLayoutIter<'a> {
    scale: f32,
    cluster_offset: u32,
    current_position: Vector2D<f32, PhyiscalPixelUnit>,
    iter: Zip<Iter<'a, rustybuzz::GlyphInfo>, Iter<'a, GlyphPosition>>,
}

impl<'a> Iterator for LineLayoutIter<'a> {
    type Item = GlyphInfo;

    fn next(&mut self) -> Option<Self::Item> {
        let (info, pos) = self.iter.next()?;

        let glyph_info = GlyphInfo {
            glyph_id: info.glyph_id as u16,
            cluster: self.cluster_offset + info.cluster,
            position: self.current_position
                - Vector2D::new(
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
    pub cluster: u32,
    pub position: Vector2D<f32, PhyiscalPixelUnit>,
}
