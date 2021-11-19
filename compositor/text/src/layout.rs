/*
 * Created on Fri Oct 22 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::str::Lines;

use allsorts::{
    font::MatchingPresentation,
    glyph_position::{GlyphLayout, TextDirection},
    gsub::{FeatureMask, Features},
    tinyvec::TinyVec,
};
use storyboard_graphics::math::{Point2D, Rect, Size2D};

use crate::font::{DrawFont, FontUnit};

pub struct TextLayout<'a> {
    font: &'a mut DrawFont,
    lines: Lines<'a>,

    direction: TextDirection,
    match_presentation: MatchingPresentation,
    script_tag: u32,
    opt_lang_tag: Option<u32>,
    features: Features,
    kerning: bool,

    line_height: i32,

    max_width: i32,
    line_offset: i32,
}

impl<'a> TextLayout<'a> {
    pub fn new(
        font: &'a mut DrawFont,
        text: &'a str,
        direction: TextDirection,
        match_presentation: MatchingPresentation,
        script_tag: u32,
        opt_lang_tag: Option<u32>,
        features: Option<Features>,
        kerning: bool,
    ) -> Self {
        let metrics = font.metrics();
        let line_height = (metrics.ascent - metrics.descent + metrics.line_gap) as i32;

        Self {
            font,
            lines: text.lines(),

            direction,
            match_presentation,
            script_tag,
            opt_lang_tag,
            features: features.unwrap_or(Features::Mask(FeatureMask::default())),
            kerning,

            line_height,

            max_width: 0,
            line_offset: 0,
        }
    }

    pub fn line_height(&self) -> i32 {
        self.line_height
    }

    pub fn currnet_bounds(&self) -> Size2D<i32, FontUnit> {
        Size2D::new(self.max_width, self.line_offset)
    }
}

impl Iterator for TextLayout<'_> {
    type Item = PositionedText;

    fn next(&mut self) -> Option<Self::Item> {
        let line = self.lines.next()?;

        let glyphs =
            self.font
                .shaper_mut()
                .map_glyphs(line, self.script_tag, self.match_presentation);

        let infos = self
            .font
            .shaper_mut()
            .shape(
                glyphs,
                self.script_tag,
                self.opt_lang_tag,
                &self.features,
                self.kerning,
            )
            .ok()?;

        let mut line_layout =
            GlyphLayout::new(self.font.shaper_mut(), &infos, self.direction, false);
        let glyph_poses = line_layout.glyph_positions().ok()?;

        let mut max_width = 0;

        let list = infos
            .into_iter()
            .zip(glyph_poses)
            .map(|(info, pos)| {
                let position =
                    Point2D::new(max_width + pos.x_offset, self.line_offset + pos.y_offset);
                let advances = Size2D::new(pos.hori_advance, pos.vert_advance);

                max_width = max_width.max(max_width + pos.x_offset + pos.hori_advance);

                PositionedGlyph {
                    glyph_id: info.glyph.glyph_index,
                    unicodes: info.glyph.unicodes,
                    position,
                    advances,
                    kerning: info.kerning,
                }
            })
            .collect();

        self.max_width = self.max_width.max(max_width);

        let bounds = Rect::new(
            Point2D::new(0, self.line_offset),
            Size2D::new(max_width, self.line_height),
        );

        self.line_offset += self.line_height;

        let positioned = PositionedText { bounds, list };

        Some(positioned)
    }
}

#[derive(Debug, Clone)]
pub struct PositionedText {
    pub bounds: Rect<i32, FontUnit>,
    pub list: Vec<PositionedGlyph>,
}

#[derive(Debug, Clone)]
pub struct PositionedGlyph {
    pub glyph_id: u16,
    pub unicodes: TinyVec<[char; 1]>,
    pub position: Point2D<i32, FontUnit>,
    pub advances: Size2D<i32, FontUnit>,
    pub kerning: i16,
}

#[cfg(test)]
#[test]
pub fn test_layout() {
    use allsorts::font::MatchingPresentation;
    use font_kit::source::SystemSource;

    use crate::font::DrawFont;

    let font = SystemSource::new()
        .select_by_postscript_name("ArialMT")
        .unwrap()
        .load()
        .unwrap();

    let mut draw_font = DrawFont::new(font);

    let mut layout = TextLayout::new(
        &mut draw_font,
        "Hello world!\nHello world!",
        TextDirection::LeftToRight,
        MatchingPresentation::NotRequired,
        0,
        None,
        None,
        true,
    );

    for line in &mut layout {
        println!("Bounds: {:?}", line.bounds);
        for info in line.list {
            println!("info: {:?}", info);
        }
    }

    println!("Bounding box: {:?}", layout.currnet_bounds())
}
