/*
 * Created on Mon Sep 06 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{error::Error, fmt::Display, sync::Arc};

use super::data::observable::Observable;
use crate::unit::PixelUnit;
use euclid::Size2D;
use raw_window_handle::HasRawWindowHandle;
use wgpu::{Adapter, Backends, Device, Features, Limits, PowerPreference, PresentMode, Queue, RequestDeviceError, Surface, SurfaceConfiguration, SurfaceError, SurfaceTexture, TextureFormat};

#[derive(Debug)]
pub struct StoryboardBackend {
    device: Arc<Device>,
    queue: Arc<Queue>,

    instance: wgpu::Instance,
    adapter: Adapter,

    surface_config: Observable<SurfaceConfiguration>,
}

impl StoryboardBackend {
    async fn init_gpu(
        instance: &wgpu::Instance,
        compatible_surface: Option<&Surface>,
        options: &BackendOptions,
    ) -> Result<(Adapter, Device, Queue), BackendInitError> {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: options.power_preference,
                compatible_surface,
                force_fallback_adapter: options.force_fallback_adapter,
            })
            .await
            .ok_or(BackendInitError::NoSuitableAdapter)?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: options.features,
                    limits: options.limits.clone(),
                    label: Some("StoryboardBackend device"),
                },
                None, // Trace path
            )
            .await?;

        Ok((adapter, device, queue))
    }

    pub async fn init(options: BackendOptions) -> Result<Self, BackendInitError> {
        let instance = wgpu::Instance::new(options.backends);

        let (adapter, device, queue) = Self::init_gpu(&instance, None, &options).await?;

        let surface_config = Observable::new(SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: TextureFormat::Bgra8Unorm,
            width: 0,
            height: 0,
            present_mode: PresentMode::Immediate,
        });

        Ok(Self {
            device: Arc::new(device),
            queue: Arc::new(queue),

            instance,
            adapter,
            surface_config,
        })
    }

    pub async fn init_surface(
        window: &impl HasRawWindowHandle,
        options: BackendOptions,
    ) -> Result<(Self, Surface), BackendInitError> {
        let instance = wgpu::Instance::new(options.backends);
        let surface = unsafe { instance.create_surface(window) };

        let (adapter, device, queue) = Self::init_gpu(&instance, Some(&surface), &options).await?;

        let surface_format = surface
            .get_preferred_format(&adapter)
            .ok_or(BackendInitError::IncompatibleSurface)?;

        let surface_config = Observable::new(SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: 0,
            height: 0,
            present_mode: PresentMode::Immediate,
        });

        Ok((
            Self {
                device: Arc::new(device),
                queue: Arc::new(queue),

                instance,
                adapter,
                surface_config,
            },
            surface,
        ))
    }

    pub fn instance(&self) -> &wgpu::Instance {
        &self.instance
    }

    pub fn adapter(&self) -> &Adapter {
        &self.adapter
    }

    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }

    pub fn queue(&self) -> &Arc<Queue> {
        &self.queue
    }

    pub fn view(&self) -> Size2D<u32, PixelUnit> {
        let config = self.surface_config.as_ref();

        Size2D::new(config.width, config.height)
    }

    pub fn update_view(&mut self, size: Size2D<u32, PixelUnit>) {
        let mut config = self.surface_config.as_mut();

        config.width = size.width;
        config.height = size.height;
    }

    pub fn surface_config(&self) -> &SurfaceConfiguration {
        self.surface_config.as_ref()
    }

    pub fn update_present_mode(&mut self, present_mode: PresentMode) {
        self.surface_config.as_mut().present_mode = present_mode;
    }

    pub fn get_current_texture(&mut self, surface: &Surface) -> Result<SurfaceTexture, BackendFrameError> {
        let view = self.surface_config.as_ref();
        if view.width == 0 || view.height == 0 {
            return Err(BackendFrameError::EmptySurface);
        }

        if self.surface_config.unmark() {
            surface.configure(&self.device, self.surface_config.as_ref());
        }

        Ok(surface.get_current_texture()?)
    }
}

#[derive(Debug)]
pub struct BackendOptions {
    pub backends: Backends,

    pub power_preference: PowerPreference,
    pub force_fallback_adapter: bool,

    pub features: Features,
    pub limits: Limits,
}

impl Default for BackendOptions {
    fn default() -> Self {
        Self {
            backends: Backends::all(),
            power_preference: Default::default(),
            force_fallback_adapter: false,
            features: Default::default(),
            limits: Default::default()
        }
    }
}

#[derive(Debug)]
pub enum BackendInitError {
    NoSuitableAdapter,
    IncompatibleSurface,
    Device(RequestDeviceError),
}

impl From<RequestDeviceError> for BackendInitError {
    fn from(err: RequestDeviceError) -> Self {
        Self::Device(err)
    }
}

impl Display for BackendInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoSuitableAdapter => writeln!(f, "No suitable gpu adapter found"),
            Self::IncompatibleSurface => writeln!(f, "Incompatible surface"),

            Self::Device(err) => err.fmt(f),
        }
    }
}

impl Error for BackendInitError {}

#[derive(Debug)]
pub enum BackendFrameError {
    EmptySurface,
    Surface(SurfaceError),
}

impl From<SurfaceError> for BackendFrameError {
    fn from(err: SurfaceError) -> Self {
        Self::Surface(err)
    }
}

impl Display for BackendFrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptySurface => writeln!(f, "Surface is not presented"),

            Self::Surface(err) => err.fmt(f),
        }
    }
}

impl Error for BackendFrameError {}
