use std::{
    fmt::Debug,
    iter::{Enumerate, Peekable, Zip},
    ops::{Deref, Range},
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

            shape_buffer: None,
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

    shape_buffer: Option<UnicodeBuffer>,
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

    fn next_text_slice(&mut self) -> Option<TextSlice> {
        let (cluster_offset, (start_offset, start_ch)) = *self.text_iter.peek()?;

        let mut next_placement = TextPlacement::default();

        let mut end_offset = start_offset + start_ch.len_utf8();

        while let Some((_, (start_pos, ch))) = self.text_iter.next() {
            if let Some(placement) = self.get_placement_for(ch) {
                end_offset = start_pos;
                next_placement = placement;
                break;
            } else {
                end_offset = start_pos + ch.len_utf8();
            }
        }

        Some(TextSlice {
            range: start_offset..end_offset,
            cluster_offset,
            next_placement,
        })
    }

    fn shape_text(&mut self, mut shape_buffer: UnicodeBuffer, text: &str) -> GlyphBuffer {
        shape_buffer.push_str(text);
        shape_buffer.guess_segment_properties();

        rustybuzz::shape(&self.face, &[], shape_buffer)
    }

    pub fn next<'iter>(&'iter mut self) -> Option<SpanLayoutRef<'iter, 'a>> {
        let slice = self.next_text_slice()?;

        let shape_buffer = self.shape_buffer.take().unwrap_or_default();
        let shape_buffer = self.shape_text(shape_buffer, &self.text[slice.range]);

        let line_layout = SpanLayout {
            scale: self.scale,
            current_position: self.current_position,
            buffer: shape_buffer,
        };

        self.current_position = slice
            .next_placement
            .get_next_placement(self.current_position + line_layout.get_total_advance());

        Some(SpanLayoutRef {
            iter: self,
            layout: Some(line_layout),
        })
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
            .field("shape_buffer", &self.shape_buffer)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub struct SpanLayoutRef<'a, 'text> {
    iter: &'a mut TextLayoutIter<'text>,
    layout: Option<SpanLayout>,
}

impl Deref for SpanLayoutRef<'_, '_> {
    type Target = SpanLayout;

    fn deref(&self) -> &Self::Target {
        self.layout.as_ref().unwrap()
    }
}

impl Drop for SpanLayoutRef<'_, '_> {
    fn drop(&mut self) {
        self.iter.shape_buffer = Some(self.layout.take().unwrap().buffer.clear())
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
pub struct SpanLayout {
    scale: f32,
    pub current_position: Vector2D<f32, PhyiscalPixelUnit>,
    buffer: GlyphBuffer,
}

impl SpanLayout {
    pub fn shape_str(face: &Face, size_px: f32, text: &str) -> Self {
        let mut shape_buffer = UnicodeBuffer::new();
        shape_buffer.push_str(text);

        let buffer = rustybuzz::shape(
            &rustybuzz::Face::from_face(face.clone()).unwrap(),
            &[],
            shape_buffer,
        );

        let scale = size_px / face.units_per_em() as f32;

        Self {
            scale,
            current_position: Vector2D::zero(),

            buffer,
        }
    }

    pub fn shape_from_buffer(face: &Face, size_px: f32, buffer: UnicodeBuffer) -> Self {
        let buffer = rustybuzz::shape(
            &rustybuzz::Face::from_face(face.clone()).unwrap(),
            &[],
            buffer,
        );

        let scale = size_px / face.units_per_em() as f32;

        Self {
            scale,
            current_position: Vector2D::zero(),

            buffer,
        }
    }

    pub fn glyph_id_iter<'a>(&'a self) -> impl Iterator<Item = u16> + 'a {
        self.buffer
            .glyph_infos()
            .iter()
            .map(|info| info.glyph_id as u16)
    }

    pub fn get_total_advance(&self) -> Vector2D<f32, PhyiscalPixelUnit> {
        let mut total = Vector2D::zero();

        for pos in self.buffer.glyph_positions() {
            total += Vector2D::new(
                pos.x_advance as f32 * self.scale,
                pos.y_advance as f32 * self.scale,
            );
        }

        total
    }

    pub fn iter(&self) -> SpanLayoutIter {
        SpanLayoutIter {
            scale: self.scale,
            cluster_offset: 0,
            current_position: self.current_position,
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
pub struct SpanLayoutIter<'a> {
    scale: f32,
    cluster_offset: u32,
    current_position: Vector2D<f32, PhyiscalPixelUnit>,
    iter: Zip<Iter<'a, rustybuzz::GlyphInfo>, Iter<'a, GlyphPosition>>,
}

impl<'a> Iterator for SpanLayoutIter<'a> {
    type Item = GlyphInfo;

    fn next(&mut self) -> Option<Self::Item> {
        let (info, pos) = self.iter.next()?;

        let glyph_info = GlyphInfo {
            glyph_id: info.glyph_id as u16,
            cluster: self.cluster_offset + info.cluster,
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
    pub cluster: u32,
    pub position: Vector2D<f32, PhyiscalPixelUnit>,
}
