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
use render::{task::RenderTaskConfiguration, renderer::RendererData};
use std::{path::Path, sync::Arc, time::Duration};
use storyboard_core::euclid::{Point2D, Rect, Size2D};
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
    event_loop::EventLoop,
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
    pub render_task_config: RenderTaskConfiguration,

    window: Window,
    surface: Surface,
}

impl Storyboard {
    /// Initalize resources for storyboard app
    pub async fn init(
        window: Window,
        options: &BackendOptions,
        present_mode: PresentMode,
        trace_path: Option<&Path>,
    ) -> Result<Self, BackendInitError> {
        let instance = Instance::new(Backends::all());

        // Safety: window is valid object to create a surface
        let surface = unsafe { instance.create_surface(&window) };

        let backend = StoryboardBackend::init(
            &instance,
            Some(&surface),
            Features::empty(),
            options,
            trace_path,
        )
        .await?;

        let screen_format = *surface
            .get_supported_formats(backend.adapter())
            .get(0)
            .ok_or(BackendInitError::NoSuitableAdapter)?;
        let texture_data = TextureData::init(backend.device());

        Ok(Self {
            backend,
            screen_format,
            texture_data,

            present_mode,
            render_task_config: RenderTaskConfiguration::default(),

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
                screen: Rect::new(Point2D::zero(), win_size),
                screen_scale: self.window.scale_factor() as _,
            },
        );

        let mut render_task = Some(RenderTask::run(
            backend.clone(),
            surface_renderer,
            Arc::new(RendererData::new(self.screen_format)),
            self.render_task_config,
        ));

        let mut app_prop = StoryboardAppProp {
            backend,
            screen_format: self.screen_format,
            texture_data,
            elapsed: Duration::ZERO,
            window: Arc::new(self.window),
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
                        .configuration_mut()
                        .surface
                        .screen.size = win_size;
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

                    let mut configuration = app_state.render_task.configuration_mut();
                    configuration.surface.screen.size = win_size;
                    configuration.surface.screen_scale = *scale_factor as _;
                }

                _ => {}
            }

            app.update(&app_prop, &mut app_state);

            match &app_state.event {
                Event::MainEventsCleared => {
                    app_prop.elapsed = instant.elapsed();
                }

                Event::LoopDestroyed => {
                    app.unload(&app_prop);
                    app_state.render_task.interrupt();
                }

                _ => {}
            }
        })
    }
}
