/*
 * Created on Tue Jun 07 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::sync::Arc;
use storyboard_core::{
    euclid::{Point2D, Rect, Size2D},
    unit::{PixelUnit, TextureUnit},
};

use crate::graphics::texture::RenderTexture2D;

#[derive(Debug, Clone)]
pub struct ComponentTexture {
    pub inner: Arc<RenderTexture2D>,
    pub layout: TextureLayout,
    pub wrapping_mode: (TextureWrap, TextureWrap),
}

impl ComponentTexture {
    pub const fn new(inner: Arc<RenderTexture2D>, layout: TextureLayout, wrapping_mode: (TextureWrap, TextureWrap)) -> Self {
        Self { inner, layout, wrapping_mode }
    }

    pub fn get_texture_bounds(
        &self,
        rect: Rect<f32, PixelUnit>,
        screen_size: Size2D<f32, PixelUnit>,
    ) -> Rect<f32, PixelUnit> {
        self.layout.get_bounds(rect, screen_size, self.inner.view().size().cast())
    }

    pub fn option_get_texture_bounds(this: Option<&Self>, rect: Rect<f32, PixelUnit>, screen_size: Size2D<f32, PixelUnit>) -> Rect<f32, PixelUnit> {
        match this {
            Some(this) => this.get_texture_bounds(rect, screen_size),
            None => Rect::new(Point2D::new(0.0, 0.0), Size2D::new(1.0, 1.0)),
        }
    }

    pub fn option_view_texture_rect(this: Option<&Self>) -> Rect<f32, TextureUnit> {
        match this {
            Some(this) => this.inner.view().texture_rect(),
            None => Default::default(),
        }
    }

    pub fn option_wrapping_mode(this: Option<&Self>) -> (TextureWrap, TextureWrap) {
        match this {
            Some(this) => this.wrapping_mode,
            None => Default::default(),
        }
    }
}

impl AsRef<Arc<RenderTexture2D>> for ComponentTexture {
    fn as_ref(&self) -> &Arc<RenderTexture2D> {
        &self.inner
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TextureLayout {
    Absolute(TextureLayoutStyle),
    Relative(TextureLayoutStyle),
}

impl TextureLayout {
    pub fn get_bounds(
        &self,
        rect: Rect<f32, PixelUnit>,
        screen_size: Size2D<f32, PixelUnit>,
        texture_size: Size2D<f32, PixelUnit>,
    ) -> Rect<f32, PixelUnit> {
        match self {
            TextureLayout::Absolute(style) => style.get_coord_rect(screen_size, texture_size),
            TextureLayout::Relative(style) => style
                .get_coord_rect(rect.size, texture_size)
                .translate(rect.origin.to_vector()),
        }
    }
}

impl Default for TextureLayout {
    fn default() -> Self {
        Self::Relative(Default::default())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextureLayoutStyle {
    None,
    Fill,
    Centered,
    Stretched,
    FitWidth,
    FitHeight,
    Fit,
    Custom(Rect<f32, PixelUnit>),
}

impl TextureLayoutStyle {
    fn none(texture_size: Size2D<f32, PixelUnit>) -> Rect<f32, PixelUnit> {
        Rect::new(Point2D::zero(), texture_size)
    }

    fn centered(
        rect_size: Size2D<f32, PixelUnit>,
        texture_size: Size2D<f32, PixelUnit>,
    ) -> Rect<f32, PixelUnit> {
        Rect::new(
            ((rect_size - texture_size) / 2.0).to_vector().to_point(),
            texture_size,
        )
    }

    fn fit_width(
        rect_size: Size2D<f32, PixelUnit>,
        texture_size: Size2D<f32, PixelUnit>,
    ) -> Rect<f32, PixelUnit> {
        let scale = rect_size.width / texture_size.width;

        let size = texture_size * scale;

        Rect::new(
            Point2D::new(0.0, (rect_size.height - size.height) / 2.0),
            size,
        )
    }

    fn fit_height(
        rect_size: Size2D<f32, PixelUnit>,
        texture_size: Size2D<f32, PixelUnit>,
    ) -> Rect<f32, PixelUnit> {
        let scale = rect_size.height / texture_size.height;

        let size = texture_size * scale;

        Rect::new(
            Point2D::new((rect_size.width - size.width) / 2.0, 0.0),
            size,
        )
    }

    pub fn get_coord_rect(
        &self,
        rect_size: Size2D<f32, PixelUnit>,
        texture_size: Size2D<f32, PixelUnit>,
    ) -> Rect<f32, PixelUnit> {
        match self {
            TextureLayoutStyle::None => Self::none(texture_size),
            TextureLayoutStyle::Fill => {
                if rect_size.width >= texture_size.width {
                    Self::fit_width(rect_size, texture_size)
                } else {
                    Self::fit_height(rect_size, texture_size)
                }
            }
            TextureLayoutStyle::Centered => Self::centered(rect_size, texture_size),
            TextureLayoutStyle::Stretched => Rect::new(Point2D::zero(), rect_size),
            TextureLayoutStyle::FitWidth => Self::fit_width(rect_size, texture_size),
            TextureLayoutStyle::FitHeight => Self::fit_height(rect_size, texture_size),
            TextureLayoutStyle::Fit => {
                if rect_size.width <= texture_size.width {
                    Self::fit_width(rect_size, texture_size)
                } else {
                    Self::fit_height(rect_size, texture_size)
                }
            }
            TextureLayoutStyle::Custom(rect) => *rect,
        }
    }
}

impl Default for TextureLayoutStyle {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum TextureWrap {
    None = 0,
    Clamp = 1,
    Repeat = 2
}

impl Default for TextureWrap {
    fn default() -> Self {
        Self::None
    }
}