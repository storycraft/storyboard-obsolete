/*
 * Created on Mon Nov 22 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use futures::executor::block_on;
use storyboard::graphics::{
    backend::{BackendOptions, StoryboardBackend},
    wgpu::{Backends, CommandEncoderDescriptor, ComputePassDescriptor, Instance},
};

fn main() {
    let backend = block_on(StoryboardBackend::init(
        &Instance::new(Backends::all()),
        None,
        &BackendOptions::default(),
    ))
    .unwrap();

    let mut encoder = backend
        .device()
        .create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Compute test encoder"),
        });

    let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
        label: Some("Compute test compute pass"),
    });

    pass.dispatch(32, 32, 1);
}
