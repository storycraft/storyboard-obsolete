/*
 * Created on Mon Sep 20 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{marker::PhantomData, sync::Arc};

use euclid::{Point2D, Rect, Size2D, Transform3D};

use crate::{component::{DrawSpace, extent::{Extent2D, ExtentStandard, ExtentUnit}}, graphics::{PixelUnit, RenderUnit, texture::Texture2D, TextureUnit}};

#[derive(Debug, Clone)]
pub struct ComponentTexture {
    pub texture: Arc<Texture2D>,
    pub layout: TextureLayout
}

#[derive(Debug, Clone, Copy)]
pub enum TextureLayout {
    None,
    Fill,
    Centered,
    Stretch,
    FitX,
    FitY,
    FitAuto,
    Custom(TextureCustomLayout),
}

#[derive(Debug, Clone, Copy)]
pub struct TextureCustomLayout {
    pub position: Extent2D,
    pub size: Extent2D,
    pub anchor: Extent2D,
}

impl Default for TextureCustomLayout {
    fn default() -> Self {
        Self {
            position: Default::default(),
            size: Extent2D {
                standard: ExtentStandard::Current,
                x: ExtentUnit::Percent(1.0),
                y: ExtentUnit::Percent(1.0),
            },
            anchor: Default::default(),
        }
    }
}

pub type QuadTextureCoord = [Point2D<f32, TextureUnit>; 4];

impl TextureLayout {
    pub const STRETCHED: QuadTextureCoord = [
        Point2D {
            x: 0.0,
            y: 0.0,
            _unit: PhantomData,
        },
        Point2D {
            x: 0.0,
            y: 1.0,
            _unit: PhantomData,
        },
        Point2D {
            x: 1.0,
            y: 1.0,
            _unit: PhantomData,
        },
        Point2D {
            x: 1.0,
            y: 0.0,
            _unit: PhantomData,
        },
    ];

    fn none(&self, space: &DrawSpace, texture_size: &Size2D<u32, PixelUnit>) -> QuadTextureCoord {
        let width = space.parent.size.width / texture_size.width as f32;
        let height = space.parent.size.height / texture_size.height as f32;

        [
            Point2D::zero(),
            (0.0, height).into(),
            (width, height).into(),
            (width, 0.0).into(),
        ]
    }

    fn centered(
        &self,
        space: &DrawSpace,
        texture_size: &Size2D<u32, PixelUnit>,
    ) -> QuadTextureCoord {
        let width = space.parent.size.width / texture_size.width as f32;
        let height = space.parent.size.height / texture_size.height as f32;

        let start_x = (1.0 - width) * 0.5;
        let start_y = (1.0 - height) * 0.5;

        let end_x = (1.0 + width) * 0.5;
        let end_y = (1.0 + height) * 0.5;

        [
            (start_x, start_y).into(),
            (start_x, end_y).into(),
            (end_x, end_y).into(),
            (end_x, start_y).into(),
        ]
    }

    fn fit_x(&self, space: &DrawSpace) -> QuadTextureCoord {
        let height = space.parent.size.height as f32 / space.parent.size.width as f32;

        let start_y = (1.0 - height) * 0.5;
        let end_y = (1.0 + height) * 0.5;

        [
            (0.0, start_y).into(),
            (0.0, end_y).into(),
            (1.0, end_y).into(),
            (1.0, start_y).into(),
        ]
    }

    fn fit_y(&self, space: &DrawSpace) -> QuadTextureCoord {
        let width = space.parent.size.width as f32 / space.parent.size.height as f32;

        let start_x = (1.0 - width) * 0.5;
        let end_x = (1.0 + width) * 0.5;

        [
            (start_x, 0.0).into(),
            (start_x, 1.0).into(),
            (end_x, 1.0).into(),
            (end_x, 0.0).into(),
        ]
    }

    fn custom(
        &self,
        layout: &TextureCustomLayout,
        space: &DrawSpace,
        texture_size: &Size2D<u32, PixelUnit>,
    ) -> QuadTextureCoord {
        let texture_size = texture_size.cast();

        let texture_rect = {
            let rect = Rect {
                origin: space.parent.origin,
                size: texture_size,
            };

            let pos = layout.position.calc(space, &rect);

            let size = layout.size.calc(space, &rect);
            let size = (size.x, size.y).into();

            let anchor = layout.anchor.calc(
                space,
                &Rect {
                    origin: Point2D::zero(),
                    size,
                },
            );

            Rect {
                origin: (pos - anchor - space.parent.origin.to_vector()).to_point(),
                size,
            }
        };

        let matrix: Transform3D<f32, PixelUnit, RenderUnit> = Transform3D::ortho(
            texture_rect.origin.x,
            texture_rect.max_x(),
            texture_rect.origin.y,
            texture_rect.max_y(),
            1.0,
            0.0,
        );

        // TODO
        let start = matrix.transform_point2d(Point2D::zero()).unwrap();
        let end = matrix
            .transform_point2d(space.parent.origin + space.parent.size)
            .unwrap();

        let start_x = (start.x + 1.0) * 0.5;
        let start_y = (start.y + 1.0) * 0.5;

        let end_x = (end.x + 1.0) * 0.5;
        let end_y = (end.y + 1.0) * 0.5;

        [
            (start_x, start_y).into(),
            (start_x, end_y).into(),
            (end_x, end_y).into(),
            (end_x, start_y).into(),
        ]
    }

    pub fn texture_coord_quad(
        &self,
        space: &DrawSpace,
        texture_size: &Size2D<u32, PixelUnit>,
    ) -> QuadTextureCoord {
        match self {
            TextureLayout::None => self.none(space, texture_size),
            TextureLayout::Fill => {
                if space.parent.size.height <= space.parent.size.width {
                    self.fit_x(space)
                } else {
                    self.fit_y(space)
                }
            }
            TextureLayout::Centered => self.centered(space, texture_size),
            TextureLayout::Stretch => Self::STRETCHED,
            TextureLayout::FitX => self.fit_x(space),
            TextureLayout::FitY => self.fit_y(space),
            TextureLayout::FitAuto => {
                if space.parent.size.height >= space.parent.size.width {
                    self.fit_x(space)
                } else {
                    self.fit_y(space)
                }
            }
            TextureLayout::Custom(layout) => self.custom(layout, space, texture_size),
        }
    }
}

impl Default for TextureLayout {
    fn default() -> Self {
        Self::None
    }
}
