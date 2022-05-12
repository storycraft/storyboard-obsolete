/*
 * Created on Sat Apr 30 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{
    thread,
    time::{Duration, Instant},
};

/// Run task for at least interval time
pub struct TimedTask<const SAMPLES: usize> {
    pub interval: Duration,

    samples: [Duration; SAMPLES],
    cursor: usize,
}

impl<const SAMPLES: usize> TimedTask<SAMPLES> {
    pub const fn new(interval: Duration) -> Self {
        Self {
            interval,

            samples: [Duration::ZERO; SAMPLES],
            cursor: 0,
        }
    }

    pub fn timing(&self) -> Duration {
        self.samples
            .iter()
            .sum::<Duration>()
            .checked_div(SAMPLES as u32)
            .unwrap_or_default()
    }

    pub const fn samples(&self) -> &[Duration] {
        &self.samples
    }

    /// Run task and sample elapsed time
    pub fn run<T: FnOnce() -> ()>(&mut self, func: T) {
        let now = Instant::now();
        func();
        let elapsed = now.elapsed();

        if elapsed < self.interval {
            thread::sleep(self.interval - elapsed);
        }

        self.samples[self.cursor] = now.elapsed();

        self.cursor = (self.cursor + 1) % SAMPLES;
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::TimedTask;

    #[test]
    pub fn test() {
        let mut timed = TimedTask::<20>::new(Duration::from_millis(100));
        
        for i in 0..20 {
            timed.run(move || println!("task {}", i))
        }

        println!("Task took average {} ms", timed.timing().as_millis())
    }
}
