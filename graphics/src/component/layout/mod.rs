/*
 * Created on Thu Nov 04 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use euclid::{Rect, Size2D, Transform3D};

use crate::{data::observable::Observable, unit::PixelUnit};

use super::{
    extent::{Extent2D, ExtentSize2D},
    transform::DrawTransform,
    DrawBox, DrawSpace,
};

#[derive(Debug)]
pub struct ComponentLayout {
    position: Observable<Extent2D>,
    anchor: Observable<Extent2D>,

    size: Observable<ExtentSize2D>,

    transform: Observable<DrawTransform>,

    space: DrawSpace,

    draw_rect: Rect<f32, PixelUnit>,
    draw_matrix: Transform3D<f32, PixelUnit, PixelUnit>,
}

impl ComponentLayout {
    pub fn new() -> Self {
        Self {
            position: Observable::default(),
            anchor: Observable::default(),

            size: Observable::default(),

            transform: Observable::default(),

            space: DrawSpace::new_screen(Rect::zero()),

            draw_rect: Rect::zero(),
            draw_matrix: Transform3D::identity(),
        }
    }

    #[inline]
    pub fn position(&self) -> &Extent2D {
        self.position.inner_ref()
    }

    #[inline]
    pub fn position_mut(&mut self) -> &mut Extent2D {
        self.position.inner_mut()
    }

    #[inline]
    pub fn set_position(&mut self, position: Extent2D) {
        self.position.set(position);
    }

    #[inline]
    pub fn anchor(&self) -> &Extent2D {
        self.anchor.inner_ref()
    }

    #[inline]
    pub fn anchor_mut(&mut self) -> &mut Extent2D {
        self.anchor.inner_mut()
    }

    #[inline]
    pub fn set_anchor(&mut self, anchor: Extent2D) {
        self.anchor.set(anchor);
    }

    #[inline]
    pub fn size(&self) -> &ExtentSize2D {
        self.size.inner_ref()
    }

    #[inline]
    pub fn size_mut(&mut self) -> &mut ExtentSize2D {
        self.size.inner_mut()
    }

    #[inline]
    pub fn set_size(&mut self, size: ExtentSize2D) {
        self.size.set(size);
    }

    #[inline]
    pub fn transform(&self) -> &DrawTransform {
        self.transform.inner_ref()
    }

    #[inline]
    pub fn transform_mut(&mut self) -> &mut DrawTransform {
        self.transform.inner_mut()
    }

    #[inline]
    pub fn set_transform(&mut self, transform: DrawTransform) {
        self.transform.set(transform);
    }

    pub fn update(&mut self, space: &DrawSpace) {
        let parent_changed = if &self.space != space {
            self.space = space.clone();
            true
        } else {
            false
        };

        let size_changed = self.size.unmark();

        if parent_changed || size_changed {
            let size = self
                .size
                .inner_ref()
                .calc(&space, &Size2D::zero())
                .to_vector()
                .to_size();
            self.draw_rect.size = size;
        }

        if parent_changed || size_changed || self.position.unmark() || self.anchor.unmark() {
            let origin = self.anchor.inner_ref().calc(
                &space,
                &Rect {
                    origin: self.position.inner_ref().calc(&space, &Rect::zero()),
                    size: -self.draw_rect.size,
                },
            );

            self.draw_rect.origin = origin;
        }

        if parent_changed || self.transform.unmark() {
            self.draw_matrix = self
                .transform
                .inner_ref()
                .calc_matrix(&space, &self.draw_rect);
        }
    }

    pub fn draw_rect(&self) -> &Rect<f32, PixelUnit> {
        &self.draw_rect
    }

    pub fn draw_matrix(&self) -> &Transform3D<f32, PixelUnit, PixelUnit> {
        &self.draw_matrix
    }

    pub fn get_draw_box(&self, space: &DrawSpace) -> DrawBox {
        space.inner_box(self.draw_rect, Some(&self.draw_matrix))
    }
}

#[cfg(test)]
#[test]
pub fn layout_test() {
    use euclid::{Point2D, Size2D};

    use crate::component::extent::{ExtentStandard, ExtentUnit};

    use super::DrawSpace;

    let screen = DrawSpace::new_screen(Rect::new(
        Point2D::new(540.0, 540.0),
        Size2D::new(1980.0, 1080.0),
    ));

    let mut layout = ComponentLayout::new();

    layout.set_position(Extent2D {
        standard: ExtentStandard::Parent,
        x: ExtentUnit::Percent(0.5),
        y: ExtentUnit::Percent(0.5),
    });

    layout.set_anchor(Extent2D {
        standard: ExtentStandard::Current,
        x: ExtentUnit::Percent(0.5),
        y: ExtentUnit::Percent(0.5),
    });

    layout.set_size(ExtentSize2D {
        standard: ExtentStandard::Parent,
        width: ExtentUnit::Percent(0.5),
        height: ExtentUnit::Percent(0.5),
    });

    layout.update(&screen);

    println!("rect: {:?}", layout.draw_rect());
}
