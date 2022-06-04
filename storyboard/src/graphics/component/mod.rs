/*
 * Created on Thu May 05 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

pub mod box2d;
pub mod primitive;
pub mod common;

use std::fmt::Debug;

use storyboard_core::wgpu::util::RenderEncoder;

use super::{
    context::{DrawContext, RenderContext},
    renderer::ComponentQueue,
};

pub trait Drawable: Send + Sync {
    fn prepare(&self, component_queue: &mut ComponentQueue, ctx: &mut DrawContext, depth: f32);
}

impl Debug for dyn Drawable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Drawable").finish_non_exhaustive()
    }
}

pub trait Component {
    fn render<'rpass>(
        &'rpass self,
        ctx: &mut RenderContext<'rpass>,
        pass: &mut dyn RenderEncoder<'rpass>,
    );
}

impl Debug for dyn Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Component").finish_non_exhaustive()
    }
}
