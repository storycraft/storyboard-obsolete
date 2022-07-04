pub use rustybuzz;
pub use ttf_parser;

pub mod cache;
pub mod component;
pub mod font;
pub mod layout;
pub mod rasterizer;

#[derive(Debug, Clone, Copy)]
pub struct FontUnit;

use std::{borrow::Cow, fmt::Debug, sync::Arc};

use layout::TextLayout;
use storyboard_core::{
    color::ShapeColor,
    euclid::{Box2D, Point2D, Rect, Vector2D, Transform3D},
    observable::Observable,
    unit::LogicalPixelUnit,
};
use storyboard_render::wgpu::{Device, Queue};
use storyboard_texture::render::{data::TextureData, RenderTexture2D};

use crate::{
    cache::GlyphCache,
    component::{GlyphRect, TextDrawable, TextRenderBatch},
    font::Font,
};

pub struct Text {
    pub position: Point2D<f32, LogicalPixelUnit>,
    pub size_px: u32,

    pub transform: Transform3D<f32, LogicalPixelUnit, LogicalPixelUnit>,

    text: Observable<Cow<'static, str>>,
    font: Observable<Font>,

    bounding_box: Box2D<f32, LogicalPixelUnit>,

    batches: Arc<Vec<TextRenderBatch>>,
}

impl Text {
    pub fn new(
        position: Point2D<f32, LogicalPixelUnit>,
        size_px: u32,
        transform: Transform3D<f32, LogicalPixelUnit, LogicalPixelUnit>,
        font: Font,
        text: Cow<'static, str>,
    ) -> Self {
        Self {
            position,
            size_px,
            transform,
            font: font.into(),
            text: text.into(),

            bounding_box: Box2D::zero(),

            batches: Arc::new(Vec::new()),
        }
    }

    pub fn font(&self) -> &Font {
        &self.font
    }

    pub fn set_font(&mut self, font: Font) {
        self.font = font.into();
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: Cow<'static, str>) {
        self.text = text.into();
    }

    pub const fn bounding_box(&self) -> Box2D<f32, LogicalPixelUnit> {
        self.bounding_box
    }

    pub fn update(
        &mut self,
        device: &Device,
        queue: &Queue,
        scale_factor: f32,
        textures: &TextureData,
        cache: &mut GlyphCache,
    ) {
        let font_invalidated = Observable::invalidate(&mut self.font);
        let text_invalidated = Observable::invalidate(&mut self.text);

        if font_invalidated || text_invalidated {
            self.bounding_box = Box2D::new(self.position, self.position);

            let scaled_size = (self.size_px as f32 * scale_factor).ceil() as u32;

            let layout = TextLayout::new(&self.font, &self.text);
            let mut layout_iter = layout.iter(8, self.size_px as f32);

            let mut batches = Vec::new();

            let ascender = layout_iter.ascender();

            while let Some(line_layout) = layout_iter.next() {
                let mut span_iter = line_layout.iter();
                let mut glyph_id_iter = line_layout.glyph_id_iter().peekable();

                while let Some(_) = glyph_id_iter.peek() {
                    if let Some(view_batch) =
                        cache.batch(device, queue, &self.font, &mut glyph_id_iter, scaled_size)
                    {
                        let texture = Arc::new(RenderTexture2D::init(
                            device,
                            view_batch.view.into(),
                            textures.bind_group_layout(),
                            textures.default_sampler(),
                        ));
                        let mut rects = Vec::new();

                        for (texture_rect, info) in view_batch.rects.iter().zip(&mut span_iter) {
                            let position = self.position
                                + info.position.cast_unit()
                                + Vector2D::new(
                                    0.0,
                                    ascender
                                        - texture_rect.tex_rect.size.height as f32 / scale_factor,
                                )
                                + (texture_rect.glyph_offset / scale_factor).cast_unit();

                            let size =
                                (texture_rect.tex_rect.size.cast() / scale_factor).cast_unit();

                            rects.push(GlyphRect {
                                rect: Rect::new(position, size),
                                texture_rect: texture.view().to_texture_rect(texture_rect.tex_rect),
                            });

                            self.bounding_box = Box2D::from_points(&[
                                self.bounding_box.min,
                                self.bounding_box.max,
                                position,
                                position + size,
                            ]);
                        }

                        batches.push(TextRenderBatch { texture, rects });
                    } else {
                        glyph_id_iter.next();
                        span_iter.next();
                    }
                }
            }

            self.batches = Arc::new(batches);
        }
    }

    pub fn draw(&mut self, color: &ShapeColor<4>) -> TextDrawable {
        TextDrawable {
            batches: self.batches.clone(),
            transform: self.transform,
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
            .field("batches", &self.batches)
            .finish_non_exhaustive()
    }
}
