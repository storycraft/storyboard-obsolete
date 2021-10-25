/*
 * Created on Thu Sep 16 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use wgpu::{
    Buffer, BufferAddress, BufferDescriptor, BufferUsages, Device, IndexFormat,
};

use crate::pass::RenderBufferSlice;

#[derive(Debug)]
pub struct IndexBuffer {
    size: BufferAddress,
    buffer: Buffer,
}

impl IndexBuffer {
    pub const FORMAT: IndexFormat = IndexFormat::Uint16;

    pub fn init(device: &Device, label: Option<&str>, indices: &[u16]) -> Self {
        let size = indices.len() as BufferAddress * std::mem::size_of::<u16>() as BufferAddress;
        let padding = size % wgpu::COPY_BUFFER_ALIGNMENT;

        let buffer = device.create_buffer(&BufferDescriptor {
            label,
            size: size + padding,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: true,
        });
        buffer
            .slice(..size)
            .get_mapped_range_mut()
            .copy_from_slice(bytemuck::cast_slice(indices));
        buffer.unmap();

        Self { size, buffer }
    }

    pub fn size(&self) -> BufferAddress {
        self.size
    }

    pub fn slice(&self) -> RenderBufferSlice {
        RenderBufferSlice::new(&self.buffer, 0, Some(self.size))
    }
}
