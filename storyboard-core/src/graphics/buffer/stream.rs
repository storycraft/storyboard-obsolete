/*
 * Created on Sat Apr 30 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{borrow::Cow, num::NonZeroU64, ops::Range};

use wgpu::{Buffer, BufferAddress, BufferBinding, BufferSlice, BufferUsages, Device, Queue};

use super::GrowingBuffer;

pub type StreamRange = Range<BufferAddress>;

#[derive(Debug)]
pub struct BufferStream<'a> {
    buffer: GrowingBuffer<'a>,
    data: Vec<u8>,
}

impl<'a> BufferStream<'a> {
    pub fn new(label: Option<Cow<'a, str>>, usages: BufferUsages) -> Self {
        Self {
            buffer: GrowingBuffer::new(
                label,
                usages | BufferUsages::COPY_DST,
                false,
            ),
            data: Vec::new(),
        }
    }

    /// Return next writer starting from end of buffer
    pub fn next_writer(&mut self) -> StreamWriter {
        let offset = self.data.len();

        StreamWriter {
            write_buffer: &mut self.data,
            offset: offset as BufferAddress,
        }
    }

    /// Write slice of data and return written range
    pub fn write_slice(&mut self, data: &[u8]) -> StreamRange {
        let mut writer = self.next_writer();
        writer.write(data);
        writer.finish()
    }

    /// Finish streaming and upload memory buffer to gpu
    pub fn finish(&mut self, device: &Device, queue: &Queue) -> StreamBuffer {
        let size = self.data.len() as BufferAddress;

        let padding = wgpu::COPY_BUFFER_ALIGNMENT - size % wgpu::COPY_BUFFER_ALIGNMENT;
        let buf_size = size + padding;

        let buffer = self.buffer.alloc(device, buf_size);
        queue.write_buffer(buffer, 0, &self.data);

        self.data.clear();

        StreamBuffer { buffer }
    }
}

#[derive(Debug)]
pub struct StreamWriter<'a> {
    write_buffer: &'a mut Vec<u8>,

    offset: BufferAddress,
}

impl<'a> StreamWriter<'a> {
    pub fn write(&mut self, data: &[u8]) {
        self.write_buffer.extend_from_slice(data);
    }

    /// Finish writer.
    /// Returns range of written data.
    pub fn finish(self) -> StreamRange {
        self.offset..(self.write_buffer.len() as BufferAddress)
    }
}

#[derive(Debug)]
pub struct StreamBuffer<'a> {
    buffer: &'a Buffer,
}

impl<'a> StreamBuffer<'a> {
    pub fn slice(&self, range: StreamRange) -> BufferSlice<'a> {
        self.buffer.slice(range)
    }

    pub fn binding(&self, range: StreamRange) -> BufferBinding<'a> {
        BufferBinding {
            buffer: self.buffer,
            offset: range.start,
            size: NonZeroU64::new(range.end - range.start),
        }
    }
}

#[cfg(test)]
#[test]
pub fn write_test() {
    use std::time::Instant;

    use wgpu::BufferUsages;

    let mut stream = BufferStream::new(None, BufferUsages::COPY_DST | BufferUsages::VERTEX);

    let start = Instant::now();

    for _ in 0..59999 {
        let mut entry = stream.next_writer();
        entry.write(&[1]);

        entry.finish();
    }

    println!("Writing took {} microseconds", start.elapsed().as_micros());
}
