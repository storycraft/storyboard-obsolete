/*
 * Created on Mon Jun 06 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

//! euclid math module extension

use std::ops::{Add, Div, Neg, Sub};

use storyboard_core::euclid::{Point2D, Rect};

pub trait RectExt<T, U> {
    fn to_coords(self) -> [Point2D<T, U>; 4];

    fn relative_to(&self, other: &Self) -> Self;
}

impl<T: Copy + Add<Output = T> + Sub<Output = T> + Neg<Output = T> + Div<Output = T>, U>
    RectExt<T, U> for Rect<T, U>
{
    fn to_coords(self) -> [Point2D<T, U>; 4] {
        [
            self.origin,
            Point2D::new(self.origin.x, self.origin.y + self.size.height),
            Point2D::new(
                self.origin.x + self.size.width,
                self.origin.y + self.size.height,
            ),
            Point2D::new(self.origin.x + self.size.width, self.origin.y),
        ]
    }

    fn relative_to(&self, other: &Self) -> Self {
        let scale = other
            .size
            .to_vector()
            .component_div(self.size.to_vector())
            .cast_unit();

        Rect::new(
            Point2D::new(
                (self.origin.x - self.size.width) / self.size.width,
                -self.origin.y / self.size.height,
            ),
            scale.to_size(),
        )
    }
}
