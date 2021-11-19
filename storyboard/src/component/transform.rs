/*
 * Created on Thu Sep 23 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use euclid::{Angle, Rect, Rotation3D, Transform3D};

use crate::graphics::PixelUnit;

use super::{
    extent::{Extent2D, ExtentUnit},
    DrawSpace,
};

const TWO_PI: f32 = 2.0 * std::f32::consts::PI;

#[derive(Debug, Clone)]
pub struct DrawTransform {
    pub origin: Extent2D,
    pub translation: TransformUnit2D,
    pub rotation: TransformUnit3D,
    pub scale: TransformUnit2D,
    pub skew: TransformUnit2D,
}

impl DrawTransform {
    pub fn calc_matrix(
        &self,
        space: &DrawSpace,
        rect: &Rect<f32, PixelUnit>,
    ) -> Transform3D<f32, PixelUnit, PixelUnit> {
        let origin = self.origin.calc(space, &rect);

        let origin_transform: Transform3D<f32, PixelUnit, PixelUnit> = Transform3D::translation(
            -origin.x + self.translation.x.calc(rect.size.width),
            -origin.y + self.translation.y.calc(rect.size.height),
            0.0,
        );
        let origin_restore_transform: Transform3D<f32, PixelUnit, PixelUnit> =
            Transform3D::translation(origin.x, origin.y, 0.0);

        let rotation: Transform3D<f32, PixelUnit, PixelUnit> = {
            let rotation_x: Rotation3D<f32, PixelUnit, PixelUnit> =
                Rotation3D::around_x(Angle::radians(self.rotation.x.calc(TWO_PI)));
            let rotation_y: Rotation3D<f32, PixelUnit, PixelUnit> =
                Rotation3D::around_y(Angle::radians(self.rotation.y.calc(TWO_PI)));
            let rotation_z: Rotation3D<f32, PixelUnit, PixelUnit> =
                Rotation3D::around_z(Angle::radians(self.rotation.z.calc(TWO_PI)));

            rotation_x
                .then(&rotation_y)
                .then(&rotation_z)
                .to_transform()
        };

        let scale: Transform3D<f32, PixelUnit, PixelUnit> = Transform3D::scale(
            self.scale.x.calc(rect.size.width) / rect.size.width,
            self.scale.y.calc(rect.size.height) / rect.size.height,
            1.0,
        );
        let skew: Transform3D<f32, PixelUnit, PixelUnit> = Transform3D::skew(
            Angle::radians(self.skew.x.calc(TWO_PI)),
            Angle::radians(self.skew.y.calc(TWO_PI)),
        );

        origin_transform
            .then(&rotation)
            .then(&scale)
            .then(&skew)
            .then(&origin_restore_transform)
    }
}

impl Default for DrawTransform {
    fn default() -> Self {
        Self {
            origin: Default::default(),
            translation: TransformUnit2D::default(),
            rotation: TransformUnit3D::default(),
            scale: TransformUnit2D {
                x: ExtentUnit::Percent(1.0),
                y: ExtentUnit::Percent(1.0),
            },
            skew: TransformUnit2D::default(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TransformUnit3D {
    pub x: ExtentUnit,
    pub y: ExtentUnit,
    pub z: ExtentUnit,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TransformUnit2D {
    pub x: ExtentUnit,
    pub y: ExtentUnit,
}
