use std::{
    fmt::Debug,
    iter::{Enumerate, Peekable, Zip},
    slice::Iter,
    str::Chars,
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
            next_placement: ParagraphPlacement::default(),

            text: self.text.chars().enumerate().peekable(),

            ascender: self.ascender,
            descender: self.descender,

            tab_width: self.space_size as u32 * tab_size,

            scale: size_px / self.units_per_em as f32,

            buf: String::new(),
            current: None,
        }
    }
}

pub struct TextLayoutIter<'a> {
    face: rustybuzz::Face<'a>,

    current_position: Vector2D<f32, PhyiscalPixelUnit>,
    next_placement: ParagraphPlacement,

    text: Peekable<Enumerate<Chars<'a>>>,

    ascender: i16,
    descender: i16,

    tab_width: u32,

    scale: f32,

    buf: String,
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

    pub fn next<'iter>(
        &'iter mut self,
    ) -> Option<(impl Iterator<Item = u16> + 'iter, LineLayoutIter<'iter>)> {
        let cluster_offset = self.text.peek()?.0 as u32;
        self.current_position = self
            .next_placement
            .get_next_placement(self.current_position);

        self.next_placement = ParagraphPlacement::default();
        for (_, ch) in &mut self.text {
            match ch {
                '\n' => {
                    self.next_placement = ParagraphPlacement::Set(Vector2D::new(
                        0.0,
                        self.current_position.y
                            + (self.ascender as f32 - self.descender as f32) * self.scale,
                    ));
                    break;
                }

                '\r' => {
                    self.next_placement =
                        ParagraphPlacement::Set(Vector2D::new(0.0, self.current_position.y));
                    break;
                }

                '\t' => {
                    self.next_placement = ParagraphPlacement::Offset(Vector2D::new(
                        self.tab_width as f32 * self.scale,
                        0.0,
                    ));
                    break;
                }

                '\x0C' => {
                    self.next_placement = ParagraphPlacement::Offset(Vector2D::new(
                        0.0,
                        (self.ascender as f32 - self.descender as f32) * self.scale,
                    ));
                    break;
                }

                _ => {
                    self.buf.push(ch);
                }
            }
        }

        let mut shape_buffer = self
            .current
            .take()
            .map(|buffer| buffer.clear())
            .unwrap_or_default();

        shape_buffer.push_str(&self.buf);
        shape_buffer.guess_segment_properties();
        self.buf.clear();

        let buffer = rustybuzz::shape(&self.face, &[], shape_buffer);
        self.current = Some(buffer);
        let current = self.current.as_ref().unwrap();

        let current_position = self.current_position;

        for pos in current.glyph_positions() {
            self.current_position += Vector2D::new(
                pos.x_advance as f32 * self.scale,
                pos.y_advance as f32 * self.scale,
            );
        }

        let indices_iter = current
            .glyph_infos()
            .iter()
            .map(|info| info.glyph_id as u16);

        let line_iter = LineLayoutIter {
            scale: self.scale,
            cluster_offset,
            current_position,
            iter: current
                .glyph_infos()
                .iter()
                .zip(current.glyph_positions().iter()),
        };

        Some((indices_iter, line_iter))
    }
}

impl Debug for TextLayoutIter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextLayoutIter")
            .field("next_position", &self.next_placement)
            .field("text", &self.text)
            .field("ascender", &self.ascender)
            .field("descender", &self.descender)
            .field("tab_width", &self.tab_width)
            .field("scale", &self.scale)
            .field("buf", &self.buf)
            .field("current", &self.current)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Copy)]
enum ParagraphPlacement {
    Set(Vector2D<f32, PhyiscalPixelUnit>),
    Offset(Vector2D<f32, PhyiscalPixelUnit>),
}

impl Default for ParagraphPlacement {
    fn default() -> Self {
        Self::Offset(Vector2D::zero())
    }
}

impl ParagraphPlacement {
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
