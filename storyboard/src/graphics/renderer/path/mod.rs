/*
 * Created on Fri Nov 19 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use bytemuck::{Pod, Zeroable};
use euclid::Rect;
use lyon::path::Path;
use palette::LinSrgba;

use crate::graphics::PixelUnit;

#[derive(Debug, Clone)]
pub struct ScalablePath {
    pub path: Path,
    pub rect: Rect<f32, PixelUnit>,
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct PathVertex {
    pub position: [f32; 3],
    pub color: LinSrgba<f32>,
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct PathInstance {
    pub matrix: [f32; 16],
}
