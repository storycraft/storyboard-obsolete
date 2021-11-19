/*
 * Created on Fri Nov 05 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{error::Error, fmt::Display};

use euclid::Size2D;
use raw_window_handle::HasRawWindowHandle;
use wgpu::{
    Adapter, Device, Instance, PresentMode, Surface, SurfaceConfiguration, SurfaceError,
    SurfaceTexture, TextureFormat,
};

use crate::{backend::{BackendInitError, BackendOptions, StoryboardBackend}, data::observable::Observable, unit::PixelUnit};

#[derive(Debug)]
pub struct StoryboardSurface {
    surface: Surface,
    surface_config: Observable<SurfaceConfiguration>,
}

impl StoryboardSurface {
    pub fn init(
        instance: &Instance,
        window: &impl HasRawWindowHandle,
    ) -> Result<Self, SurfaceInitError> {
        let surface = unsafe { instance.create_surface(window) };

        let surface_config = Observable::new(SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: TextureFormat::Bgra8UnormSrgb,
            width: 0,
            height: 0,
            present_mode: PresentMode::Immediate,
        });

        Ok(Self {
            surface,
            surface_config,
        })
    }

    pub fn get_preferred_format(&self, adapter: &Adapter) -> Option<TextureFormat> {
        self.surface.get_preferred_format(adapter)
    }

    pub fn view(&self) -> Size2D<u32, PixelUnit> {
        let config = self.surface_config.inner_ref();

        Size2D::new(config.width, config.height)
    }

    pub fn update_view(&mut self, size: Size2D<u32, PixelUnit>) {
        let mut config = self.surface_config.inner_mut();

        config.width = size.width;
        config.height = size.height;
    }

    pub fn surface_config(&self) -> &SurfaceConfiguration {
        self.surface_config.inner_ref()
    }

    pub fn update_format_for(&mut self, adapter: &Adapter) -> Option<TextureFormat> {
        let format = self.surface.get_preferred_format(adapter)?;

        self.surface_config.inner_mut().format = format;

        Some(format)
    }

    pub fn update_present_mode(&mut self, present_mode: PresentMode) {
        self.surface_config.inner_mut().present_mode = present_mode;
    }

    pub fn get_current_texture(
        &mut self,
        device: &Device,
    ) -> Result<SurfaceTexture, BackendFrameError> {
        let view = self.surface_config.inner_ref();
        if view.width == 0 || view.height == 0 {
            return Err(BackendFrameError::EmptySurface);
        }

        if self.surface_config.unmark() {
            self.surface
                .configure(&device, self.surface_config.inner_ref());
        }

        Ok(self.surface.get_current_texture()?)
    }

    pub async fn create_backend(&self, instance: &Instance, options: BackendOptions) -> Result<StoryboardBackend, BackendInitError> {
        StoryboardBackend::init_surface(instance, &self.surface, options).await
    }
}

#[derive(Debug)]
pub enum SurfaceInitError {
    IncompatibleSurface,
}

impl Display for SurfaceInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IncompatibleSurface => writeln!(f, "Incompatible surface"),
        }
    }
}

impl Error for SurfaceInitError {}

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
