/*
 * Created on Mon Sep 27 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::num::NonZeroU64;

use wgpu::{Buffer, BufferAddress, BufferBinding, BufferUsages, Device};

use crate::graphics::{buffer::GrowingBuffer, pass::RenderBufferSlice};

#[derive(Debug)]
pub struct StreamBufferAllocator {
    buffer: GrowingBuffer,

    buf: Vec<u8>,
}

impl StreamBufferAllocator {
    pub fn new(usages: BufferUsages) -> Self {
        Self {
            buffer: GrowingBuffer::new(
                Some("Stream allocator buffer".into()),
                usages | BufferUsages::COPY_DST | BufferUsages::MAP_WRITE,
                true,
            ),
            buf: Vec::new(),
        }
    }

    pub fn start_entry(&mut self) -> StreamEntry {
        let offset = self.buf.len();

        StreamEntry {
            write_buffer: &mut self.buf,
            offset: offset as BufferAddress,
        }
    }

    pub fn flush(&mut self, device: &Device) -> StreamBuffer {
        let size = self.buf.len() as BufferAddress;
        let padding = wgpu::COPY_BUFFER_ALIGNMENT - size % wgpu::COPY_BUFFER_ALIGNMENT;

        self.buf.reserve(padding as usize);
        for _ in 0..padding {
            self.buf.push(0);
        }

        let buf_size = size + padding;

        let buffer = self.buffer.alloc(device, buf_size);
        buffer
            .slice(..buf_size)
            .get_mapped_range_mut()
            .copy_from_slice(&mut self.buf);
        buffer.unmap();

        self.buf.clear();

        StreamBuffer { buffer }
    }
}

#[derive(Debug)]
pub struct StreamEntry<'a> {
    write_buffer: &'a mut Vec<u8>,

    offset: BufferAddress,
}

impl<'a> StreamEntry<'a> {
    pub fn write(&mut self, data: &[u8]) {
        self.write_buffer.extend_from_slice(data);
    }

    pub fn finish(self) -> StreamSlice {
        StreamSlice {
            offset: self.offset,
            size: self.write_buffer.len() as u64 - self.offset,
        }
    }
}

#[derive(Debug)]
pub struct StreamBuffer<'a> {
    buffer: &'a Buffer,
}

impl StreamBuffer<'_> {
    pub fn slice(&self, slice: &StreamSlice) -> RenderBufferSlice {
        RenderBufferSlice::new(&self.buffer, slice.offset, Some(slice.size))
    }

    pub fn binding(&self, slice: &StreamSlice) -> BufferBinding {
        BufferBinding {
            buffer: &self.buffer,
            offset: slice.offset,
            size: NonZeroU64::new(slice.size),
        }
    }
}

#[derive(Debug)]
pub struct StreamSlice {
    offset: BufferAddress,
    size: BufferAddress,
}

#[cfg(test)]
#[test]
pub fn write_test() {
    use std::time::Instant;

    use wgpu::BufferUsages;

    let mut allocator = StreamBufferAllocator::new(BufferUsages::COPY_DST | BufferUsages::VERTEX);

    let start = Instant::now();

    for _ in 0..59999 {
        let mut entry = allocator.start_entry();
        entry.write(&[1]);

        entry.finish();
    }

    println!("Writing took {} microseconds", start.elapsed().as_micros());
}
