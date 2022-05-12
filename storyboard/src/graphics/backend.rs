/*
 * Created on Mon Sep 06 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{error::Error, fmt::Display};

use storyboard_core::wgpu::{
    Adapter, Device, DeviceDescriptor, Features, Instance, Limits, PowerPreference, Queue,
    RequestAdapterOptions, RequestDeviceError, Surface,
};

#[derive(Debug)]
pub struct StoryboardBackend {
    device: Device,
    queue: Queue,

    features: Features,

    adapter: Adapter,
}

impl StoryboardBackend {
    pub async fn init(
        instance: &Instance,
        compatible_surface: Option<&Surface>,
        features: Features,
        options: &BackendOptions,
    ) -> Result<Self, BackendInitError> {
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: options.power_preference,
                compatible_surface,
                force_fallback_adapter: options.force_fallback_adapter,
            })
            .await
            .ok_or(BackendInitError::NoSuitableAdapter)?;

        let adapter_features = adapter.features();

        if !adapter_features.contains(features) {
            return Err(BackendInitError::IncompatibleFeatures(
                features - adapter_features,
            ));
        }

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    features,
                    limits: options.limits.clone(),
                    label: Some("StoryboardBackend device"),
                },
                None, // Trace path
            )
            .await?;

        Ok(Self {
            device,
            queue,

            features,

            adapter,
        })
    }

    pub const fn adapter(&self) -> &Adapter {
        &self.adapter
    }

    pub const fn features(&self) -> Features {
        self.features
    }

    pub const fn device(&self) -> &Device {
        &self.device
    }

    pub const fn queue(&self) -> &Queue {
        &self.queue
    }
}

#[derive(Debug)]
pub struct BackendOptions {
    pub power_preference: PowerPreference,
    pub force_fallback_adapter: bool,

    pub limits: Limits,
}

impl Default for BackendOptions {
    fn default() -> Self {
        Self {
            power_preference: Default::default(),
            force_fallback_adapter: false,
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
