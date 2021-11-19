/*
 * Created on Fri Nov 12 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

pub mod component;
pub mod graphics;
pub mod id_gen;
pub mod observable;
pub mod state;
pub mod store;
pub mod thread;

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use component::DrawSpace;
pub use euclid as math;
use math::{Point2D, Rect, Size2D};
pub use palette as color;
pub use ringbuffer;
use thread::render::{RenderConfiguration, RenderQueue, RenderThread};
pub use wgpu;
pub use winit as window;

use wgpu::{
    Backends, BlendState, ColorTargetState, ColorWrites, Instance, PresentMode, Surface,
    TextureFormat,
};

use graphics::{
    backend::{BackendOptions, StoryboardBackend},
    renderer::RenderData,
    texture::{Texture2D, TextureData},
    PixelUnit,
};

use ringbuffer::{ConstGenericRingBuffer, RingBufferWrite};

use state::{State, StateStatus, StateSystem};
use window::{
    event::Event,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

#[derive(Debug)]
pub struct Storyboard {
    event_loop: EventLoop<()>,

    backend: StoryboardBackend,

    render_data: RenderData,
    texture_data: TextureData,

    pub render_present_mode: PresentMode,

    window: Window,
    surface: Surface,
}

impl Storyboard {
    pub async fn init(builder: WindowBuilder, options: &BackendOptions) -> Self {
        let event_loop = EventLoop::new();
        let instance = Instance::new(Backends::all());

        let window = builder.build(&event_loop).unwrap();
        let surface = unsafe { instance.create_surface(&window) };

        let backend = StoryboardBackend::init(&instance, Some(&surface), options)
            .await
            .unwrap();

        let framebuffer_format = surface.get_preferred_format(backend.adapter()).unwrap();

        let texture_data = TextureData::init(backend.device(), framebuffer_format);
        let render_data = RenderData::init(
            backend.device(),
            backend.queue(),
            &texture_data,
            &[ColorTargetState {
                format: framebuffer_format,
                blend: Some(BlendState::ALPHA_BLENDING),
                write_mask: ColorWrites::all(),
            }],
            None,
        );

        Self {
            event_loop,

            backend,

            texture_data,
            render_data,

            render_present_mode: PresentMode::Fifo,

            window,
            surface,
        }
    }

    pub const fn window(&self) -> &Window {
        &self.window
    }

    pub fn run(self, state: impl StoryboardState + 'static) -> ! {
        let win_size = self.window.inner_size();

        let graphics = GraphicsData {
            texture_data: Arc::new(self.texture_data),
            render_data: Arc::new(self.render_data),

            backend: Arc::new(self.backend),
        };

        let system_prop = StoryboardSystemProp {
            window: self.window,
            graphics,
        };

        let render_thread = RenderThread::run(
            system_prop.graphics.backend.clone(),
            self.surface,
            system_prop
                .graphics
                .texture_data
                .framebuffer_texture_format(),
            system_prop.graphics.render_data.clone(),
            system_prop.graphics.texture_data.clone(),
            RenderConfiguration {
                size: Size2D::new(win_size.width, win_size.height),
                present_mode: self.render_present_mode,
            },
        );

        let mut system_state = StoryboardSystemState {
            events: ConstGenericRingBuffer::new(),
            screen: DrawSpace::new_screen(Rect::new(
                Point2D::zero(),
                Size2D::new(win_size.width as f32, win_size.height as f32),
            )),
            elapsed: Duration::ZERO,

            render_thread,
        };

        let mut state_system = StateSystem::new(Box::new(state), system_prop);

        self.event_loop.run(move |event, _, control_flow| {
            let instant = Instant::now();

            if state_system.finished() {
                system_state.render_thread.interrupt();
                *control_flow = ControlFlow::Exit;
                return;
            }

            if let Some(event) = event.to_static() {
                system_state.events.push(event);
            }

            state_system.run(&mut system_state);

            *control_flow = ControlFlow::Poll;
            system_state.elapsed = instant.elapsed();
        })
    }
}

pub trait StoryboardState: State<StoryboardSystemProp, StoryboardSystemState> {
    fn update(
        &mut self,
        system_prop: &StoryboardSystemProp,
        system_state: &mut StoryboardSystemState,
    ) -> StateStatus<StoryboardSystemProp, StoryboardSystemState>;

    fn load(&mut self, system_prop: &StoryboardSystemProp);
    fn unload(&mut self, system_prop: &StoryboardSystemProp);
}

impl<T: StoryboardState> State<StoryboardSystemProp, StoryboardSystemState> for T {
    fn update(
        &mut self,
        system_prop: &StoryboardSystemProp,
        system_state: &mut StoryboardSystemState,
    ) -> StateStatus<StoryboardSystemProp, StoryboardSystemState> {
        StoryboardState::update(self, system_prop, system_state)
    }

    fn load(&mut self, system_prop: &StoryboardSystemProp) {
        StoryboardState::load(self, system_prop)
    }

    fn unload(&mut self, system_prop: &StoryboardSystemProp) {
        StoryboardState::unload(self, system_prop)
    }
}

pub struct StoryboardSystemProp {
    pub window: Window,
    pub graphics: GraphicsData,
}

pub struct StoryboardSystemState {
    pub events: ConstGenericRingBuffer<Event<'static, ()>, 32>,
    pub screen: DrawSpace,
    pub elapsed: Duration,

    render_thread: RenderThread,
}

impl StoryboardSystemState {
    pub fn submit_render_queue(&mut self, render_queue: RenderQueue) -> bool {
        self.render_thread.submit_queue(render_queue)
    }

    pub const fn render_thread(&self) -> &RenderThread {
        &self.render_thread
    }
}

pub struct GraphicsData {
    pub render_data: Arc<RenderData>,
    pub texture_data: Arc<TextureData>,

    pub backend: Arc<StoryboardBackend>,
}

impl GraphicsData {
    pub fn create_texture(&self, format: TextureFormat, size: Size2D<u32, PixelUnit>) -> Texture2D {
        self.texture_data
            .create_texture(self.backend.device(), format, size)
    }
}