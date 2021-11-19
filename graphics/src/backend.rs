/*
 * Created on Mon Sep 06 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{error::Error, fmt::Display, sync::Arc};

use wgpu::{
    Adapter, Device, Features, Limits, PowerPreference, Queue,
    RequestDeviceError, Surface,
};

#[derive(Debug)]
pub struct StoryboardBackend {
    device: Arc<Device>,
    queue: Arc<Queue>,

    features: Features,

    adapter: Adapter,
}

impl StoryboardBackend {
    async fn init_gpu(
        instance: &wgpu::Instance,
        compatible_surface: Option<&Surface>,
        options: &BackendOptions,
    ) -> Result<(Adapter, Features, Device, Queue), BackendInitError> {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: options.power_preference,
                compatible_surface,
                force_fallback_adapter: options.force_fallback_adapter,
            })
            .await
            .ok_or(BackendInitError::NoSuitableAdapter)?;

        let features = adapter.features();

        if !features.contains(options.features) {
            return Err(BackendInitError::IncompatibleFeatures(
                options.features - features,
            ));
        }

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features,
                    limits: options.limits.clone(),
                    label: Some("StoryboardBackend device"),
                },
                None, // Trace path
            )
            .await?;

        Ok((adapter, features, device, queue))
    }

    pub async fn init(instance: &wgpu::Instance, options: BackendOptions) -> Result<Self, BackendInitError> {
        let (adapter, features, device, queue) = Self::init_gpu(instance, None, &options).await?;

        Ok(Self {
            device: Arc::new(device),
            queue: Arc::new(queue),

            features,

            adapter,
        })
    }

    pub async fn init_surface(instance: &wgpu::Instance, surface: &Surface, options: BackendOptions) -> Result<Self, BackendInitError> {
        let (adapter, features, device, queue) = Self::init_gpu(instance, Some(surface), &options).await?;

        Ok(Self {
            device: Arc::new(device),
            queue: Arc::new(queue),

            features,

            adapter,
        })
    }

    pub fn adapter(&self) -> &Adapter {
        &self.adapter
    }

    pub fn features(&self) -> Features {
        self.features
    }

    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }

    pub fn queue(&self) -> &Arc<Queue> {
        &self.queue
    }
}

#[derive(Debug)]
pub struct BackendOptions {
    pub power_preference: PowerPreference,
    pub force_fallback_adapter: bool,

    pub features: Features,
    pub limits: Limits,
}

impl Default for BackendOptions {
    fn default() -> Self {
        Self {
            power_preference: Default::default(),
            force_fallback_adapter: false,
            features: Default::default(),
            limits: Default::default(),
        }
    }
}

#[derive(Debug)]
pub enum BackendInitError {
    NoSuitableAdapter,
    IncompatibleFeatures(Features),
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
            Self::IncompatibleFeatures(features) => {
                writeln!(f, "Incompatible features: {:?}", features)
            }

            Self::Device(err) => err.fmt(f),
        }
    }
}

impl Error for BackendInitError {}
