/*
 * Created on Sat Jun 11 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{borrow::Cow, fmt::Debug, sync::Arc};

use allsorts::{
    font::MatchingPresentation,
    glyph_position::{GlyphLayout, TextDirection},
    gsub::{FeatureMask, Features},
};
use storyboard::{
    core::euclid::Rect,
    graphics::texture::{data::TextureData, RenderTexture2D},
};

use storyboard::core::{
    component::color::ShapeColor,
    euclid::{Point2D, Vector2D},
    observable::Observable,
    unit::PixelUnit,
    wgpu::{Device, Queue},
};

use crate::{
    cache::GlyphCache,
    component::{GlyphRect, TextDrawable},
    font::Font,
};

use super::TextRenderBatch;

pub struct Text<'face> {
    pub position: Point2D<f32, PixelUnit>,
    pub size_px: u32,
    pub color: ShapeColor<4>,

    text: Observable<Cow<'static, str>>,
    shaper: Observable<allsorts::Font<Font<'face>>>,
    batches: Arc<Vec<TextRenderBatch>>,
}

impl<'face> Text<'face> {
    pub fn new(
        position: Point2D<f32, PixelUnit>,
        size_px: u32,
        color: ShapeColor<4>,
        font: Font<'face>,
        text: Cow<'static, str>,
    ) -> Self {
        Self {
            position,
            size_px,
            color,
            shaper: allsorts::Font::new(font).unwrap().unwrap().into(),
            text: text.into(),

            batches: Arc::new(Vec::new()),
        }
    }

    pub fn font(&self) -> &Font<'face> {
        &self.shaper.font_table_provider
    }

    pub fn set_font(&mut self, font: Font<'face>) {
        self.shaper = allsorts::Font::new(font).unwrap().unwrap().into();
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: Cow<'static, str>) {
        self.text = text.into();
    }

    pub fn draw(
        &mut self,
        device: &Device,
        queue: &Queue,
        scale_factor: f32,
        textures: &TextureData,
        cache: &mut GlyphCache,
        mut submit: impl FnMut(TextDrawable),
    ) {
        let font_invalidated = Observable::invalidate(&mut self.shaper);
        let text_invalidated = Observable::invalidate(&mut self.text);

        let size_px = (self.size_px as f32 * scale_factor).ceil() as u32;

        if font_invalidated || text_invalidated {
            let glyphs = self
                .shaper
                .map_glyphs(&self.text, 0, MatchingPresentation::NotRequired);

            let view_batches = cache.get_batch(
                device,
                queue,
                &self.shaper.font_table_provider,
                glyphs.iter().map(|glyph| glyph.glyph_index),
                size_px,
            );

            let mut batches = Vec::new();

            if let Ok(infos) = self.shaper.shape(
                glyphs,
                0,
                None,
                &Features::Mask(FeatureMask::default()),
                true,
            ) {
                let scale =
                size_px as f32 / self.shaper.font_table_provider.units_per_em() as f32;

                let mut layout =
                    GlyphLayout::new(&mut self.shaper, &infos, TextDirection::LeftToRight, false);

                let positions = layout.glyph_positions().unwrap();

                let mut positions_iter = positions.iter();

                let mut offset = Vector2D::<f32, PixelUnit>::new(0.0, 0.0);
                for view_batch in view_batches {
                    let texture = Arc::new(RenderTexture2D::init(
                        device,
                        view_batch.view.into(),
                        textures.bind_group_layout(),
                        textures.default_sampler(),
                    ));
                    let mut rects = Vec::new();

                    for (texture_rect, pos) in view_batch.rects.iter().zip(&mut positions_iter) {
                        rects.push(GlyphRect {
                            rect: Rect::new(
                                self.position
                                    + Vector2D::new(
                                        0.0,
                                        self.shaper.font_table_provider.ascender() as f32 * scale - texture_rect.rasterized_size.height,
                                    )
                                    + offset
                                    + texture_rect.glyph_offset,
                                texture_rect.rasterized_size,
                            ),
                            texture_rect: texture_rect.tex_rect,
                        });

                        offset += Vector2D::<f32, PixelUnit>::new(
                            pos.hori_advance as f32 * scale,
                            pos.vert_advance as f32 * scale,
                        );
                    }

                    batches.push(TextRenderBatch { texture, rects });
                }
            }

            self.batches = Arc::new(batches);
        }

        submit(TextDrawable {
            batches: self.batches.clone(),
            color: self.color.clone(),
        });
    }
}

impl Debug for Text<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Text")
            .field("position", &self.position)
            .field("size_px", &self.size_px)
            .field("color", &self.color)
            .field("text", &self.text)
            .field("glyphs", &self.batches)
            .finish_non_exhaustive()
    }
}
