/*
 * Created on Sat Oct 02 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{hash::BuildHasherDefault, ops::Range};

use rustc_hash::FxHashMap;
use storyboard_core::wgpu::{
    util::RenderEncoder, BindGroup, Buffer, BufferAddress, BufferSlice, DynamicOffset,
    IndexFormat, RenderPipeline, ShaderStages,
};

#[derive(Debug)]
pub struct StoryboardRenderPass<'a, T> {
    pass: T,

    current_bind_groups: FxHashMap<u32, (&'a BindGroup, usize)>,

    current_pipeline: Option<&'a RenderPipeline>
}

impl<'a, T: RenderEncoder<'a>> StoryboardRenderPass<'a, T> {
    pub fn new(pass: T) -> Self {
        Self {
            pass,

            current_pipeline: None,

            current_bind_groups: FxHashMap::with_capacity_and_hasher(
                16,
                BuildHasherDefault::default(),
            )
        }
    }

    pub fn set_pipeline(&mut self, pipeline: &'a RenderPipeline) {
        if let Some(current_pipeline) = &self.current_pipeline {
            if std::ptr::eq(*current_pipeline, pipeline) {
                return;
            }
        }

        self.current_pipeline = Some(pipeline.clone());

        self.reset_pipeline_desc();

        self.pass.set_pipeline(pipeline)
    }

    pub fn set_bind_group(
        &mut self,
        index: u32,
        bind_group: &'a BindGroup,
        offsets: &[DynamicOffset],
    ) {
        let offsets_ptr = offsets.as_ptr() as usize;

        if let Some((current_group, current_offsets_ptr)) = self.current_bind_groups.get(&index) {
            if std::ptr::eq(bind_group, *current_group)
                && offsets.as_ptr() as usize == *current_offsets_ptr
            {
                return;
            }
        }

        self.current_bind_groups
            .insert(index, (bind_group, offsets_ptr));

        self.pass.set_bind_group(index, bind_group, offsets)
    }

    pub fn set_index_buffer(&mut self, slice: BufferSlice<'a>, index_format: IndexFormat) {
        self.pass.set_index_buffer(slice, index_format)
    }

    #[inline(always)]
    pub fn set_vertex_buffer(&mut self, slot: u32, slice: BufferSlice<'a>) {
        self.pass.set_vertex_buffer(slot, slice)
    }

    #[inline(always)]
    pub fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>) {
        self.pass.draw(vertices, instances)
    }

    #[inline(always)]
    pub fn draw_indexed(&mut self, indices: Range<u32>, base_vertex: i32, instances: Range<u32>) {
        self.pass.draw_indexed(indices, base_vertex, instances)
    }

    #[inline(always)]
    pub fn draw_indirect(&mut self, indirect_buffer: &'a Buffer, indirect_offset: BufferAddress) {
        self.pass.draw_indirect(indirect_buffer, indirect_offset)
    }

    #[inline(always)]
    pub fn draw_indexed_indirect(
        &mut self,
        indirect_buffer: &'a Buffer,
        indirect_offset: BufferAddress,
    ) {
        self.pass
            .draw_indexed_indirect(indirect_buffer, indirect_offset)
    }

    #[inline(always)]
    pub fn set_push_constants(&mut self, stages: ShaderStages, offset: u32, data: &[u8]) {
        self.pass.set_push_constants(stages, offset, data)
    }

    fn reset_pipeline_desc(&mut self) {
        self.current_bind_groups.clear();
    }
}

impl<'a, T: RenderEncoder<'a>> RenderEncoder<'a> for StoryboardRenderPass<'a, T> {
    fn set_bind_group(&mut self, index: u32, bind_group: &'a BindGroup, offsets: &[DynamicOffset]) {
        self.set_bind_group(index, bind_group, offsets)
    }

    fn set_pipeline(&mut self, pipeline: &'a RenderPipeline) {
        self.set_pipeline(pipeline)
    }

    fn set_index_buffer(&mut self, slice: BufferSlice<'a>, index_format: IndexFormat) {
        self.set_index_buffer(slice, index_format)
    }

    fn set_vertex_buffer(&mut self, slot: u32, slice: BufferSlice<'a>) {
        self.set_vertex_buffer(slot, slice)
    }

    fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>) {
        self.draw(vertices, instances)
    }

    fn draw_indexed(&mut self, indices: Range<u32>, base_vertex: i32, instances: Range<u32>) {
        self.draw_indexed(indices, base_vertex, instances)
    }

    fn draw_indirect(&mut self, indirect_buffer: &'a Buffer, indirect_offset: BufferAddress) {
        self.draw_indirect(indirect_buffer, indirect_offset)
    }

    fn draw_indexed_indirect(
        &mut self,
        indirect_buffer: &'a Buffer,
        indirect_offset: BufferAddress,
    ) {
        self.draw_indexed_indirect(indirect_buffer, indirect_offset)
    }

    fn set_push_constants(&mut self, stages: ShaderStages, offset: u32, data: &[u8]) {
        self.set_push_constants(stages, offset, data)
    }
}
