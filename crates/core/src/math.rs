//! euclid math module extension

use std::ops::{Add, Div, Neg, Sub};

use euclid::{Point2D, Rect, Size2D};

pub trait RectExt<T, U> {
    fn into_coords(self) -> [Point2D<T, U>; 4];

    fn relative_in(&self, other: &Self) -> Self;
}

impl<T: Copy + Add<Output = T> + Sub<Output = T> + Neg<Output = T> + Div<Output = T>, U>
    RectExt<T, U> for Rect<T, U>
{
    fn into_coords(self) -> [Point2D<T, U>; 4] {
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

    fn relative_in(&self, other: &Self) -> Self {
        Rect::new(
            Point2D::new(
                (other.origin.x - self.origin.x) / self.size.width,
                (other.origin.y - self.origin.y) / self.size.height,
            ),
            Size2D::new(other.size.width / self.size.width, other.size.height / self.size.height),
        )
    }
}
