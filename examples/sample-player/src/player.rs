use std::{
    borrow::Cow,
    io::{Read, Seek},
    sync::Arc,
};

use instant::Instant;
use rodio::{buffer::SamplesBuffer, Decoder, Sink, Source};
use rustfft::{num_complex::Complex, Fft, FftPlanner};
use storyboard::{
    app::{StoryboardAppProp, StoryboardAppState},
    core::{
        color::ShapeColor,
        euclid::{Point2D, Rect, Size2D, Transform3D},
    },
    winit::event::{Event, WindowEvent},
};
use storyboard_box2d::{Box2D, Box2DStyle};
use storyboard_state::{State, StateStatus};
use storyboard_text::{font::Font, Text, cache::GlyphCache};

use crate::{StoryboardStateData, FONT};

pub const BAR_COUNT: usize = 36;
pub struct Player {
    sink: Sink,
    samples: Vec<f32>,
    sample_rate: u32,
    channels: u16,
    start_time: Instant,
    fft: Arc<dyn Fft<f32>>,
    bars: [f32; BAR_COUNT],

    glyph_cache: GlyphCache,
    fps: Text,
}

impl Player {
    pub fn new(sink: Sink, decoder: Decoder<impl Read + Seek>) -> Self {
        let sample_rate = decoder.sample_rate();
        let channels = decoder.channels();

        let fft = FftPlanner::new().plan_fft_forward(decoder.sample_rate() as usize);

        Self {
            sink,
            samples: decoder.convert_samples().collect(),
            sample_rate,
            channels,
            start_time: Instant::now(),
            fft,
            bars: [0.0; BAR_COUNT],

            glyph_cache: GlyphCache::new(),
            fps: Text::new(
                Point2D::new(10.0, 10.0),
                16,
                Transform3D::identity(),
                Font::new(Cow::Borrowed(FONT), 0).unwrap(),
                Cow::Borrowed(""),
            ),
        }
    }
}

impl State<StoryboardStateData> for Player {
    fn load(&mut self, _: &StoryboardAppProp) {
        println!(
            "Channels: {}, sample_rate: {}",
            self.channels, self.sample_rate
        );

        self.sink.append(SamplesBuffer::new(
            self.channels,
            self.sample_rate,
            self.samples.clone(),
        ));
        self.sink.play();

        self.start_time = Instant::now();
    }

    fn unload(&mut self, _: &StoryboardAppProp) {
        self.sink.stop();
    }

    fn update(
        &mut self,
        system_prop: &StoryboardAppProp,
        system_state: &mut StoryboardAppState,
    ) -> StateStatus<StoryboardStateData> {
        match &system_state.event {
            Event::RedrawRequested(_) => {
                let (_, win_height): (u32, u32) = system_prop.window.inner_size().into();

                for (i, bar) in self.bars.iter().enumerate() {
                    let height = bar.max(10.0);

                    let size = Size2D::new(10.0_f32, height);
                    let origin = Point2D::new(
                        20.0 + i as f32 * 20.0,
                        win_height as f32 - size.height - 20.0,
                    );

                    system_state.draw(Box2D {
                        bounds: Rect::new(origin, size),
                        texture: None,
                        fill_color: ShapeColor::TRANSPARENT,
                        border_color: ShapeColor::WHITE,
                        style: Box2DStyle {
                            border_radius: [5.0; 4],
                            border_thickness: 2.0,

                            ..Default::default()
                        },
                        transform: Transform3D::identity(),
                    });
                }

                self.fps.set_text(Cow::Owned(format!(
                    "{} fps",
                    system_state.render_task.frame_rate().floor()
                )));

                self.fps.update(
                    system_prop.backend.device(),
                    system_prop.backend.queue(),
                    system_prop.window.scale_factor() as _,
                    &system_prop.texture_data,
                    &mut self.glyph_cache,
                );

                system_state.draw(self.fps.draw(&ShapeColor::WHITE));

                system_state.render();
            }

            Event::MainEventsCleared => {
                let time = self.start_time.elapsed();

                let idx = ((time.as_millis() as f64 / 1000.0)
                    * self.sample_rate as f64
                    * self.channels as f64) as usize;

                if (idx + self.sample_rate as usize) <= self.samples.len() {
                    let mut buf = Vec::with_capacity(self.sample_rate as usize);

                    for i in 0..self.sample_rate as usize {
                        buf.push(Complex::new(self.samples[idx + i], 0.0));
                    }

                    self.fft.process(&mut buf);

                    for i in 0..self.bars.len() {
                        let start = (i + 1) * (i + 1) * 8;
                        let end = (i + 2) * (i + 2) * 8;

                        let sum = buf[start..end]
                            .iter()
                            .map(|val| val.re.hypot(val.im))
                            .sum::<f32>();

                        self.bars[i] = sum / self.bars.len() as f32 / 2.0;
                    }
                } else {
                    return StateStatus::PopState;
                }

                system_prop.request_redraw();
            }

            Event::WindowEvent {
                window_id: _,
                event: WindowEvent::CloseRequested,
            } => {
                return StateStatus::PopState;
            }

            _ => {}
        };

        StateStatus::Poll
    }
}
