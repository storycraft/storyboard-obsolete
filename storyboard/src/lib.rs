// Nightly features
#![feature(generic_associated_types)]

pub mod app;

// Reexports
pub use storyboard_core as core;
pub use storyboard_render as render;
pub use storyboard_texture as texture;
pub use winit;

use instant::Instant;

use app::{StoryboardApp, StoryboardAppProp, StoryboardAppState};
use std::{sync::Arc, time::Duration, path::Path};
use storyboard_core::euclid::Size2D;
use storyboard_render::{
    backend::{BackendInitError, BackendOptions, StoryboardBackend},
    renderer::surface::{StoryboardSurfaceRenderer, SurfaceConfiguration},
    task::RenderTask,
    wgpu::TextureFormat,
    wgpu::{Backends, Features, Instance, PresentMode, Surface},
};
use storyboard_texture::render::data::TextureData;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

/// Storyboard app.
/// Holds graphics, windows resources for app before start.
#[derive(Debug)]
pub struct Storyboard {
    backend: StoryboardBackend,
    screen_format: TextureFormat,
    texture_data: TextureData,

    pub present_mode: PresentMode,

    window: Window,
    surface: Surface,
}

impl Storyboard {
    /// Initalize resources for storyboard app
    pub async fn init(
        window: Window,
        options: &BackendOptions,
        present_mode: PresentMode,
        trace_path: Option<&Path>
    ) -> Result<Self, BackendInitError> {
        let instance = Instance::new(Backends::all());

        // Safety: window is valid object to create a surface
        let surface = unsafe { instance.create_surface(&window) };

        let backend =
            StoryboardBackend::init(&instance, Some(&surface), Features::empty(), options, trace_path).await?;

        let screen_format = *surface
            .get_supported_formats(backend.adapter())
            .get(0)
            .ok_or_else(|| BackendInitError::NoSuitableAdapter)?;
        let texture_data = TextureData::init(backend.device());

        Ok(Self {
            backend,
            screen_format,
            texture_data,

            present_mode,

            window,
            surface,
        })
    }

    pub const fn backend(&self) -> &StoryboardBackend {
        &self.backend
    }

    pub const fn screen_format(&self) -> TextureFormat {
        self.screen_format
    }

    pub const fn window(&self) -> &Window {
        &self.window
    }

    pub const fn texture_data(&self) -> &TextureData {
        &self.texture_data
    }

    /// Start app
    ///
    /// Start render thread and run given inital [StoryboardApp].
    pub fn run(self, event_loop: EventLoop<()>, mut app: impl StoryboardApp + 'static) -> ! {
        let backend = Arc::new(self.backend);
        let texture_data = Arc::new(self.texture_data);

        let win_size = {
            let (width, height) = self.window.inner_size().into();

            Size2D::new(width, height)
        };

        let surface_renderer = StoryboardSurfaceRenderer::new(
            self.surface,
            SurfaceConfiguration {
                present_mode: self.present_mode,
                screen_size: win_size,
                screen_scale: self.window.scale_factor() as _,
            },
            self.screen_format,
        );

        let mut render_task = Some(RenderTask::run(backend.clone(), surface_renderer));

        let mut app_prop = StoryboardAppProp {
            backend,
            screen_format: self.screen_format,
            texture_data,
            elapsed: Duration::ZERO,
            window: self.window,
        };
        app.load(&app_prop);

        event_loop.run(move |event, _, control_flow| {
            let instant = Instant::now();

            let mut app_state = StoryboardAppState {
                render_task: &mut render_task.as_mut().unwrap(),
                control_flow,
                event,
            };

            match &app_state.event {
                Event::WindowEvent {
                    window_id: _,
                    event: WindowEvent::Resized(size),
                } => {
                    let win_size = {
                        let (width, height) = (*size).into();

                        Size2D::new(width, height)
                    };

                    app_state
                        .render_task
                        .set_configuration(SurfaceConfiguration {
                            screen_size: win_size,
                            ..app_state.render_task.configuration()
                        });
                }

                Event::WindowEvent {
                    window_id: _,
                    event:
                        WindowEvent::ScaleFactorChanged {
                            scale_factor,
                            new_inner_size,
                        },
                } => {
                    let win_size = {
                        let (width, height) = (**new_inner_size).into();

                        Size2D::new(width, height)
                    };

                    app_state
                        .render_task
                        .set_configuration(SurfaceConfiguration {
                            screen_size: win_size,
                            screen_scale: *scale_factor as _,
                            ..app_state.render_task.configuration()
                        });
                }

                _ => {}
            }

            app.update(&app_prop, &mut app_state);

            if let ControlFlow::Exit = app_state.control_flow {
                app.unload(&app_prop);
                app_state.render_task.interrupt();
                return;
            }

            match &app_state.event {
                Event::MainEventsCleared => {
                    app_prop.elapsed = instant.elapsed();
                }

                _ => {}
            }
        })
    }
}
