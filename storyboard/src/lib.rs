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
    compositor::{ComponentCompositor, StoryboardCompositor},
    renderer::StoryboardRenderer,
    texture::TextureData,
};
use state::{StoryboardStateData, StoryboardSystemProp, StoryboardSystemState};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use storyboard_core::euclid::Size2D;
use storyboard_core::{
    state::{State, StateSystem, SystemStatus},
    wgpu::{Backends, Features, Instance, PresentMode, Surface},
};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub type DefaultCompositor = StoryboardCompositor;

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

    /// Start app with default compositor
    ///
    /// Start render thread and run given inital [StoryboardState].
    /// The state system will wait for event loop event when the state returns [SystemStatus::Wait].
    pub fn run(self, state: impl State<StoryboardStateData<DefaultCompositor>> + 'static) {
        let compositor = DefaultCompositor::init(&self.backend, &self.texture_data);

        self.run_with_compositor(state, compositor)
    }

    /// Start app with custom compositor
    pub fn run_with_compositor<Compositor: ComponentCompositor + Send + Sync + 'static>(
        self,
        state: impl State<StoryboardStateData<Compositor>> + 'static,
        compositor: Compositor,
    ) {
        let compositor = Arc::new(compositor);

        let mut system_prop = StoryboardSystemProp {
            backend: Arc::new(self.backend),
            texture_data: Arc::new(self.texture_data),
            elapsed: Duration::ZERO,
            window: self.window,
        };

        let mut state_system = StateSystem::new(Box::new(state), &system_prop);

        let mut render_task = SurfaceRenderTask::new(
            system_prop.backend.clone(),
            self.surface,
            system_prop.texture_data.framebuffer_texture_format(),
            StoryboardRenderer::new(compositor),
        );

        let win_size = {
            let (width, height) = system_prop.window.inner_size().into();

            Size2D::new(width, height)
        };
        render_task.reconfigure(win_size, self.render_present_mode);

        self.event_loop.run(move |event, _, control_flow| {
            let instant = Instant::now();

            let mut system_state = StoryboardSystemState {
                event,
                components: Vec::new(),
            };

            // TODO:: Threading
            if let Event::WindowEvent {
                window_id: _,
                event: WindowEvent::Resized(size),
            } = &system_state.event
            {
                if *size != PhysicalSize::new(0, 0) {
                    let win_size = {
                        let (width, height) = (*size).into();
    
                        Size2D::new(width, height)
                    };
                    
                    render_task.reconfigure(win_size, self.render_present_mode);
                }
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
            if system_state.components.len() > 0 {
                if system_prop.window.inner_size() != PhysicalSize::new(0, 0) {
                    render_task.render(&system_state.components);
                }
                system_state.components.clear();
            }

            system_prop.elapsed = instant.elapsed();
        });
    }
}
