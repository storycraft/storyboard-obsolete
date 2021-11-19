/*
 * Created on Fri Nov 05 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use storyboard_box_2d::compositor::BoxCompositor;
use storyboard_graphics::{
    pipeline::PipelineTargetDescriptor,
    wgpu::{BindGroupLayout, Device},
};
use storyboard_path::compositor::PathCompositor;
use storyboard_primitive::compositor::PrimitiveCompositor;

#[derive(Debug)]
pub struct StoryboardCompositor {
    primitive: PrimitiveCompositor,
    box_2d: BoxCompositor,
    path: PathCompositor,
}

impl StoryboardCompositor {
    pub fn init(
        device: &Device,
        texture2d_bind_group_layout: &BindGroupLayout,
        pipeline_desc: PipelineTargetDescriptor,
    ) -> Self {
        let primitive = PrimitiveCompositor::init(
            device,
            texture2d_bind_group_layout,
            pipeline_desc.clone(),
        );

        let box_2d = BoxCompositor::init(
            device,
            texture2d_bind_group_layout,
            pipeline_desc.clone(),
        );

        let path = PathCompositor::init(device, pipeline_desc);

        Self {
            primitive,
            box_2d,
            path,
        }
    }

    pub fn primitive(&self) -> &PrimitiveCompositor {
        &self.primitive
    }

    pub fn box_2d(&self) -> &BoxCompositor {
        &self.box_2d
    }

    pub fn path(&self) -> &PathCompositor {
        &self.path
    }
}
