/*
 * Created on Wed Sep 15 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

pub mod color;
pub mod extent;
pub mod layout;
pub mod transform;

use std::fmt::Debug;

use euclid::{Point2D, Rect, Transform3D};

use crate::graphics::{PixelUnit, RenderUnit};

#[derive(Debug, Clone, Copy)]
pub struct DrawBox {
    pub screen: Rect<f32, PixelUnit>,

    pub rect: Rect<f32, PixelUnit>,

    pub matrix: Transform3D<f32, PixelUnit, RenderUnit>,
}

impl DrawBox {
    pub const fn into_space(self) -> DrawSpace {
        DrawSpace {
            screen: self.screen,
            parent: self.rect,
            matrix: self.matrix,
        }
    }

    pub fn get_quad_2d(&self, rect: &Rect<f32, PixelUnit>) -> [Point2D<f32, RenderUnit>; 4] {
        let min_x = rect.min_x();
        let min_y = rect.min_y();

        let max_x = rect.max_x();
        let max_y = rect.max_y();

        [
            self.matrix
                .transform_point2d((min_x, min_y).into())
                .unwrap(),
            self.matrix
                .transform_point2d((min_x, max_y).into())
                .unwrap(),
            self.matrix
                .transform_point2d((max_x, max_y).into())
                .unwrap(),
            self.matrix
                .transform_point2d((max_x, min_y).into())
                .unwrap(),
        ]
    }
}

impl Into<DrawSpace> for DrawBox {
    fn into(self) -> DrawSpace {
        self.into_space()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DrawSpace {
    pub screen: Rect<f32, PixelUnit>,
    pub parent: Rect<f32, PixelUnit>,
    pub matrix: Transform3D<f32, PixelUnit, RenderUnit>,
}

impl DrawSpace {
    pub fn new_screen(screen: Rect<f32, PixelUnit>) -> Self {
        let min = screen.min();
        let max = screen.max();

        let matrix = Transform3D::ortho(min.x, max.x, max.y, min.y, 0.0, 65535.0);
        Self {
            screen,
            parent: screen,
            matrix,
        }
    }

    pub fn inner_box(
        &self,
        rect: Rect<f32, PixelUnit>,
        transform: Option<&Transform3D<f32, PixelUnit, PixelUnit>>,
    ) -> DrawBox {
        let matrix = transform.map_or(self.matrix, |transform| transform.then(&self.matrix));

        DrawBox {
            screen: self.screen,
            rect,
            matrix,
        }
    }

    pub const fn into_box(self) -> DrawBox {
        DrawBox {
            screen: self.screen,
            rect: self.parent,
            matrix: self.matrix,
        }
    }
}
