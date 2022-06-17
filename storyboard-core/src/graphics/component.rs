use wgpu::CommandEncoder;

use std::fmt::Debug;

use crate::graphics::renderer::{
    context::{DrawContext, RenderContext},
    ComponentQueue,
};

use super::renderer::pass::StoryboardRenderPass;

pub trait Drawable: Send {
    fn prepare(
        &self,
        component_queue: &mut ComponentQueue,
        ctx: &mut DrawContext,
        encoder: &mut CommandEncoder,
        depth: f32,
    );
}

impl Debug for dyn Drawable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Drawable").finish_non_exhaustive()
    }
}

pub trait Component: Send {
    fn render_opaque<'rpass>(
        &'rpass self,
        ctx: &RenderContext<'rpass>,
        pass: &mut StoryboardRenderPass<'rpass>,
    );

    fn render_transparent<'rpass>(
        &'rpass self,
        ctx: &RenderContext<'rpass>,
        pass: &mut StoryboardRenderPass<'rpass>,
    );
}

impl Debug for dyn Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Component").finish_non_exhaustive()
    }
}
