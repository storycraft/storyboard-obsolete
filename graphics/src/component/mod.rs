/*
 * Created on Wed Sep 15 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

pub mod color;
pub mod extent;
pub mod texture;
pub mod transform;
pub mod layout;

use std::fmt::Debug;

use euclid::{Point2D, Rect, Transform3D};
use wgpu::CommandEncoder;

use crate::{renderer::RenderStateQueue, unit::{PixelUnit, WgpuUnit}};

use super::{
    context::{DrawContext, RenderContext},
    pass::StoryboardRenderPass,
};

pub trait DrawState<'a> {
    fn prepare(
        &mut self,
        context: &mut DrawContext,
        depth: f32,
        encoder: &mut CommandEncoder,
        state_queue: &mut RenderStateQueue<'a>
    );
}

pub trait RenderState {
    fn render<'r>(&'r mut self, context: &'r RenderContext, pass: &mut StoryboardRenderPass<'r>);
}

pub struct Drawable<T> {
    pub opaque: bool,
    pub state: T,
}

#[derive(Debug, Clone, Copy)]
pub struct DrawBox {
    pub screen: Rect<f32, PixelUnit>,

    pub rect: Rect<f32, PixelUnit>,

    pub matrix: Transform3D<f32, PixelUnit, WgpuUnit>,
}

impl DrawBox {
    pub const fn into_space(self) -> DrawSpace {
        DrawSpace {
            screen: self.screen,
            parent: self.rect,
            matrix: self.matrix,
        }
    }

    pub fn get_quad_2d(&self, rect: &Rect<f32, PixelUnit>) -> [Point2D<f32, WgpuUnit>; 4] {
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
    pub matrix: Transform3D<f32, PixelUnit, WgpuUnit>,
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

    pub fn inner_box_isolated(
        &self,
        rect: Rect<f32, PixelUnit>,
        transform: Option<Transform3D<f32, PixelUnit, WgpuUnit>>,
    ) -> DrawBox {
        let matrix = transform.unwrap_or(Transform3D::identity());

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