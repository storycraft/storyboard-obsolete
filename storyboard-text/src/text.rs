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
    tables::FontTableProvider,
};
use storyboard::{
    graphics::{
        texture::{RenderTexture2D, data::TextureData},
    },
};

use storyboard::core::{
    component::color::ShapeColor,
    euclid::{Point2D, Vector2D},
    observable::Observable,
    unit::PixelUnit,
    wgpu::{Device, Queue},
};
use ttf_parser::{Face, Tag};

use crate::{component::Glyph, cache::GlyphCache};

pub struct Text<'face> {
    pub position: Point2D<f32, PixelUnit>,
    pub size_px: u32,
    pub color: ShapeColor<4>,

    text: Observable<Cow<'static, str>>,
    shaper: Observable<allsorts::Font<Font<'face>>>,
    glyphs: Vec<(Vector2D<f32, PixelUnit>, Arc<RenderTexture2D>)>,
}

impl<'face> Text<'face> {
    pub fn new(
        position: Point2D<f32, PixelUnit>,
        size_px: u32,
        color: ShapeColor<4>,
        font: Face<'face>,
        text: Cow<'static, str>,
    ) -> Self {
        Self {
            position,
            size_px,
            color,
            shaper: allsorts::Font::new(Font(font)).unwrap().unwrap().into(),
            text: text.into(),

            glyphs: Vec::new(),
        }
    }

    pub fn font(&self) -> &Face<'face> {
        &self.shaper.font_table_provider.0
    }

    pub fn set_font(&mut self, font: Face<'face>) {
        self.shaper = allsorts::Font::new(Font(font)).unwrap().unwrap().into();
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
        textures: &TextureData,
        cache: &mut GlyphCache,
        submit: impl Fn(Glyph),
    ) {
        let font_invalidated = Observable::invalidate(&mut self.shaper);
        let text_invalidated = Observable::invalidate(&mut self.text);

        if font_invalidated || text_invalidated {
            let glyphs = self
                .shaper
                .map_glyphs(&self.text, 0, MatchingPresentation::NotRequired);

            if let Ok(infos) =
                self.shaper
                    .shape(glyphs, 0, None, &Features::Mask(FeatureMask::default()), true)
            {
                self.glyphs.clear();

                let scale =
                    self.size_px as f32 / self.shaper.font_table_provider.0.units_per_em() as f32;

                let mut layout =
                    GlyphLayout::new(&mut self.shaper, &infos, TextDirection::LeftToRight, false);

                let mut offset = Vector2D::<f32, PixelUnit>::new(0.0, 0.0);

                let positions = layout.glyph_positions().unwrap();

                for (info, position) in infos.iter().zip(&positions) {
                    if let Some((glyph_offset, view)) = cache.get_view(
                        device,
                        queue,
                        self.font(),
                        info.glyph.glyph_index,
                        self.size_px,
                    ) {
                        let y_offset = -(view.rect().size.height as f32);

                        let texture = RenderTexture2D::init(
                            device,
                            view,
                            textures.bind_group_layout(),
                            textures.default_sampler(),
                        );

                        self.glyphs.push((
                            Vector2D::new(
                                glyph_offset.x + (offset.x + position.x_offset as f32) * scale,
                                glyph_offset.y + y_offset + (offset.y + position.y_offset as f32) * scale,
                            ),
                            Arc::new(texture),
                        ));
                    }

                    offset += Vector2D::<f32, PixelUnit>::new(
                        position.hori_advance as f32,
                        position.vert_advance as f32,
                    );
                }
            }
        }

        for (offset, texture) in &self.glyphs {
            submit(Glyph {
                position: self.position + *offset,
                color: self.color.clone(),
                texture: texture.clone(),
            });
        }
    }
}

impl Debug for Text<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Text")
            .field("position", &self.position)
            .field("size_px", &self.size_px)
            .field("color", &self.color)
            .field("text", &self.text)
            .field("glyphs", &self.glyphs)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
struct Font<'face>(Face<'face>);

impl FontTableProvider for Font<'_> {
    fn table_data<'a>(
        &'a self,
        tag: u32,
    ) -> Result<Option<Cow<'a, [u8]>>, allsorts::error::ParseError> {
        Ok(self.0.table_data(Tag(tag)).map(|data| Cow::Borrowed(data)))
    }

    fn has_table<'a>(&'a self, tag: u32) -> bool {
        self.0.table_data(Tag(tag)).is_some()
    }
}
