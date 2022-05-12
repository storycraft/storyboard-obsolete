/*
 * Created on Sat Apr 30 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

#[derive(Debug, Clone, Copy)]
/// Pixel unit px
pub struct PixelUnit;

#[derive(Debug, Clone, Copy)]
/// Unit projected with ortho projection. [-1.0, 1.0]
pub struct RenderUnit;

#[derive(Debug, Clone, Copy)]
/// Unit on wgpu texture. [0.0, 1.0]
pub struct TextureUnit;
