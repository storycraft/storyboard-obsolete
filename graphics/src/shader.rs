/*
 * Created on Sun Sep 19 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use wgpu::{Device, PipelineLayout, PipelineLayoutDescriptor, ShaderModule, ShaderModuleDescriptor, ShaderSource};

#[derive(Debug, Clone)]
pub struct RenderShaderDescriptor<'a> {
    pub label: Option<&'a str>
}

#[derive(Debug)]
pub struct RenderShader {
    module: ShaderModule,

    pipeline_layout: PipelineLayout,
}

impl RenderShader {
    pub fn new(
        module: ShaderModule,
        pipeline_layout: PipelineLayout,
    ) -> Self {
        Self {
            module,
            pipeline_layout,
        }
    }

    pub fn init(
        device: &Device,
        source: ShaderSource,
        desc: &RenderShaderDescriptor,
        pipeline_layout_desc: &PipelineLayoutDescriptor,
    ) -> Self {
        let module = device.create_shader_module(&ShaderModuleDescriptor {
            label: desc.label,
            source,
        });

        let pipeline_layout = device.create_pipeline_layout(pipeline_layout_desc);

        Self {
            module,
            pipeline_layout,
        }
    }

    pub fn module(&self) -> &ShaderModule {
        &self.module
    }

    pub fn pipeline_layout(&self) -> &PipelineLayout {
        &self.pipeline_layout
    }
}
