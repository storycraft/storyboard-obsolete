/*
 * Created on Sat Oct 02 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{hash::BuildHasherDefault, ops::Range};

use rustc_hash::FxHashMap;
use wgpu::{
    BindGroup, Buffer, BufferAddress, BufferSlice, Color, DynamicOffset, IndexFormat, RenderBundle,
    RenderPass, RenderPipeline,
};

#[derive(Debug)]
pub struct StoryboardRenderPass<'a> {
    pass: RenderPass<'a>,

    current_bind_groups: FxHashMap<u32, (&'a BindGroup, usize)>,

    current_pipeline: Option<&'a RenderPipeline>,
    current_index_buffer: Option<(RenderBufferSlice<'a>, IndexFormat)>,
}

impl<'a> StoryboardRenderPass<'a> {
    pub fn new(pass: RenderPass<'a>) -> Self {
        Self {
            pass,

            current_pipeline: None,

            current_bind_groups: FxHashMap::with_capacity_and_hasher(
                16,
                BuildHasherDefault::default(),
            ),
            current_index_buffer: None,
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
            if std::ptr::eq(bind_group, *current_group) && offsets.as_ptr() as usize == *current_offsets_ptr {
                return;
            }
        }

        self.current_bind_groups.insert(index, (bind_group, offsets_ptr));

        self.pass.set_bind_group(index, bind_group, offsets)
    }

    pub fn set_index_buffer(&mut self, slice: RenderBufferSlice<'a>, index_format: IndexFormat) {
        if let Some((current_slice, current_format)) = &self.current_index_buffer {
            if current_slice.eq(&slice) && index_format == *current_format {
                return;
            }
        }

        self.current_index_buffer = Some((slice, index_format));
        self.pass.set_index_buffer(slice.into(), index_format)
    }

    pub fn execute_bundles<I: Iterator<Item = &'a RenderBundle>>(&mut self, render_bundles: I) {
        self.current_pipeline = None;
        self.reset_pipeline_desc();
        self.pass.execute_bundles(render_bundles)
    }

    #[inline(always)]
    pub fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32, min_depth: f32, max_depth: f32) {
        self.pass.set_viewport(x, y, w, h, min_depth, max_depth)
    }

    #[inline(always)]
    pub fn set_scissor_rect(&mut self, x: u32, y: u32, width: u32, height: u32) {
        self.pass.set_scissor_rect(x, y, width, height)
    }

    #[inline(always)]
    pub fn set_blend_constant(&mut self, color: Color) {
        self.pass.set_blend_constant(color)
    }

    #[inline(always)]
    pub fn set_stencil_reference(&mut self, reference: u32) {
        self.pass.set_stencil_reference(reference)
    }

    #[inline(always)]
    pub fn set_vertex_buffer(&mut self, slot: u32, slice: RenderBufferSlice<'a>) {
        self.pass.set_vertex_buffer(slot, slice.into())
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

    #[inline]
    fn reset_pipeline_desc(&mut self) {
        self.current_index_buffer = None;
        self.current_bind_groups.clear();
    }

    pub fn into_inner(self) -> RenderPass<'a> {
        self.pass
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RenderBufferSlice<'a> {
    buffer: &'a Buffer,
    offset: BufferAddress,
    size: Option<BufferAddress>,
}

impl<'a> RenderBufferSlice<'a> {
    pub const fn new(buffer: &'a Buffer, offset: BufferAddress, size: Option<BufferAddress>) -> Self {
        Self {
            buffer,
            offset,
            size,
        }
    }
}

impl<'a> Into<BufferSlice<'a>> for RenderBufferSlice<'a> {
    fn into(self) -> BufferSlice<'a> {
        match self.size {
            Some(size) => self.buffer.slice(self.offset..(self.offset + size)),
            None => self.buffer.slice(self.offset..),
        }
    }
}

impl<'a> PartialEq for RenderBufferSlice<'a> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.buffer, other.buffer)
            && self.offset == other.offset
            && self.size == other.size
    }
}

impl<'a> Eq for RenderBufferSlice<'a> {}
