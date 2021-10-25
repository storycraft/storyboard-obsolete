/*
 * Created on Tue Sep 14 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use bytemuck::Pod;

#[derive(Debug, Default, Clone)]
pub struct VertexBatch<V, const DRAW_COUNT: usize> {
    instances: u32,
    buf: Vec<V>,
}

impl<V: Pod, const DRAW_COUNT: usize> VertexBatch<V, DRAW_COUNT> {
    pub fn new() -> Self {
        Self {
            instances: 0,
            buf: Vec::with_capacity(DRAW_COUNT),
        }
    }

    pub fn instances(&self) -> u32 {
        self.instances
    }

    pub fn append(&mut self, instance: &[V; DRAW_COUNT]) {
        self.buf.extend_from_slice(instance);
        self.instances += 1;
    }

    pub fn commit(&self) -> BatchInfo {
        let instances = self.instances;

        let data = bytemuck::cast_slice(&self.buf);

        BatchInfo { instances, data }
    }

    pub fn finish(&mut self) {
        if self.buf.len() > 512 * DRAW_COUNT && self.buf.capacity() * 3 > self.buf.len() {
            self.buf = Vec::with_capacity(self.buf.len() * 3 / 2)
        } else {
            self.buf.clear();
        }

        self.instances = 0;
    }
}

#[derive(Debug)]
pub struct BatchInfo<'a> {
    pub instances: u32,
    pub data: &'a [u8],
}
