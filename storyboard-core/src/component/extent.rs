/*
 * Created on Tue Sep 21 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::ops::Mul;

use euclid::{Vector2D, Vector3D};

#[derive(Debug, Clone, Copy, PartialEq)]
/// Represent absolute or relative value
pub enum ExtentUnit<T> {
    Fixed(T),
    Percent(T),
}

impl<T: Default> Default for ExtentUnit<T> {
    fn default() -> Self {
        Self::Fixed(Default::default())
    }
}

impl<T: Copy> ExtentUnit<T> {
    pub fn value(&self) -> T {
        match self {
            ExtentUnit::Fixed(value) => *value,
            ExtentUnit::Percent(value) => *value,
        }
    }
}

impl<T: Copy + Mul<Output = T>> ExtentUnit<T> {
    pub fn calc(&self, size: T) -> T {
        match self {
            ExtentUnit::Fixed(fixed) => *fixed,
            ExtentUnit::Percent(percent) => *percent * size,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExtentStandard {
    Root,
    Parent,
    Current,
}

impl Default for ExtentStandard {
    fn default() -> Self {
        Self::Current
    }
}

impl ExtentStandard {
    #[inline]
    pub fn select<T>(&self, root: T, parent: T, current: T) -> T {
        match self {
            ExtentStandard::Root => root,
            ExtentStandard::Parent => parent,
            ExtentStandard::Current => current,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
/// Value represented with relative or fixed unit
pub struct Extent<T> {
    pub standard: ExtentStandard,
    pub unit: ExtentUnit<T>,
}

impl<T> Extent<T> {
    pub const fn new(unit: ExtentUnit<T>, standard: ExtentStandard) -> Self {
        Self { standard, unit }
    }
}

impl<T> From<ExtentUnit<T>> for Extent<T> {
    fn from(unit: ExtentUnit<T>) -> Self {
        Self {
            standard: ExtentStandard::default(),
            unit,
        }
    }
}

impl<T: Copy + Mul<Output = T>> Extent<T> {
    pub fn calc(&self, root: T, parent: T, current: T) -> T {
        self.unit.calc(self.standard.select(root, parent, current))
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Extent2D<T> {
    pub x: Extent<T>,
    pub y: Extent<T>,
}

impl<T> From<(Extent<T>, Extent<T>)> for Extent2D<T> {
    fn from((x, y): (Extent<T>, Extent<T>)) -> Self {
        Self { x, y }
    }
}

impl<T: Copy + Mul<Output = T>> Extent2D<T> {
    pub fn calc<U>(
        &self,
        root: Vector2D<T, U>,
        parent: Vector2D<T, U>,
        current: Vector2D<T, U>,
    ) -> Vector2D<T, U> {
        Vector2D::new(
            self.x.calc(root.x, parent.x, current.x),
            self.y.calc(root.y, parent.y, current.y),
        )
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Extent3D<T> {
    pub x: Extent<T>,
    pub y: Extent<T>,
    pub z: Extent<T>,
}

impl<T> From<(Extent<T>, Extent<T>, Extent<T>)> for Extent3D<T> {
    fn from((x, y, z): (Extent<T>, Extent<T>, Extent<T>)) -> Self {
        Self { x, y, z }
    }
}

impl<T: Copy + Mul<Output = T>> Extent3D<T> {
    pub fn calc<U>(
        &self,
        root: Vector3D<T, U>,
        parent: Vector3D<T, U>,
        current: Vector3D<T, U>,
    ) -> Vector3D<T, U> {
        Vector3D::new(
            self.x.calc(root.x, parent.x, current.x),
            self.y.calc(root.y, parent.y, current.y),
            self.z.calc(root.z, parent.z, current.z),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{Extent, ExtentUnit, ExtentStandard};

    #[test]
    pub fn extent_test() {
        let extent = Extent::new(ExtentUnit::Percent(0.5), ExtentStandard::Parent);

        assert_eq!(extent.calc(1.0, 2.0, 3.0), 1.0)
    }
}
