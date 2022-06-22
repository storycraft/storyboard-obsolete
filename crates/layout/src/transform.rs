use storyboard_core::euclid::{
    vec2, vec3, Angle, Rect, Rotation3D, Transform3D, Vector2D, Vector3D,
};

use crate::extent::{Extent, Extent2D, Extent3D, ExtentStandard, ExtentUnit};

#[derive(Debug, Clone)]
pub struct TransformRect<T, U> {
    pub rect: Rect<T, U>,
    pub transform: DrawTransform<U>,
}

const TWO_PI: f32 = 2.0 * std::f32::consts::PI;

#[derive(Debug, Clone)]
pub struct DrawTransformExtent {
    pub origin: Extent2D<f32>,
    pub translation: Extent2D<f32>,
    pub rotation: Extent3D<f32>,
    pub scale: Extent2D<f32>,
    pub skew: Extent2D<f32>,
}

impl DrawTransformExtent {
    pub fn calc<U>(
        &self,
        root: &TransformRect<f32, U>,
        parent: &TransformRect<f32, U>,
        current: Rect<f32, U>,
    ) -> DrawTransform<U> {
        let origin = self.origin.calc(
            root.transform.origin,
            parent.transform.origin,
            current.size.to_vector(),
        );

        let translation = self.translation.calc(
            root.rect.size.to_vector(),
            parent.rect.size.to_vector(),
            current.size.to_vector(),
        );

        let rotation = self.rotation.calc(
            root.transform.rotation,
            parent.transform.rotation,
            vec3(TWO_PI, TWO_PI, TWO_PI),
        );

        let scale = self
            .scale
            .calc(
                root.rect.size.to_vector(),
                parent.rect.size.to_vector(),
                current.size.to_vector(),
            )
            .component_div(current.size.to_vector());

        let skew = self.skew.calc(
            root.transform.skew,
            parent.transform.skew,
            vec2(TWO_PI, TWO_PI),
        );

        DrawTransform {
            origin,
            translation,
            rotation,
            scale,
            skew,
        }
    }
}

impl Default for DrawTransformExtent {
    fn default() -> Self {
        Self {
            origin: Default::default(),
            translation: Default::default(),
            rotation: Default::default(),
            scale: Extent2D {
                x: Extent {
                    unit: ExtentUnit::Percent(1.0),
                    standard: ExtentStandard::Current,
                },
                y: Extent {
                    unit: ExtentUnit::Percent(1.0),
                    standard: ExtentStandard::Current,
                },
            },
            skew: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DrawTransform<U> {
    pub origin: Vector2D<f32, U>,
    pub translation: Vector2D<f32, U>,
    pub rotation: Vector3D<f32, U>,
    pub scale: Vector2D<f32, U>,
    pub skew: Vector2D<f32, U>,
}

impl<U> DrawTransform<U> {
    pub fn into_matrix(&self) -> Transform3D<f32, U, U> {
        Transform3D::identity()
            .then_translate((-self.origin + self.translation).to_3d())
            .then_scale(self.scale.x, self.scale.y, 1.0)
            .then(
                &Rotation3D::<f32, U, U>::around_x(Angle::radians(self.rotation.x))
                    .then(&Rotation3D::<f32, U, U>::around_y(Angle::radians(
                        self.rotation.y,
                    )))
                    .then(&Rotation3D::<f32, U, U>::around_z(Angle::radians(
                        self.rotation.z,
                    )))
                    .to_transform(),
            )
            .then(&Transform3D::skew(
                Angle::radians(self.skew.x),
                Angle::radians(self.skew.y),
            ))
            .then_translate(self.origin.to_3d())
    }
}
