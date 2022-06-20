use std::{
    iter,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use crossbeam_channel::{bounded, Receiver, Sender};
use parking_lot::Mutex;
use storyboard_core::{
    graphics::{
        backend::StoryboardBackend,
        component::Drawable,
        renderer::surface::{StoryboardSurfaceRenderer, SurfaceConfiguration},
    },
    tick_task::DedicatedTickTask,
    wgpu::{CommandBuffer, Maintain},
};
use trait_stack::TraitStack;
use triple_buffer::{Input, Output, TripleBuffer};

#[derive(Debug)]
pub struct RenderTask {
    renderer_config: Arc<(Mutex<SurfaceConfiguration>, AtomicBool)>,
    input: Input<(TraitStack<dyn Drawable + 'static>, Vec<CommandBuffer>)>,

    signal_sender: Sender<()>,
    task: DedicatedTickTask<RenderTaskData>,
}

impl RenderTask {
    pub fn run(backend: Arc<StoryboardBackend>, renderer: StoryboardSurfaceRenderer) -> Self {
        let (input, output) = TripleBuffer::default().split();

        let (signal_sender, signal_receiver) = bounded(2);

        let configuration = renderer.configuration();

        let renderer_config = Arc::new((Mutex::new(configuration), AtomicBool::new(false)));

        let data = RenderTaskData {
            backend,
            configuration: renderer_config.clone(),
            signal_receiver,
            output,
            renderer,
        };

        let task = DedicatedTickTask::run(data, |data| {
            if let Ok(_) = data.signal_receiver.recv() {
                if data.configuration.1.load(Ordering::Relaxed) {
                    data.configuration.1.store(false, Ordering::Relaxed);

                    data.renderer
                        .set_configuration(*data.configuration.0.lock());
                }

                if data.output.update() {
                    if data.output.output_buffer().0.len() > 0 {
                        if let Some(res) = data.renderer.render(
                            data.backend.device(),
                            data.backend.queue(),
                            data.output.output_buffer().0.iter(),
                        ) {
                            data.backend.device().poll(Maintain::Wait);
                            data.backend.queue().submit(
                                iter::once(res.command_buffer)
                                    .chain(data.output.output_buffer().1.drain(..)),
                            );

                            res.surface_texture.present();
                            return;
                        }
                    }

                    if data.output.output_buffer().1.len() > 0 {
                        data.backend.device().poll(Maintain::Wait);
                        data.backend
                            .queue()
                            .submit(data.output.output_buffer().1.drain(..));
                    }
                }
            }
        });

        Self {
            renderer_config,
            input,
            signal_sender,
            task,
        }
    }

    pub fn configuration(&self) -> SurfaceConfiguration {
        *self.renderer_config.0.lock()
    }

    pub fn set_configuration(&self, configuration: SurfaceConfiguration) {
        *self.renderer_config.0.lock() = configuration;
        self.renderer_config.1.store(true, Ordering::Relaxed);
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
    configuration: Arc<(Mutex<SurfaceConfiguration>, AtomicBool)>,
    signal_receiver: Receiver<()>,
    output: Output<(TraitStack<dyn Drawable + 'static>, Vec<CommandBuffer>)>,
    renderer: StoryboardSurfaceRenderer,
}
