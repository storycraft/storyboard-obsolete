use instant::Instant;
use std::time::Duration;

/// Run task for at least interval time
#[derive(Debug)]
pub struct TimeSampler<const SAMPLES: usize> {
    pub interval: Duration,

    samples: [Instant; SAMPLES],
    cursor: usize,
}

impl<const SAMPLES: usize> TimeSampler<SAMPLES> {
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,

            samples: [Instant::now() - interval; SAMPLES],
            cursor: 0,
        }
    }

    pub fn timing(&self) -> Duration {
        self.samples
            .windows(2)
            .map(|window| window[1].duration_since(window[0]))
            .sum::<Duration>()
            .checked_div(SAMPLES as u32)
            .unwrap_or_default()
    }

    pub const fn samples(&self) -> &[Instant] {
        &self.samples
    }

    pub fn last_sample(&self) -> Instant {
        self.samples[self.cursor]
    }

    /// Run task and sample elapsed time if the duration of last run is equal or greater than interval
    pub fn run<T: FnOnce()>(&mut self, func: T) {
        let now = Instant::now();
        if now.duration_since(self.last_sample()) >= self.interval {
            func();
            self.samples[self.cursor] = now;

            self.cursor = (self.cursor + 1) % SAMPLES;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::TimeSampler;

    #[test]
    pub fn test() {
        let mut timed = TimeSampler::<20>::new(Duration::from_millis(100));

        for i in 0..20 {
            timed.run(move || println!("task {}", i));
        }

        println!("Task took average {} ms", timed.timing().as_millis())
    }
}
