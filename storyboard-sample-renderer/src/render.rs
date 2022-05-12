/*
 * Created on Sat Apr 30 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use storyboard_core::{graphics::buffer::stream::StreamRange, wgpu::RenderPass};

use crate::Box2D;

#[derive(Debug)]
pub struct BoxRenderer {

}

impl BoxRenderer {
    pub const fn new() -> Self {
        Self {

        }
    }

    pub fn prepare(&mut self, box_2d: &Box2D) -> Box2DRenderState {
        Box2DRenderState {
            vertex_range: 0..0,
            instance_range: 0..0,
            texture: None
        }
    }

    pub fn render<'r>(&self, box_2d_state: Box2DRenderState, pass: &mut RenderPass<'r>) {
        todo!()
    }

    pub fn finish(&mut self) {
        
    }
}

pub struct Box2DRenderState {
    pub vertex_range: StreamRange,
    pub instance_range: StreamRange,
    pub texture: Option<()>,
}
