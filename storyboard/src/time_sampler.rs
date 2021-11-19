/*
 * Created on Fri Nov 19 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

pub struct TimeSampler<const SAMPLES: usize> {
    samples: [u64; SAMPLES],
    cursor: usize,
}

impl<const SAMPLES: usize> TimeSampler<SAMPLES> {
    pub fn new() -> Self {
        Self {
            samples: [0; SAMPLES],
            cursor: 0
        }
    }

    pub fn timing(&self) -> f64 {
        self.samples.iter().sum::<u64>() as f64 / SAMPLES as f64
    }

    pub fn samples(&self) -> &[u64] {
        &self.samples
    }

    pub fn push(&mut self, time: u64) {
        let cursor = self.cursor;

        self.samples[cursor] = time;

        self.cursor = (self.cursor + 1) % SAMPLES;
    }
}
