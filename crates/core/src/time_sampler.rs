use instant::{Duration, Instant};

#[derive(Debug)]
pub struct TimeSampler {
    pub report_rate: Duration,

    fps_sample_start: Option<Instant>,
    last_sample_start: Instant,
    total_elapsed: Duration,
    total_count: u32,

    average_elapsed: Option<f64>
}

impl TimeSampler {
    pub fn new(report_interval: Duration) -> Self {
        Self {
            report_rate: report_interval,
            fps_sample_start: None,
            last_sample_start: Instant::now(),
            total_elapsed: Duration::ZERO,
            total_count: 0,

            average_elapsed: None,
        }
    }

    pub fn sample_start(&mut self) -> Instant {
        let now = Instant::now();

        if self.fps_sample_start.is_none() {
            self.fps_sample_start = Some(now);
        }

        self.last_sample_start = now;

        now
    }

    pub fn sample_end(&mut self) -> Duration {
        let elapsed = self.last_sample_start.elapsed();

        self.total_elapsed += elapsed;
        self.total_count += 1;

        if let Some(fps_sample_start) = self.fps_sample_start {
            if fps_sample_start.elapsed() >= self.report_rate {
                self.fps_sample_start.take();
                let rate = 1.0 / self
                    .total_elapsed
                    .div_f32(self.total_count as _)
                    .as_secs_f64();
    
                self.total_elapsed = Duration::ZERO;
                self.total_count = 0;
    
                self.average_elapsed = Some(rate);
            }
        }
        

        elapsed
    }

    pub fn average_elapsed(&mut self) -> Option<f64> {
        self.average_elapsed.take()
    }
}
