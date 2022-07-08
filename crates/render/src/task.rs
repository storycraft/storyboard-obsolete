use std::{
    iter,
    num::NonZeroU32,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    time::Duration, hint,
};

use crate::{
    backend::StoryboardBackend,
    component::Drawable,
    renderer::{surface::{StoryboardSurfaceRenderer, SurfaceConfiguration}, RendererData},
};
use crossbeam_channel::{bounded, Receiver, Sender};
use parking_lot::{Mutex, MutexGuard};
use storyboard_core::{tick_task::IndependentTickTask, time_sampler::TimeSampler};
use trait_stack::TraitStack;
use triple_buffer::{Input, Output, TripleBuffer};
use wgpu::{CommandBuffer, Maintain};

#[derive(Debug)]
pub struct RenderTask {
    renderer_config: Arc<(Mutex<RenderConfiguration>, AtomicBool)>,
    input: Input<(TraitStack<dyn Drawable + 'static>, Vec<CommandBuffer>)>,

    frame_rate: Arc<AtomicU64>,

    signal_sender: Sender<()>,
    task: IndependentTickTask<RenderTaskData>,
}

impl RenderTask {
    pub fn run(
        backend: Arc<StoryboardBackend>,
        renderer: StoryboardSurfaceRenderer,
        renderer_data: Arc<RendererData>,
        task_config: RenderTaskConfiguration,
    ) -> Self {
        let (input, output) = TripleBuffer::default().split();

        let (signal_sender, signal_receiver) = bounded(2);

        let frame_rate = Arc::new(AtomicU64::new(0));

        let renderer_config = Arc::new((
            Mutex::new(RenderConfiguration {
                surface: renderer.configuration(),
                task: task_config,
            }),
            AtomicBool::new(false),
        ));

        let data = RenderTaskData {
            backend,
            renderer_data,

            configuration: renderer_config.clone(),
            signal_receiver,
            output,

            frame_sampler: TimeSampler::new(task_config.report_rate),
            max_fps: task_config.max_fps,
            frame_rate: frame_rate.clone(),

            renderer,
        };

        let task = IndependentTickTask::run(data, |data| {
            if data.signal_receiver.recv().is_ok() {
                let start = data.frame_sampler.sample_start();

                if data.configuration.1.load(Ordering::Relaxed) {
                    data.configuration.1.store(false, Ordering::Relaxed);

                    let configuration = data.configuration.0.lock();
                    data.renderer.set_configuration(configuration.surface);

                    data.frame_sampler.report_rate = configuration.task.report_rate;
                    data.max_fps = configuration.task.max_fps;
                }

                if data.output.update() {
                    if !data.output.output_buffer().0.is_empty() {
                        if let Some(res) = data.renderer.render(
                            data.backend.device(),
                            data.backend.queue(),
                            data.output.output_buffer().0.iter(),
                            &data.renderer_data
                        ) {
                            data.backend.device().poll(Maintain::Wait);
                            data.backend.queue().submit(
                                iter::once(res.command_buffer)
                                    .chain(data.output.output_buffer().1.drain(..)),
                            );

                            res.surface_texture.present();
                        }
                    } else if !data.output.output_buffer().1.is_empty() {
                        data.backend.device().poll(Maintain::Wait);
                        data.backend
                            .queue()
                            .submit(data.output.output_buffer().1.drain(..));
                    }
                }

                if let Some(max_fps) = data.max_fps {
                    while start.elapsed() < Duration::from_secs_f32(1.0 / max_fps.get() as f32) {
                        hint::spin_loop();
                    }
                }

                data.frame_sampler.sample_end();

                if let Some(rate) = data.frame_sampler.average_elapsed() {
                    data.frame_rate.store(rate.to_bits(), Ordering::Relaxed);
                }
            }
        });

        Self {
            renderer_config,
            frame_rate,
            input,
            signal_sender,
            task,
        }
    }

    pub fn configuration(&self) -> RenderConfiguration {
        *self.renderer_config.0.lock()
    }

    pub fn configuration_mut(&self) -> MutexGuard<RenderConfiguration> {
        let lock = self.renderer_config.0.lock();
        self.renderer_config.1.store(true, Ordering::Relaxed);

        lock
    }

    pub fn set_configuration(&self, configuration: RenderConfiguration) {
        *self.renderer_config.0.lock() = configuration;
        self.renderer_config.1.store(true, Ordering::Relaxed);
    }

    pub fn frame_rate(&self) -> f64 {
        f64::from_bits(self.frame_rate.load(Ordering::Relaxed))
    }

    pub fn interrupted(&self) -> bool {
        self.task.interrupted()
    }

    pub fn interrupt(&mut self) {
        self.task.interrupt();
        self.signal_sender.try_send(()).ok();
    }

    #[inline]
    pub const fn threaded(&self) -> bool {
        self.task.threaded()
    }

    pub fn set_threaded(&mut self, threaded: bool) {
        if self.threaded() && !threaded {
            self.interrupt();
        }

        self.task.set_threaded(threaded);
    }

    pub fn push(&mut self, item: impl Drawable + 'static) {
        self.input.input_buffer().0.push(item);
    }

    pub fn push_command_buffer(&mut self, buffer: CommandBuffer) {
        self.input.input_buffer().1.push(buffer);
    }

    pub fn submit(&mut self) {
        self.input.publish();
        self.signal_sender.try_send(()).ok();

        self.task.tick();
        self.input.input_buffer().0.clear();
        self.input.input_buffer().1.clear();
    }

    pub fn join(self) -> StoryboardSurfaceRenderer {
        self.task.join().renderer
    }
}

#[derive(Debug)]
struct RenderTaskData {
    backend: Arc<StoryboardBackend>,
    renderer_data: Arc<RendererData>,

    configuration: Arc<(Mutex<RenderConfiguration>, AtomicBool)>,
    signal_receiver: Receiver<()>,
    output: Output<(TraitStack<dyn Drawable + 'static>, Vec<CommandBuffer>)>,

    frame_sampler: TimeSampler,
    max_fps: Option<NonZeroU32>,
    frame_rate: Arc<AtomicU64>,

    renderer: StoryboardSurfaceRenderer,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderConfiguration {
    pub surface: SurfaceConfiguration,
    pub task: RenderTaskConfiguration,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderTaskConfiguration {
    pub report_rate: Duration,
    pub max_fps: Option<NonZeroU32>,
}

impl Default for RenderTaskConfiguration {
    fn default() -> Self {
        Self {
            report_rate: Duration::from_secs(1),
            max_fps: None,
        }
    }
}
