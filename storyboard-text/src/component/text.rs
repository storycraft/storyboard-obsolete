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
    unit::LogicalPixelUnit,
    wgpu::{Device, Queue},
};

use crate::{
    cache::GlyphCache,
    component::{GlyphRect, TextDrawable},
    font::Font,
};

use super::TextRenderBatch;

pub struct Text {
    pub position: Point2D<f32, LogicalPixelUnit>,
    pub size_px: u32,

    text: Observable<Cow<'static, str>>,
    shaper: Observable<allsorts::Font<Font>>,
    batches: Arc<Vec<TextRenderBatch>>,
}

impl Text {
    pub fn new(
        position: Point2D<f32, LogicalPixelUnit>,
        size_px: u32,
        font: Font,
        text: Cow<'static, str>,
    ) -> Self {
        Self {
            position,
            size_px,
            shaper: allsorts::Font::new(font).unwrap().unwrap().into(),
            text: text.into(),

            batches: Arc::new(Vec::new()),
        }
    }

    pub fn font(&self) -> &Font {
        &self.shaper.font_table_provider
    }

    pub fn set_font(&mut self, font: Font) {
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
        color: &ShapeColor<4>,
        scale_factor: f32,
        textures: &TextureData,
        cache: &mut GlyphCache
    ) -> TextDrawable {
        let font_invalidated = Observable::invalidate(&mut self.shaper);
        let text_invalidated = Observable::invalidate(&mut self.text);

        if font_invalidated || text_invalidated {
            let glyphs = self
                .shaper
                .map_glyphs(&self.text, 0, MatchingPresentation::NotRequired);

            let scaled_size = (self.size_px as f32 * scale_factor).ceil() as u32;
            let view_batches = cache.batch_glyphs(
                device,
                queue,
                &self.shaper.font_table_provider,
                glyphs.iter().map(|glyph| glyph.glyph_index),
                scaled_size,
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
                    self.size_px as f32 / self.shaper.font_table_provider.units_per_em() as f32;

                let mut layout =
                    GlyphLayout::new(&mut self.shaper, &infos, TextDirection::LeftToRight, false);

                let positions = layout.glyph_positions().unwrap();

                let mut positions_iter = positions.iter();

                let mut offset = Vector2D::<f32, LogicalPixelUnit>::new(0.0, 0.0);
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
                                        self.shaper.font_table_provider.ascender() as f32 * scale
                                            - texture_rect.rasterized_size.height / scale_factor,
                                    )
                                    + offset
                                    + (texture_rect.glyph_offset / scale_factor).cast_unit(),
                                (texture_rect.rasterized_size / scale_factor).cast_unit(),
                            ),
                            texture_rect: texture_rect.tex_rect,
                        });

                        offset += Vector2D::<f32, LogicalPixelUnit>::new(
                            pos.hori_advance as f32 * scale,
                            pos.vert_advance as f32 * scale,
                        );
                    }

                    batches.push(TextRenderBatch { texture, rects });
                }
            }

            self.batches = Arc::new(batches);
        }

        TextDrawable {
            batches: self.batches.clone(),
            color: color.clone(),
        }
    }
}

impl Debug for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Text")
            .field("position", &self.position)
            .field("size_px", &self.size_px)
            .field("text", &self.text)
            .field("glyphs", &self.batches)
            .finish_non_exhaustive()
    }
}
