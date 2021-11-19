/*
 * Created on Mon Sep 13 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

pub mod index;
pub mod stream;

use wgpu::{Buffer, BufferAddress, BufferDescriptor, BufferUsages, Device};

#[derive(Debug)]
pub struct GrowingBuffer {
    label: Option<String>,
    usages: BufferUsages,
    mapped_at_creation: bool,

    buffer: Option<Buffer>,
    buffer_size: BufferAddress,
}

impl GrowingBuffer {
    pub const fn new(label: Option<String>, usages: BufferUsages, mapped_at_creation: bool) -> Self {
        Self {
            label,
            usages,
            mapped_at_creation,
            buffer: None,
            buffer_size: 0
        }
    }

    pub fn alloc(&mut self, device: &Device, size: BufferAddress) -> &Buffer {
        if self.buffer.is_none() || self.buffer_size < size || self.buffer_size > size * 3 {
            let buf = device.create_buffer(&BufferDescriptor {
                label: self.label.as_deref(),
                size,
                usage: self.usages,
                mapped_at_creation: self.mapped_at_creation,
            });

            self.buffer = Some(buf);
        }

        self.buffer.as_ref().unwrap()
    }
}
