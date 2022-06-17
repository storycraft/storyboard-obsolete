use std::{
    io, iter,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use crossbeam_channel::{bounded, Receiver, Sender};
use parking_lot::Mutex;
use storyboard_core::{
    graphics::{
        component::Drawable,
        renderer::surface::{StoryboardSurfaceRenderer, SurfaceConfiguration},
    },
    tick_task::DedicatedTickTask,
    trait_stack::TraitStack,
};
use triple_buffer::{Input, Output, TripleBuffer};

#[derive(Debug)]
pub struct RenderTask {
    renderer_config: Arc<(Mutex<SurfaceConfiguration>, AtomicBool)>,
    input: Input<TraitStack<dyn Drawable + 'static>>,
    signal_sender: Option<Sender<()>>,
    task: DedicatedTickTask<RenderTaskData>,
}

impl RenderTask {
    pub fn run(renderer: StoryboardSurfaceRenderer) -> io::Result<Self> {
        let (input, output) = TripleBuffer::default().split();
        let (signal_sender, signal_receiver) = bounded(1);

        let configuration = renderer.configuration();

        let renderer_config = Arc::new((Mutex::new(configuration), AtomicBool::new(false)));

        let data = RenderTaskData {
            configuration: renderer_config.clone(),
            signal_receiver,
            output,
            renderer,
        };

        let task = DedicatedTickTask::run(data, |data| {
            if let Ok(_) = data.signal_receiver.recv() {
                let drawables = data.output.read();

                if data.configuration.1.load(Ordering::Relaxed) {
                    data.configuration.1.store(false, Ordering::Relaxed);
    
                    data.renderer
                        .set_configuration(*data.configuration.0.lock());
                }
    
                if let Some(res) = data.renderer.render(&drawables) {
                    data.renderer
                        .backend()
                        .queue()
                        .submit(iter::once(res.command_buffer));
                    res.surface_texture.present();
                }
            }
        })?;

        Ok(Self {
            renderer_config,
            input,
            signal_sender: Some(signal_sender),
            task,
        })
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
        self.signal_sender.take();
        self.task.interrupt()
    }

    pub fn push(&mut self, item: impl Drawable + 'static) {
        self.input.input_buffer().push(item);
    }

    pub fn submit(&mut self) {
        if let Some(signal_sender) = &self.signal_sender {
            self.input.publish();
            self.input.input_buffer().clear();
            signal_sender.send(()).ok();
            self.task.tick();
        }
    }

    pub fn threaded(&self) -> bool {
        self.task.threaded()
    }

    pub fn join(self) -> StoryboardSurfaceRenderer {
        self.task.join().renderer
    }
}

#[derive(Debug)]
struct RenderTaskData {
    configuration: Arc<(Mutex<SurfaceConfiguration>, AtomicBool)>,
    signal_receiver: Receiver<()>,
    output: Output<TraitStack<dyn Drawable + 'static>>,
    renderer: StoryboardSurfaceRenderer,
}
