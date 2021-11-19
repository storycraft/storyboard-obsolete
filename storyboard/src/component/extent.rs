/*
 * Created on Tue Sep 21 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use euclid::{Point2D, Rect, Size2D};

use crate::graphics::PixelUnit;

use super::DrawSpace;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExtentUnit {
    Fixed(f32),
    Percent(f32),
}

impl Default for ExtentUnit {
    fn default() -> Self {
        Self::Fixed(f32::default())
    }
}

impl ExtentUnit {
    pub fn value(&self) -> f32 {
        match self {
            ExtentUnit::Fixed(value) => *value,
            ExtentUnit::Percent(value) => *value,
        }
    }

    pub fn calc(&self, size: f32) -> f32 {
        match self {
            ExtentUnit::Fixed(fixed) => *fixed,
            ExtentUnit::Percent(percent) => percent * size,
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

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Extent2D {
    pub standard: ExtentStandard,
    pub x: ExtentUnit,
    pub y: ExtentUnit,
}

impl From<(ExtentUnit, ExtentUnit)> for Extent2D {
    fn from(units: (ExtentUnit, ExtentUnit)) -> Self {
        Self {
            standard: ExtentStandard::default(),
            x: units.0,
            y: units.1,
        }
    }
}

impl Extent2D {
    pub fn calc(
        &self,
        space: &DrawSpace,
        current: &Rect<f32, PixelUnit>,
    ) -> Point2D<f32, PixelUnit> {
        match self.standard {
            ExtentStandard::Root => Point2D::new(
                space.screen.origin.x + self.x.calc(space.screen.size.width),
                space.screen.origin.y + self.y.calc(space.screen.size.height),
            ),

            ExtentStandard::Parent => Point2D::new(
                space.parent.origin.x + self.x.calc(space.parent.size.width),
                space.parent.origin.y + self.y.calc(space.parent.size.height),
            ),

            ExtentStandard::Current => Point2D::new(
                current.origin.x + self.x.calc(current.size.width),
                current.origin.y + self.y.calc(current.size.height),
            ),
        }
    }
}

impl From<ExtentSize2D> for Extent2D {
    fn from(extent: ExtentSize2D) -> Self {
        Self {
            standard: extent.standard,
            x: extent.width,
            y: extent.height,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ExtentSize2D {
    pub standard: ExtentStandard,
    pub width: ExtentUnit,
    pub height: ExtentUnit,
}

impl From<(ExtentUnit, ExtentUnit)> for ExtentSize2D {
    fn from(units: (ExtentUnit, ExtentUnit)) -> Self {
        Self {
            standard: ExtentStandard::default(),
            width: units.0,
            height: units.1,
        }
    }
}

impl ExtentSize2D {
    pub fn calc(&self, space: &DrawSpace, size: &Size2D<f32, PixelUnit>) -> Size2D<f32, PixelUnit> {
        match self.standard {
            ExtentStandard::Root => Size2D::new(
                self.width.calc(space.screen.size.width),
                self.height.calc(space.screen.size.height),
            ),

            ExtentStandard::Parent => Size2D::new(
                self.width.calc(space.parent.size.width),
                self.height.calc(space.parent.size.height),
            ),

            ExtentStandard::Current => {
                Size2D::new(self.width.calc(size.width), self.height.calc(size.height))
            }
        }
    }
}

impl From<Extent2D> for ExtentSize2D {
    fn from(extent: Extent2D) -> Self {
        Self {
            standard: extent.standard,
            width: extent.x,
            height: extent.y,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Extent {
    pub standard: ExtentStandard,
    pub unit: ExtentUnit,
}

impl From<ExtentUnit> for Extent {
    fn from(unit: ExtentUnit) -> Self {
        Self {
            standard: ExtentStandard::default(),
            unit,
        }
    }
}

impl Extent {
    pub fn calc(&self, root: f32, parent: f32, current: f32) -> f32 {
        match self.standard {
            ExtentStandard::Root => root + self.unit.calc(root),
            ExtentStandard::Parent => parent + self.unit.calc(parent),
            ExtentStandard::Current => current + self.unit.calc(current),
        }
    }
}
