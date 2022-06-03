/*
 * Created on Mon May 02 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

// Nightly features
#![feature(generic_associated_types)]

pub mod graphics;
pub mod state;
pub mod task;

// Reexports
pub use storyboard_core as core;
use task::render::SurfaceRenderTask;
pub use winit;

use graphics::{
    backend::{BackendInitError, BackendOptions, StoryboardBackend},
    renderer::StoryboardRenderer,
    texture::TextureData,
};
use state::{StoryboardStateData, StoryboardSystemProp, StoryboardSystemState};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use storyboard_core::euclid::Size2D;
use storyboard_core::{
    state::{State, StateSystem, SystemStatus},
    store::Store,
    wgpu::{Backends, Features, Instance, PresentMode, Surface},
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

/// Storyboard app.
/// Holds graphics, windows resources for app before start.
#[derive(Debug)]
pub struct Storyboard {
    event_loop: EventLoop<()>,

    backend: StoryboardBackend,
    texture_data: TextureData,

    pub render_present_mode: PresentMode,

    window: Window,
    surface: Surface,
}

impl Storyboard {
    /// Initalize resources for storyboard app
    pub async fn init(
        builder: WindowBuilder,
        options: &BackendOptions,
    ) -> Result<Self, BackendInitError> {
        let event_loop = EventLoop::new();

        let window = builder.build(&event_loop).unwrap();

        let instance = Instance::new(Backends::all());

        // Safety: window is valid object to create a surface
        let surface = unsafe { instance.create_surface(&window) };

        let backend =
            StoryboardBackend::init(&instance, Some(&surface), Features::empty(), options).await?;

        let framebuffer_format = surface.get_preferred_format(backend.adapter()).unwrap();
        let texture_data = TextureData::init(backend.device(), framebuffer_format);

        Ok(Self {
            event_loop,

            backend,
            texture_data,

            render_present_mode: PresentMode::Mailbox,

            window,
            surface,
        })
    }

    pub const fn backend(&self) -> &StoryboardBackend {
        &self.backend
    }

    pub const fn texture_data(&self) -> &TextureData {
        &self.texture_data
    }

    /// Start app
    ///
    /// Start render thread and run given inital [StoryboardState].
    /// The state system will wait for event loop event when the state returns [SystemStatus::Wait].
    pub fn run(self, state: impl State<StoryboardStateData> + 'static) -> ! {
        let backend = Arc::new(self.backend);
        let texture_data = Arc::new(self.texture_data);

        let draw_resources = Arc::new(Store::new());

        let surface_render_task = Arc::new(Mutex::new(SurfaceRenderTask::new(
            backend.clone(),
            texture_data.clone(),
            self.surface,
            StoryboardRenderer::new(draw_resources.clone()),
        )));

        let mut system_prop = StoryboardSystemProp {
            backend,
            texture_data,
            elapsed: Duration::ZERO,
            window: self.window,
            render_task: surface_render_task.clone(),
        };

        let mut state_system = StateSystem::new(Box::new(state), &system_prop);

        let win_size = {
            let (width, height) = system_prop.window.inner_size().into();

            Size2D::new(width, height)
        };

        surface_render_task
            .lock()
            .unwrap()
            .reconfigure(win_size, self.render_present_mode);

        self.event_loop.run(move |event, _, control_flow| {
            let instant = Instant::now();

            let mut system_state = StoryboardSystemState { event };

            // TODO:: Threading
            if let Event::WindowEvent {
                window_id: _,
                event: WindowEvent::Resized(size),
            } = &system_state.event
            {
                let win_size = {
                    let (width, height) = (*size).into();

                    Size2D::new(width, height)
                };

                surface_render_task
                    .lock()
                    .unwrap()
                    .reconfigure(win_size, self.render_present_mode);
            }

            let status = state_system.run(&system_prop, &mut system_state);

            if state_system.finished() {
                *control_flow = ControlFlow::Exit;
                return;
            } else {
                *control_flow = match status {
                    SystemStatus::Poll => ControlFlow::Poll,
                    SystemStatus::Wait => ControlFlow::Wait,
                }
            };

            // TODO:: Threading
            if let Event::RedrawRequested(_) = system_state.event {
                surface_render_task.lock().unwrap().render();
            }

            system_prop.elapsed = instant.elapsed();
        })
    }
}
