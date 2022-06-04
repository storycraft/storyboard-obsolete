/*
 * Created on Sat Apr 30 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

pub mod stream;

use std::borrow::Cow;

use wgpu::{BufferUsages, BufferAddress, Buffer, Device, BufferDescriptor};

#[derive(Debug)]
/// Dynamically resizing buffer.
pub struct GrowingBuffer<'label> {
    label: Option<Cow<'label, str>>,
    usages: BufferUsages,
    mapped_at_creation: bool,

    buffer: Option<Buffer>,
    buffer_size: BufferAddress,
}

impl<'label> GrowingBuffer<'label> {
    pub const fn new(label: Option<Cow<'label, str>>, usages: BufferUsages, mapped_at_creation: bool) -> Self {
        Self {
            label,
            usages,
            mapped_at_creation,
            buffer: None,
            buffer_size: 0
        }
    }

    pub const fn size(&self) -> BufferAddress {
        self.buffer_size
    }

    /// Allocate and provide buffer given size.
    /// The size of buffer can be larger than requested size.
    /// Reusing buffer if possible.
    pub fn alloc(&mut self, device: &Device, size: BufferAddress) -> &Buffer {
        if self.buffer.is_none() || self.buffer_size < size {
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
