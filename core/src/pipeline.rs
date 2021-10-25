/*
 * Created on Mon Sep 20 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use wgpu::{ColorTargetState, DepthStencilState, MultisampleState, PolygonMode, PrimitiveTopology};

#[derive(Debug, Clone, Default)]
pub struct PipelineTargetDescriptor<'a> {
    pub fragments_targets: &'a [ColorTargetState],

    pub topology: Option<PrimitiveTopology>,
    pub polygon_mode: PolygonMode,

    pub depth_stencil: Option<DepthStencilState>,
    pub multisample: MultisampleState,
}
