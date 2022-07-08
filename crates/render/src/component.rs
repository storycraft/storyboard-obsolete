use wgpu::{CommandEncoder, TextureFormat, DepthStencilState, CompareFunction, StencilState, StencilFaceState, DepthBiasState};

use std::fmt::Debug;

use crate::renderer::{
    context::{DrawContext, RenderContext},
    ComponentQueue,
};

use super::renderer::pass::StoryboardRenderPass;

pub const DEPTH_TEXTURE_FORMAT: TextureFormat = TextureFormat::Depth32Float;

pub const OPAQUE_DEPTH_STENCIL: DepthStencilState = DepthStencilState {
    format: DEPTH_TEXTURE_FORMAT,
    depth_write_enabled: true,
    depth_compare: CompareFunction::Less,
    stencil: StencilState {
        front: StencilFaceState::IGNORE,
        back: StencilFaceState::IGNORE,
        read_mask: 0,
        write_mask: 0,
    },
    bias: DepthBiasState {
        constant: 0,
        slope_scale: 0.0,
        clamp: 0.0,
    },
};
pub const TRANSPARENT_DEPTH_STENCIL: DepthStencilState = DepthStencilState {
    format: DEPTH_TEXTURE_FORMAT,
    depth_write_enabled: false,
    depth_compare: CompareFunction::Less,
    stencil: StencilState {
        front: StencilFaceState::IGNORE,
        back: StencilFaceState::IGNORE,
        read_mask: 0,
        write_mask: 0,
    },
    bias: DepthBiasState {
        constant: 0,
        slope_scale: 0.0,
        clamp: 0.0,
    },
};

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
